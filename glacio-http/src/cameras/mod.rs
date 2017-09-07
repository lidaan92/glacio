//! Our fleet of remote cameras.
//!
//! This is the umbrella module for code that works with all of our remote cameras as a fleet,
//! hense the plural.

pub mod handlers;

mod camera;
mod config;
mod image;

pub use self::config::{CameraConfig, Config};
