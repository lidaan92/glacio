#[macro_use]
extern crate iron;
#[macro_use]
extern crate router;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

pub mod api;

#[derive(Debug)]
pub enum Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// A remote camera that transmits pictures back to home.
#[derive(Debug, Serialize)]
pub struct Camera;
