#[macro_use]
extern crate iron;
#[macro_use]
extern crate router;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

pub mod api;

/// A remote camera that transmits pictures back to home.
#[derive(Debug, Serialize)]
pub struct Camera;
