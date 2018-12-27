use std::convert::*;
use std::thread;
use std::sync::mpsc::channel;
use std::io::{copy, Read};
use std::cmp::{Ordering, Ord, PartialOrd, PartialEq};
use scraper::{Html, Selector};
use image::{self, GenericImageView};
use mime_sniffer::MimeTypeSniffer;
use reqwest;
use url::Url;

pub type Result<T> = std::result::Result<T, Error>;

/// infer an icon using the default Inferer.
pub fn infer_icon(url: &str) -> Result<Icon> {
    Inferer::default().infer(url)
}

/// Inferer infers the best icon for a given url. 
pub struct Inferer<D: Downloader> {
    /// client downloads the icon data into a buffer. 
    pub client: D,
}

/// Default to using reqwest crate to perform network calls. 
impl Inferer<reqwest::Client> {
    fn default() -> Inferer<reqwest::Client> {
        Inferer {
            client: reqwest::ClientBuilder::new().build().unwrap(),
        }
    }
}

/// infer the best icon for a url by downloading icon links and comparing for
/// size, preferring the largest. 
impl<D: Downloader + Clone + Send + 'static> Inferer<D> {
    fn infer(&self, url: &str) -> Result<Icon> {
        let (tx, tr) = channel();
        let mut workers = vec![];
        for link in self.scrape(url)? {
            let client = self.client.clone();
            let tx = tx.clone();
            workers.push(thread::spawn(move || {
                let icon = Icon::download(&client, &link);
                tx.send(icon).expect("sending icon result over channel");
            }));
        }
        let mut icons = vec![];
        for _ in workers {
            icons.push(tr.recv().expect("receiving icon from channel")?);
        }
        icons.sort();
        match icons.into_iter().last() {
            Some(icon) => Ok(icon),
            None => Err(Error::Markup("no icons found".into())),
        }
    }
    /// Scrape icon links form the html markup at the given url.
    // FIXME: - Should we return stronger types, like Vec<Url>? 
    //        - Should the scraping errors simply be ignored? They would only 
    //          be useful for debugging, not for users, so how to expose for 
    //          debugging?
    fn scrape(&self, url: &str) -> Result<Vec<String>> {
        let mut body = self.client.get(url)?;
        let mut buf = String::new();
        body.read_to_string(&mut buf)?;
        let doc = Html::parse_document(&buf);
        let link_el = Selector::parse("link").unwrap();
        let links: Vec<String> = doc.select(&link_el)
            .map(|el| {
                let el = el.value();
                let rel = match el.attr("rel") {
                    Some(rel) => rel,
                    None => return Err(Error::Markup("no rel attribute on link element".into())),
                };
                let href = match el.attr("href") {
                    Some(href) => href,
                    None => return Err(Error::Markup("no href attribute on link element".into())),
                };
                if !rel.contains("icon") {
                    return Err(Error::Markup("link[rel] does not include 'icon'".into()));
                }
                Ok(href.into())
            })
            .filter(|r: &Result<String>| r.is_ok())
            .map(|r: Result<String>| r.unwrap())
            .collect();
        Ok(links)
    }
}

/// Downloader performs network requests. 
/// The default Downloader uses reqwest crate. 
pub trait Downloader {
    fn get(&self, url: &str) -> Result<Box<Read>>;
}

impl Downloader for reqwest::Client {
    fn get(&self, url: &str) -> Result<Box<Read>> {
        Ok(Box::new(reqwest::get(url)?))
    }
}

/// Icon is icon detected for a website. 
#[derive(Eq, Debug)]
pub struct Icon {
    pub source: String,
    pub name: String,
    pub size: usize,
    pub ext: String,
    pub mime: String,
    pub buffer: Vec<u8>, 
    pub dimensions: Size,
}

impl Icon {
    fn download(client: &impl Downloader, href: &str) -> Result<Icon> {
        let mut response = client.get(href)?;
        let mut icon_data: Vec<u8> = vec![];
        copy(&mut response, &mut icon_data)?;
        Ok(Icon{
            source: href.into(),
            name: Url::parse(href)?.host_str().unwrap_or_else(|| "").into(),
            // Assumes the url ends with a valid file extension.
            ext: format!(".{0}", href.split('.').last().unwrap()),
            mime: MimeTypeSniffer::sniff_mime_type(&icon_data).unwrap().into(),
            size: icon_data.len(),
            dimensions: image::load_from_memory(&icon_data)?.dimensions().into(),
            buffer: icon_data,
        })
    }
}

impl PartialOrd for Icon {
    fn partial_cmp(&self, other: &Icon) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Icon {
    fn cmp(&self, other: &Icon) -> Ordering {
        self.dimensions.cmp(&other.dimensions)
    }
}

impl PartialEq for Icon {
    fn eq(&self, other: &Icon) -> bool {
        self.name == other.name && self.dimensions == other.dimensions 
    }
}

impl std::convert::AsRef<[u8]> for Icon {
    fn as_ref(&self) -> &[u8] {
        &self.buffer
    }
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

/// parse dimensions like "64x64".
impl std::str::FromStr for Size {
    type Err = ParseError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('x').collect();
        if parts.len() < 2 {
            return Err(ParseError::Size(format!("input: {}", s)));
        }
        Ok(Size{
            w: parts[0].parse()?,
            h: parts[1].parse()?,
        })
    }
}

impl From<(u32, u32)> for Size {
    fn from(d: (u32, u32)) -> Self {
        Size{w: d.0, h: d.1}
    }
}

#[derive(Debug)]
pub enum Error {
    Parse(ParseError),
    IO(std::io::Error),
    Download(reqwest::Error),
    Image(image::ImageError),
    Markup(String),
}

#[derive(Debug)]
pub enum ParseError {
    Int(std::num::ParseIntError),
    Url(url::ParseError),
    Size(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Parse(err) => write!(f, "parsing: {}", err),
            Error::IO(err) => write!(f, "io: {}", err),
            Error::Download(err) => write!(f, "downloading: {}", err),
            Error::Image(err) => write!(f, "image: {}", err),
            Error::Markup(s) => write!(f, "markup: {}", s),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::Int(err) => write!(f, "int: {}", err),
            ParseError::Url(err) => write!(f, "url: {}", err),
            ParseError::Size(err) => write!(f, "size: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Parse(err) => Some(err),
            Error::IO(err) => Some(err),
            Error::Download(err) => Some(err),
            Error::Image(err) => Some(err),
            Error::Markup(_) => None,
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::Int(err) => Some(err),
            ParseError::Url(err) => Some(err),
            ParseError::Size(_) => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(err)
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::Parse(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Download(err)
    }
}

impl From<image::ImageError> for Error {
    fn from(err: image::ImageError) -> Self {
        Error::Image(err)
    }
}

impl From<std::num::ParseIntError> for ParseError {
    fn from(err: std::num::ParseIntError) -> Self {
        ParseError::Int(err)
    }
}

impl From<url::ParseError> for ParseError {
    fn from(err: url::ParseError) -> Self {
        ParseError::Url(err)
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::Parse(ParseError::Url(err))
    }
}