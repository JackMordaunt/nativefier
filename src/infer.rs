use std::{
    convert::*,
    error::Error,
    io::{
        copy,
        Read,
    },
};
use scraper::{
    Html,
    Selector,
    ElementRef,
};
use reqwest;
use url::Url;

pub type Result<T> = std::result::Result<T, Box<Error>>;

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

impl<D: Downloader> Inferer<D> {
    fn infer(&self, url: &str) -> Result<Icon> {
        let mut body = self.client.get(url)?;
        let mut buf = String::new();
        body.read_to_string(&mut buf)?;
        let doc = Html::parse_document(&buf);
        let apple_touch = Selector::parse("link[rel=\"apple-touch-icon\"]").unwrap();
        for link in doc.select(&apple_touch) {
            return Ok(Icon::download(&self.client, &link)?);
        }
        // Look for high res icon.
        let icon_link = Selector::parse("link[rel=\"icon\"]").unwrap();
        let mut links: Vec<ElementRef> = doc.select(&icon_link).collect();
        links.sort_by(|left, right| {
            let l_size: Size = match left.value().attr("sizes") {
                Some(left_sizes) => left_sizes.parse().unwrap_or(Size::empty()),
                None => Size::empty(),
            };
            let r_size: Size = match right.value().attr("sizes") {
                Some(right_sizes) => right_sizes.parse().unwrap_or(Size::empty()),
                None => Size::empty(),
            };
            l_size.cmp(&r_size)
        });
        for link in links {
            return Ok(Icon::download(&self.client, &link)?);
        }
        Err(format!("no icon found for {}", url).into())
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

/// Icon is the highest resolution icon detected for a website. 
pub struct Icon {
    pub source: String,
    pub name: String,
    pub size: usize,
    pub ext: String,
    pub mime: String,
    pub buffer: Vec<u8>, 
}

impl Icon {
    fn download(client: &impl Downloader, link: &ElementRef) -> Result<Icon> {
        let href = match link.value().attr("href") {
            Some(href) => href,
            None => return Err("no href attribute on link element".into()),
        };
        let mime = match link.value().attr("type") {
            Some(mime) => mime,
            None => "image/png",
        };
        let mut response = client.get(href)?;
        let mut icon_data: Vec<u8> = vec![];
        copy(&mut response, &mut icon_data)?;
        Ok(Icon{
            source: href.into(),
            name: Url::parse(href)?.host_str().unwrap_or("".into()).into(),
            ext: format!(".{0}", href.split(".").last().unwrap()).into(),
            size: icon_data.len(),
            mime: mime.into(),
            buffer: icon_data,
        })
    }
}

impl std::convert::AsRef<[u8]> for Icon {
    fn as_ref(&self) -> &[u8] {
        &self.buffer
    }
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Size {
    w: u32,
    h: u32,
}

impl Size {
    fn empty() -> Size {
        Size{
            w: 0,
            h: 0,
        }
    }
}

/// parse dimensions like "64x64".
impl std::str::FromStr for Size {
    type Err = Box<Error>;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("x").collect();
        if parts.len() < 2 {
            return Err(format!("invalid dimensions: {}", s).into());
        }
        Ok(Size{
            w: parts[0].parse()?,
            h: parts[1].parse()?,
        })
    }
}