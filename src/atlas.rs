//! Heartbeat messages from the ATLAS system.
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
//! `Result<Heartbeat, Error>`. To get all version 3 heartbeats:
//!
//! ```
//! # use glacio::atlas;
//! let heartbeats = atlas::read_sbd("data", "300234063556840")
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

use {Error, Result};
use chrono::{DateTime, Utc};
use convert::TryInto;
use regex::Regex;
use sbd::mo::Message;
use sbd::storage::{FilesystemStorage, Storage};
use std::cmp::Ordering;
use std::path::Path;
use std::vec::IntoIter;
use sutron::InterleavedMessage;

/// Returns a `ReadSbd`, which can iterate over the heartbeats in an sbd storage.
///
/// # Examples
///
/// ```
/// # use glacio::atlas;
/// for result in atlas::read_sbd("data", "300234063556840").unwrap() {
///     let heartbeat = result.unwrap();
///     println!("{:?}", heartbeat);
/// }
/// ```
pub fn read_sbd<P: AsRef<Path>>(path: P, imei: &str) -> Result<ReadSbd> {
    FilesystemStorage::open(path)
        .and_then(|storage| storage.messages_from_imei(imei))
        .map(|messages| ReadSbd { iter: messages.into_iter() })
        .map_err(Error::from)
}

/// An iterator over heartbeats in an sbd storage.
///
/// The iterator type is a `Result<Heartbeat>`, because we can fail in the middle of a stream of
/// heartbeats.
#[derive(Debug)]
pub struct ReadSbd {
    iter: IntoIter<Message>,
}

/// A heartbeat from the ATLAS system.
///
/// These heartbeats are transmitted via Iridium SBD. Because of the SBD message length
/// restriction, heartbeats may come in one or more messages, and might have to be pieced together.
#[derive(Clone, Copy, Debug, PartialOrd)]
pub struct Heartbeat {
    /// The date and time of the *first* heartbeat sbd message.
    pub datetime: DateTime<Utc>,
    /// The state of charge of the first battery.
    pub soc1: f32,
    /// The state of charge of the second battery.
    pub soc2: f32,
}

impl Iterator for ReadSbd {
    type Item = Result<Heartbeat>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut message = InterleavedMessage::new();
        while let Some(sbd_message) = self.iter.next() {
            match message.add(&sbd_message) {
                Ok(()) => if message.is_complete() {
                    return Some(message.try_into());
                },
                Err(err) => return Some(Err(err)),
            }
        }
        None
    }
}

impl PartialEq for Heartbeat {
    fn eq(&self, other: &Heartbeat) -> bool {
        self.datetime == other.datetime
    }
}

impl Eq for Heartbeat {}

impl Ord for Heartbeat {
    fn cmp(&self, other: &Heartbeat) -> Ordering {
        self.datetime.cmp(&other.datetime)
    }
}

impl TryInto<Heartbeat> for InterleavedMessage {
    type Err = Error;

    fn try_into(self) -> Result<Heartbeat> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?x)^ATHB(?P<version>\d{2})(?P<bytes>\d+)\r\n
                                                .*\r\n # scanner on
                                                .*\r\n # external temp, pressure, rh
                                                .*\r\n # scan start
                                                .*\r\n # scan stop
                                                .*\r\n # scan skip
                                                .*,(?P<soc1>\d+\.\d+),(?P<soc2>\d+\.\d+)\r\n
                                                .*\r\n # efoy1
                                                .*\r\n # efoy2
                                                .* # riegl switch
                                                \z").unwrap();
        }
        if let Some(captures) = RE.captures(&self.data) {
            Ok(Heartbeat {
                   datetime: self.datetime.unwrap(),
                   soc1: captures.name("soc1")
                       .unwrap()
                       .parse()?,
                   soc2: captures.name("soc2")
                       .unwrap()
                       .parse()?,
               })
        } else {
            Err(Error::Heartbeat("Unable to parse heartbeat".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heartbeats() {
        let read_sbd = read_sbd("data", "300234063556840").unwrap();
        let heartbeats = read_sbd.collect::<Vec<Result<Heartbeat>>>();
        assert_eq!(2, heartbeats.len());
        assert!(heartbeats.iter().all(|result| result.is_ok()));
    }

    #[test]
    fn heartbeat_parsing() {
        let mut read_sbd = read_sbd("data", "300234063556840").unwrap();
        let heartbeat = read_sbd.next().unwrap().unwrap();
        assert_eq!(94.208, heartbeat.soc1);
        assert_eq!(94.947, heartbeat.soc2);
    }
}
