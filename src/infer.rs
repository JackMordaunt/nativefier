use std::convert::*;
use std::thread;
use std::sync::mpsc::channel;
use std::io::{copy, Read};
use std::cmp::{Ordering, Ord, PartialOrd, PartialEq};
use std::sync::Arc;
use std::result::Result as StdResult;
use scraper::{Html, Selector};
use image::{self, GenericImageView};
use reqwest;
use url::Url;
use log::debug;
use crate::error::{Error, ParseError};

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
impl<D> Inferer<D>
    where D: Downloader + Clone + Send + Sync + 'static
{
    fn infer(&self, url: &str) -> Result<Icon> {
        let (tx, tr) = channel();
        let client = Arc::new(self.client.clone());
        let mut workers = vec![];
        for link in self.scrape(url)? {
            let client = client.clone();
            let tx = tx.clone();
            workers.push(thread::spawn(move || {
                let icon = match Icon::download(client.as_ref(), &link) {
                    Ok(icon) => Some(icon),
                    Err(err) => {debug!("downloading icon: {}", err); None},
                };
                tx.send(icon).expect("sending icon over channel");
            }));
        }
        let mut icons = vec![];
        for _ in workers {
            if let Some(icon) = tr.recv().expect("receiving icon from channel") {
                icons.push(icon);
            }
        }
        icons.sort();
        match icons.into_iter().last() {
            Some(icon) => Ok(icon),
            None => Err(Error::Scrape("no icons found".into())),
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
        let base = Url::parse(url)?;        
        let links: Vec<String> = doc.select(&link_el)
            .map(|el| {
                let el = el.value();
                let rel = match el.attr("rel") {
                    Some(rel) => rel,
                    None => return Err(Error::Scrape("no rel attribute on link element".into())),
                };
                let href = match el.attr("href") {
                    Some(href) => href,
                    None => return Err(Error::Scrape("no href attribute on link element".into())),
                };
                if !rel.contains("icon") {
                    return Err(Error::Scrape("'rel' attribute does not include 'icon'".into()));
                }
                Ok(href.into())
            })
            .map(|r: Result<String>| {
                match r {
                    Ok(link) => link,
                    Err(err) => {
                        debug!("malformed link: {}", err);
                        "".into()
                    },
                }
            })
            .map(|link: String| {
                if link.contains("http") {
                   return link; 
                }
                match base.join(&link) {
                    Ok(url) => url.into_string(),
                    Err(err) => {
                        debug!("joining {} to {}: {}", &link, &base, err);
                        "".into()
                    },
                }
            })
            .filter(|link: &String| {
                !link.is_empty()
            })
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
        Ok(Box::new(reqwest::Client::get(self, url).send()?))
    }
}

/// Icon is icon detected for a website. 
#[derive(Eq, Debug)]
pub struct Icon {
    pub source: String,
    pub name: String,
    pub size: usize,
    pub ext: String,
    pub buffer: Vec<u8>, 
    pub dimensions: Size,
}

impl Icon {
    fn download(client: &impl Downloader, href: &str) -> Result<Icon> {
        let mut response = client.get(href)?;
        let mut icon_data: Vec<u8> = vec![];
        copy(&mut response, &mut icon_data)?;
        let img = image::load_from_memory(&icon_data)?;
        Ok(Icon{
            source: href.into(),
            name: Url::parse(href)?.host_str().unwrap_or_else(|| "").into(),
            // Assumes the url ends with a valid file extension.
            ext: format!(".{0}", href.split('.').last().unwrap()),
            size: icon_data.len(),
            dimensions: img.dimensions().into(),
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
    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
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

pub type Result<T> = StdResult<T, Error>;