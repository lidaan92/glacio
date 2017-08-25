extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate sbd;
extern crate url;

#[deny(missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
       trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
       unused_qualifications)]

pub mod camera;
pub mod heartbeat;

pub use camera::{Camera, Image};
pub use heartbeat::Heartbeat;

#[derive(Debug)]
pub enum Error {
    /// Wrapper around `chrono::ParseError`.
    ChronoParse(chrono::ParseError),
    /// Invalid image filename.
    ImageFilename(String),
    /// Problem reconstructing an interleaved message.
    InterleavedMessage(String),
    /// Wrapper around `std::io::Error`.
    Io(std::io::Error),
    /// Wrapper around `std::num::ParseIntError`.
    ParseInt(std::num::ParseIntError),
    /// Wrapper around `sbd::Error`.
    Sbd(sbd::Error),
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

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Error {
        Error::ParseInt(err)
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

impl From<sbd::Error> for Error {
    fn from(err: sbd::Error) -> Error {
        Error::Sbd(err)
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
            Error::ImageFilename(_) => "invalid image filename",
            Error::InterleavedMessage(_) => "problem reconstructing an interleaved message",
            Error::Io(ref err) => err.description(),
            Error::ParseInt(ref err) => err.description(),
            Error::Sbd(ref err) => err.description(),
            Error::StripPrefix(ref err) => err.description(),
            Error::UrlParse(ref err) => err.description(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::ChronoParse(ref err) => err.fmt(f),
            Error::ImageFilename(ref msg) => write!(f, "invalid image filename: {}", msg),
            Error::InterleavedMessage(ref msg) => write!(f, "interleaved message error: {}", msg),
            Error::Io(ref err) => err.fmt(f),
            Error::ParseInt(ref err) => err.fmt(f),
            Error::Sbd(ref err) => err.fmt(f),
            Error::StripPrefix(ref err) => err.fmt(f),
            Error::UrlParse(ref err) => err.fmt(f),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
