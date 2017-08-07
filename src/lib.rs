extern crate chrono;
#[macro_use]
extern crate iron;
extern crate persistent;
#[macro_use]
extern crate router;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate url;

pub mod api;
mod camera;

pub use camera::{Camera, Image};

#[derive(Debug)]
pub enum Error {
    ChronoParse(chrono::ParseError),
    Io(std::io::Error),
    StripPrefix(std::path::StripPrefixError),
    TomlDe(toml::de::Error),
    UrlParse(url::ParseError),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<std::path::StripPrefixError> for Error {
    fn from(err: std::path::StripPrefixError) -> Error {
        Error::StripPrefix(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Error {
        Error::TomlDe(err)
    }
}

impl From<chrono::ParseError> for Error {
    fn from(err: chrono::ParseError) -> Error {
        Error::ChronoParse(err)
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Error {
        Error::UrlParse(err)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ChronoParse(ref err) => err.description(),
            Error::Io(ref err) => err.description(),
            Error::StripPrefix(ref err) => err.description(),
            Error::TomlDe(ref err) => err.description(),
            Error::UrlParse(ref err) => err.description(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::ChronoParse(ref err) => err.fmt(f),
            Error::Io(ref err) => err.fmt(f),
            Error::StripPrefix(ref err) => err.fmt(f),
            Error::TomlDe(ref err) => err.fmt(f),
            Error::UrlParse(ref err) => err.fmt(f),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
