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
use mime_sniffer::MimeTypeSniffer;
use image::{self, GenericImageView};

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

// TODO(jfm) "better icon detection":
// 1. [x] Search for any link that contains "icon" in it's "rel" attribute.
// 2. [x] Download all and compare sizes. 
// 3. [ ] Compare dimensions instead of buffer size.
// 4. [ ] Download icons concurrently. 
impl<D: Downloader> Inferer<D> {
    fn infer(&self, url: &str) -> Result<Icon> {
        let mut body = self.client.get(url)?;
        let mut buf = String::new();
        body.read_to_string(&mut buf)?;
        let doc = Html::parse_document(&buf);
        let link_el = Selector::parse("link").unwrap();
        let mut icons: Vec<Icon> = doc.select(&link_el)
            .map(|el: ElementRef| {
                let el = el.value();
                let rel = match el.attr("rel") {
                    Some(rel) => rel,
                    None => return Err("no rel attribute on link element".into()),
                };
                let href = match el.attr("href") {
                    Some(href) => href,
                    None => return Err("no href attribute on link element".into()),
                };
                if !rel.contains("icon") {
                    return Err("link[rel] does not include 'icon'".into());
                }
                Icon::download(&self.client, &href)
            })
            .filter_map(|icon| {
                icon.ok()
            })
            .collect();
        icons.sort();
        match icons.into_iter().last() {
            Some(icon) => Ok(icon),
            None => Err("no icons found".into()),
        }
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

use std::cmp::{Ordering, Ord, PartialOrd, PartialEq};

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
    type Err = Box<Error>;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('x').collect();
        if parts.len() < 2 {
            return Err(format!("invalid dimensions: {}", s).into());
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