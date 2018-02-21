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
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate url;

#[macro_use]
mod macros;

pub mod atlas;
pub mod camera;
pub mod sutron;

pub use camera::{Camera, Image};
