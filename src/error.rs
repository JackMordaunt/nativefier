use std::error::Error as StdError;
use image;
use url;
use reqwest;

#[derive(Debug)]
pub enum Error {
    /// Parsing failures for various primitives. 
    Parse(ParseError),
    /// Download and IO errors.  
    /// Wraps a trait object because we don't know what concrete error the
    /// implementor will use. 
    Download(Box<dyn StdError + Sync + Send>),
    /// Image decoding and processing errors. 
    Image(image::ImageError),
    /// Scraping markup for icons. 
    Scrape(String),
    // InferName captures errors that occur while trying to infer app name from 
    // a url. 
    InferName { url: url::Url, reason: String },
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
            Error::Download(err) => write!(f, "downloading: {}", err),
            Error::Image(err) => write!(f, "image: {}", err),
            Error::Scrape(s) => write!(f, "scraping: {}", s),
            Error::InferName {url, reason} => write!(f, "inferring name for {}: {}", url, reason),
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

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Parse(err) => Some(err),
            Error::Download(err) => Some(err.as_ref()),
            Error::Image(err) => Some(err),
            _ => None,
        }
    }
}

impl StdError for ParseError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ParseError::Int(err) => Some(err),
            ParseError::Url(err) => Some(err),
            ParseError::Size(_) => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Download(Box::new(err))
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::Parse(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Download(Box::new(err))
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