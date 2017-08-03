#[macro_use]
extern crate iron;
#[macro_use]
extern crate log;
#[macro_use]
extern crate router;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

pub mod api;
mod camera;

pub use camera::Camera;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    TomlDe(toml::de::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Error {
        Error::TomlDe(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
