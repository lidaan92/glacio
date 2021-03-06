//! JSON API for glacio data.
//!
//! This crate uses the `glacio` crate to fetch glacier research data, and turns it into a JSON API
//! for the web.

#![deny(missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
        trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
        unused_qualifications)]

extern crate chrono;
extern crate glacio;
#[macro_use]
extern crate iron;
#[cfg(test)]
extern crate iron_test;
extern crate logger;
extern crate params;
extern crate percent_encoding;
#[macro_use]
extern crate router;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate toml;

pub mod atlas;
pub mod cameras;
pub mod paginate;

mod api;
mod config;
mod json;

pub use api::Api;
pub use config::Config;
pub use paginate::Paginate;

/// Our custom error enum.
#[derive(Debug)]
pub enum Error {
    /// Wrapper around `glacio::atlas::Error`.
    Atlas(glacio::atlas::Error),
    /// Wrapper around `glacio::camera::Error`.
    Camera(glacio::camera::Error),
    /// Invalid configuration.
    Config(String),
    /// Wrapper around `std::io::Error`.
    Io(std::io::Error),
    /// Wrapper around `std::num::ParseIntError`.
    ParseInt(std::num::ParseIntError),
    /// Wrapper around `toml::de::Error`.
    TomlDe(toml::de::Error),
}

/// Our custom result type.
pub type Result<T> = std::result::Result<T, Error>;

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

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Error {
        Error::TomlDe(err)
    }
}

impl From<glacio::atlas::Error> for Error {
    fn from(err: glacio::atlas::Error) -> Error {
        Error::Atlas(err)
    }
}

impl From<glacio::camera::Error> for Error {
    fn from(err: glacio::camera::Error) -> Error {
        Error::Camera(err)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Atlas(ref err) => err.description(),
            Error::Camera(ref err) => err.description(),
            Error::Config(_) => "api configuration error",
            Error::Io(ref err) => err.description(),
            Error::ParseInt(ref err) => err.description(),
            Error::TomlDe(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Atlas(ref err) => Some(err),
            Error::Camera(ref err) => Some(err),
            Error::Config(_) => None,
            Error::Io(ref err) => Some(err),
            Error::ParseInt(ref err) => Some(err),
            Error::TomlDe(ref err) => Some(err),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Atlas(ref err) => write!(f, "atlas error: {}", err),
            Error::Camera(ref err) => write!(f, "camera error: {}", err),
            Error::Config(ref msg) => write!(f, "api configuration error: {}", msg),
            Error::Io(ref err) => write!(f, "io error: {}", err),
            Error::ParseInt(ref err) => write!(f, "parse int error: {}", err),
            Error::TomlDe(ref err) => write!(f, "toml de error: {}", err),
        }
    }
}
