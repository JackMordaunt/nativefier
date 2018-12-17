use std::{
    convert::*,
    error::Error,
    io::{
        copy,
    },
};
use scraper::{Html, Selector};
use reqwest;

pub type Result<T> = std::result::Result<T, Box<Error>>;

pub fn infer_icon(url: &str) -> Result<Icon> {
    let body = reqwest::get(url)?.text()?;
    let doc = Html::parse_document(&body);
    let apple_touch = Selector::parse("link[rel=\"apple-touch-icon\"]").unwrap();
    for link in doc.select(&apple_touch) {
        let href = match link.value().attr("href") {
            Some(href) => href,
            None => return Err("no href attribute on link element".into()),
        };
        let mut response = reqwest::get(href)?;
        let mut icon_data: Vec<u8> = vec![];
        copy(&mut response, &mut icon_data)?;
        return Ok(Icon{
            source: href.into(),
            name: "not implemented yet".into(),
            ext: ".png".into(),
            mime: "not implemented yet".into(),
            buffer: icon_data,
        });
    }
    Ok(Icon::empty())
}

/// Icon is the highest resolution icon detected for a website. 
pub struct Icon {
    pub source: String,
    pub name: String,
    pub ext: String,
    pub mime: String,
    pub buffer: Vec<u8>, 
}

impl Icon {
    fn empty() -> Icon {
        Icon {
            source: "".into(),
            name: "".into(),
            ext: "".into(),
            mime: "".into(),
            buffer: Vec::new(),
        }
    }
}

impl std::convert::AsRef<[u8]> for Icon {
    fn as_ref(&self) -> &[u8] {
        &self.buffer
    }
}