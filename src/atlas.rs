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

use {Error, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use sbd::mo::Message;
use sbd::storage::FilesystemStorage;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::vec::IntoIter;
use sutron;

/// ATLAS heartbeats from SBD messages.
///
/// Configure the source to fetch heartbeats of one or more versions from a filesystem sbd storage.
#[derive(Debug)]
pub struct SbdSource {
    path: PathBuf,
    imeis: Vec<String>,
    versions: Vec<u8>,
}

/// An iterator over heartbeats in an sbd storage.
///
/// The iterator type is a `Result<Heartbeat>`, because we can fail in the middle of a stream of
/// heartbeats.
#[derive(Debug)]
pub struct ReadSbd {
    iter: IntoIter<Message>,
    versions: Vec<u8>,
}

/// A heartbeat from the ATLAS system.
///
/// These heartbeats are transmitted via Iridium SBD. Because of the SBD message length
/// restriction, heartbeats may come in one or more messages, and might have to be pieced together.
#[derive(Clone, Copy, Debug, PartialOrd)]
pub struct Heartbeat {
    /// The version of heartbeat message.
    pub version: u8,
    /// The date and time of the *first* heartbeat sbd message.
    pub datetime: DateTime<Utc>,
    /// The state of charge of the first battery.
    pub soc1: f32,
    /// The state of charge of the second battery.
    pub soc2: f32,
}

impl SbdSource {
    /// Creates a new source for the provided local filesystem path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::atlas::SbdSource;
    /// let source = SbdSource::new("data");
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> SbdSource {
        SbdSource {
            path: path.as_ref().to_path_buf(),
            imeis: Vec::new(),
            versions: Vec::new(),
        }
    }

    /// Sets (or clears) the imei numbers to be used as heartbeat sources.
    ///
    /// If the slice is empty, this clears the imei filter and all imeis will be used.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::atlas::SbdSource;
    /// let source = SbdSource::new("data").imeis(&["300234063556840"]);
    /// ```
    pub fn imeis(mut self, imeis: &[&str]) -> SbdSource {
        self.imeis = imeis.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Sets (or clears) the heartbeat versions to be returned.
    ///
    /// If the slice is empty, clears the versions filter.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::atlas::SbdSource;
    /// let source = SbdSource::new("data").versions(&[3]);
    pub fn versions(mut self, versions: &[u8]) -> SbdSource {
        self.versions = versions.to_vec();
        self
    }

    /// Returns an iterator over the heartbeats in this source.
    ///
    /// Returns an error if the underlying storage can't be opened.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::atlas::SbdSource;
    /// let source = SbdSource::new("data");
    /// for heartbeat in source.iter().unwrap() {
    ///     println!("{:?}", heartbeat);
    /// }
    pub fn iter(&self) -> Result<ReadSbd> {
        use sbd::storage::Storage;
        let storage = FilesystemStorage::open(&self.path)?;
        let mut messages = Vec::new();
        if self.imeis.is_empty() {
            messages = storage.messages()?;
        } else {
            for imei in &self.imeis {
                messages.extend(storage.messages_from_imei(imei)?);
            }
        }
        Ok(ReadSbd {
               iter: messages.into_iter(),
               versions: self.versions.clone(),
           })
    }
}

impl Iterator for ReadSbd {
    type Item = Result<Heartbeat>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut message = sutron::Message::new();
        let mut datetime = None;
        while let Some(sbd_message) = self.iter.next() {
            if datetime.is_none() {
                datetime = Some(sbd_message.time_of_session());
            }
            match message.push(sbd_message.payload_str().unwrap()) {
                Ok(new_message) => {
                    if new_message.is_complete() {
                        match Heartbeat::new(&String::from(new_message), datetime.unwrap()) {
                            Ok(heartbeat) => {
                                if self.versions.is_empty() ||
                                   self.versions.contains(&heartbeat.version) {
                                    return Some(Ok(heartbeat));
                                } else {
                                    message = sutron::Message::new();
                                }
                            }
                            Err(err) => return Some(Err(err)),
                        }
                    } else {
                        message = new_message;
                    }
                }
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

impl Heartbeat {
    fn new(message: &str, datetime: DateTime<Utc>) -> Result<Heartbeat> {
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
        if let Some(captures) = RE.captures(message) {
            Ok(Heartbeat {
                   version: captures.name("version")
                       .unwrap()
                       .parse()?,
                   datetime: datetime,
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
        let read_sbd = SbdSource::new("data").iter().unwrap();
        let heartbeats = read_sbd.collect::<Vec<Result<Heartbeat>>>();
        assert_eq!(2, heartbeats.len());
        assert!(heartbeats.iter().all(|result| result.is_ok()));
    }

    #[test]
    fn heartbeat_parsing() {
        let mut read_sbd = SbdSource::new("data").iter().unwrap();
        let heartbeat = read_sbd.next().unwrap().unwrap();
        assert_eq!(94.208, heartbeat.soc1);
        assert_eq!(94.947, heartbeat.soc2);
    }
}
