//! Organize and present remote glacier and weather station data.
//!
//! We maintain multiple weather stations, remote LiDAR installations, and remote cameras. Data
//! from these systems is transmitted to local servers via satellite connections. These data are
//! housed in various locations:
//!
//! - Greg Hanlon's CWMS server
//! - On lidar.io:
//!     - In Iridium Short Burst Data (SBD) messages in `/var/iridium`
//!     - As images in `/home/iridiumcam/StarDot`
//!
//! This crate brings together these disparate data sources into a single Rust API. For now, we
//! don't have any of the weather station data, only cameras and ATLAS status information.

#![deny(missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
        trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
        unused_qualifications)]

extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate sbd;
extern crate url;

pub mod atlas;
pub mod camera;

mod sutron;
mod utils;

pub use camera::{Camera, Image};

/// Our custom error enum.
#[derive(Debug)]
pub enum Error {
    /// Wrapper around `chrono::ParseError`.
    ChronoParse(chrono::ParseError),
    /// An EFOY already has a cartridge of that name.
    DuplicateEfoyCartridge(String),
    /// The message was unable to be converted into a ATLAS heartbeat.
    Heartbeat(String),
    /// The image filename was not in the proper format.
    ImageFilename(String),
    /// Problem reconstructing a Sutron interleaved message.
    ///
    /// Interleaved messages are used to send long messages over (byte-limited) Iridium SBD.
    InterleavedMessage(String),
    /// Wrapper around `std::io::Error`.
    Io(std::io::Error),
    /// Wrapper around `std::num::ParseFloatError`.
    ParseFloat(std::num::ParseFloatError),
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

impl From<std::num::ParseFloatError> for Error {
    fn from(err: std::num::ParseFloatError) -> Error {
        Error::ParseFloat(err)
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
            Error::DuplicateEfoyCartridge(_) => "duplicate efoy cartridge",
            Error::Heartbeat(_) => "error parsing heartbeat",
            Error::ImageFilename(_) => "invalid image filename",
            Error::InterleavedMessage(_) => "problem reconstructing an interleaved message",
            Error::Io(ref err) => err.description(),
            Error::ParseFloat(ref err) => err.description(),
            Error::ParseInt(ref err) => err.description(),
            Error::Sbd(ref err) => err.description(),
            Error::StripPrefix(ref err) => err.description(),
            Error::UrlParse(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::ChronoParse(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
            Error::ParseFloat(ref err) => Some(err),
            Error::ParseInt(ref err) => Some(err),
            Error::Sbd(ref err) => Some(err),
            Error::StripPrefix(ref err) => Some(err),
            Error::UrlParse(ref err) => Some(err),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::ChronoParse(ref err) => err.fmt(f),
            Error::DuplicateEfoyCartridge(ref name) => {
                write!(f, "duplicate efoy cartridge: {}", name)
            }
            Error::Heartbeat(ref msg) => write!(f, "error parsing heartbeat: {}", msg),
            Error::ImageFilename(ref msg) => write!(f, "invalid image filename: {}", msg),
            Error::InterleavedMessage(ref msg) => write!(f, "interleaved message error: {}", msg),
            Error::Io(ref err) => err.fmt(f),
            Error::ParseFloat(ref err) => err.fmt(f),
            Error::ParseInt(ref err) => err.fmt(f),
            Error::Sbd(ref err) => err.fmt(f),
            Error::StripPrefix(ref err) => err.fmt(f),
            Error::UrlParse(ref err) => err.fmt(f),
        }
    }
}

/// Our custom result type.
pub type Result<T> = std::result::Result<T, Error>;
