//! Remote LiDAR system located at the Helheim Glacier in Greenland.
//!
//! The ATLAS system, a remote LiDAR scanner at the Helheim Glacier in southeast Greenland, sends
//! back hourly heartbeat messages to keep us informed of its status. These heartbeats include
//! information such as the last LiDAR scan, the state of charge of the system batteries, and whether
//! the EFOY fuel cells are on, among other information.
//!
//! These messages have gone through three versions as of this writing:
//!
//! - Version 1 messages were transmitted from system install in July 2015 through system update in
//! August 2016. These messages were sent with IMEI 300234063909200.
//! - Version 2 messages were transmitted from system update August 2016 through system shutdown in
//! October 2016. These messages were sent with IMEI 300234063556840.
//! - Version 3 messages are being transmitted as of system reboot and update in July 2017, with
//! IMEI 300234063556840.
//!
//! As of this writing, this module supports only version 3 heartbeat messages.
//!
//! # Examples
//!
//! The bulk of the work is done by the `read_sbd` function, which returns an iterator over
//! `Result<Heartbeat, Error>`. To get all valid version 3 heartbeats from imei 300234063556840 in
//! the `data` directory:
//!
//! ```
//! use glacio::atlas::SbdSource;
//! let heartbeats = SbdSource::new("data")
//!     .imeis(&["300234063556840"])
//!     .versions(&[3])
//!     .iter()
//!     .unwrap()
//!     .filter_map(|result| result.ok())
//!     .collect::<Vec<_>>();
//! ```
//!
//! # Future work
//!
//! When we install ATLAS 2 on the north short of the glacier in the summer of 2018, we will
//! undoubtedly update the heartbeat format and use the same format for both systems. This module
//! will require an update to handle the new heartbeat version.

pub mod efoy;
mod heartbeat;

pub use self::efoy::Efoy;
pub use self::heartbeat::{Heartbeat, ReadSbd, SbdSource};
