use crate::error::{Error, ParseError};
use image;
use log::debug;
use reqwest;
use scraper::{Html, Selector};
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::convert::*;
use std::io::{copy, Read};
use std::result::Result as StdResult;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use url::Url;

pub type Result<T> = StdResult<T, Error>;

/// Infer an icon using the default Inferer.
pub fn infer_icon(url: &Url) -> Result<Icon> {
    Inferer::default().infer(&url.clone().into_string())
}

/// Infer an application name from a url.
///
/// Note: [jiahaog/nativefier](https://github.com/jiahaog/nativefier) infers the
/// name from the html <title> tag, whereas we just inspect the url's hostname.
///
/// This is quicker (no io), but doesn't allow for "pretty" titles (with capital
/// letters, whitespace, etc).
pub fn infer_name(url: &Url) -> Result<String> {
    let host = match url.host_str() {
        Some(host) => host,
        None => {
            return Err(Error::InferName {
                url: url.clone(),
                reason: "url does not include hostname".into(),
            })
        }
    };
    // If there is two dots eg www.example.com, take the middle part "example"
    // as the name.
    // If there is one dot eg soundcloud.com, take the first part "soundcloud"
    // as the name.
    match host.matches(".").count() {
        1 => Ok(host.split(".").nth(0).unwrap().into()),
        2 => Ok(host.split(".").nth(1).unwrap().into()),
        _ => Err(Error::InferName {
            url: url.clone(),
            reason: "url contains an uncommon hostname format".into(),
        }),
    }
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
where
    D: Downloader + Clone + Send + Sync + 'static,
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
                    Err(err) => {
                        debug!("downloading icon: {}", err);
                        None
                    }
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
        let links: Vec<String> = doc
            .select(&link_el)
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
                    return Err(Error::Scrape(
                        "'rel' attribute does not include 'icon'".into(),
                    ));
                }
                Ok(href.into())
            })
            .map(|r: Result<String>| match r {
                Ok(link) => link,
                Err(err) => {
                    debug!("malformed link: {}", err);
                    "".into()
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
                    }
                }
            })
            .filter(|link: &String| !link.is_empty())
            .collect();
        Ok(links)
    }
}

/// Downloader performs network requests.
/// The default Downloader uses reqwest crate.
pub trait Downloader {
    fn get(&self, url: &str) -> Result<Box<dyn Read>>;
}

impl Downloader for reqwest::Client {
    fn get(&self, url: &str) -> Result<Box<dyn Read>> {
        Ok(Box::new(reqwest::Client::get(self, url).send()?))
    }
}

/// Icon is icon detected for a website.
#[derive(Debug)]
pub struct Icon {
    /// Uri (typically url) with which this icon was loaded from.
    pub source: String,
    /// Name of the icon. May be empty.
    pub name: String,
    /// Extension for the given image type.
    pub ext: String,
    /// Container for the image data.
    pub img: image::RgbaImage,
}

impl Icon {
    /// Download the image at href and use it to create an icon.
    fn download(client: &impl Downloader, href: &str) -> Result<Icon> {
        let mut response = client.get(href)?;
        let mut icon_data: Vec<u8> = vec![];
        copy(&mut response, &mut icon_data)?;
        // FIXME: Fails on svg case, since image crate doesn't suppport svg.
        // TODO: Handle svg.
        let kind = image::guess_format(&icon_data)?;
        let ext = match kind {
            image::PNG => "png",
            image::ICO => "ico",
            image::JPEG => "jpeg",
            _ => "",
        };
        let img = image::load_from_memory(&icon_data)?;
        Ok(Icon {
            source: href.into(),
            name: Url::parse(href)?.host_str().unwrap_or_else(|| "").into(),
            img: img.to_rgba(),
            ext: ext.into(),
        })
    }
}

impl Eq for Icon {}

impl PartialOrd for Icon {
    fn partial_cmp(&self, other: &Icon) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Icon {
    fn cmp(&self, other: &Icon) -> Ordering {
        let left: Size = self.img.dimensions().into();
        let right: Size = other.img.dimensions().into();
        left.cmp(&right)
    }
}

impl PartialEq for Icon {
    fn eq(&self, other: &Icon) -> bool {
        self.name == other.name && self.img.dimensions() == other.img.dimensions()
    }
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone)]
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
        Ok(Size {
            w: parts[0].parse()?,
            h: parts[1].parse()?,
        })
    }
}

impl From<(u32, u32)> for Size {
    fn from(d: (u32, u32)) -> Self {
        Size { w: d.0, h: d.1 }
    }
}
