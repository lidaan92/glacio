extern crate chrono;
extern crate url;

mod camera;

pub use camera::{Camera, Image};

#[derive(Debug)]
pub enum Error {
    /// Wrapper around `chrono::ParseError`.
    ChronoParse(chrono::ParseError),
    /// Wrapper around `std::io::Error`.
    Io(std::io::Error),
    /// Wrapper around `std::path::StripPrefixError`.
    StripPrefix(std::path::StripPrefixError),
    /// Wrapper around `url::ParseError`.
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
            Error::UrlParse(ref err) => err.fmt(f),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
