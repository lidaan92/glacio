use {Error, Result};
use atlas::efoy;
use chrono::{DateTime, Utc};
use regex::Regex;
use sbd::mo::Message;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::vec::IntoIter;

/// Status report from the entire ATLAS system.
///
/// These heartbeats are transmitted via Iridium SBD. Because of the SBD message length
/// restriction, heartbeats may come in one or more messages, and might have to be pieced together.
#[derive(Clone, Debug, PartialOrd)]
pub struct Heartbeat {
    /// The version of heartbeat message.
    pub version: u8,
    /// The date and time of the *first* heartbeat sbd message.
    pub datetime: DateTime<Utc>,
    /// The state of charge of the first battery.
    pub soc1: f32,
    /// The state of charge of the second battery.
    pub soc2: f32,
    /// Information about efoy system 1.
    pub efoy1: efoy::Heartbeat,
    /// Information about efoy system 2.
    pub efoy2: efoy::Heartbeat,
    /// Are the Riegl systems enabled?
    ///
    /// There's a hardware switch that disables the housing and scanner. The switch is controlled
    /// by the data logger, which flips the switch when the state of charges get too low.
    pub are_riegl_systems_on: bool,
}

/// Structure for retrieving ATLAS heartbeats from SBD messages.
///
/// Configure the source to fetch heartbeats of one or more versions from a filesystem sbd storage.
#[derive(Debug)]
pub struct SbdSource {
    path: PathBuf,
    imeis: Vec<String>,
    versions: Vec<u8>,
}

/// An iterator over heartbeats provided by an `SbdSource`.
///
/// The iterator type is a `Result<Heartbeat>`, because we can fail in the middle of a stream of
/// heartbeats.
#[derive(Debug)]
pub struct ReadSbd {
    iter: IntoIter<Message>,
    versions: Vec<u8>,
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
                                                (?P<efoy1>.*)\r\n # efoy1
                                                (?P<efoy2>.*)\r\n # efoy2
                                                (?P<riegl_switch>.*) # riegl switch
                                                \z").unwrap();
        }
        if let Some(captures) = RE.captures(message) {
            Ok(Heartbeat {
                   version: captures.name("version")
                       .unwrap()
                       .as_str()
                       .parse()?,
                   datetime: datetime,
                   soc1: captures.name("soc1")
                       .unwrap()
                       .as_str()
                       .parse()?,
                   soc2: captures.name("soc2")
                       .unwrap()
                       .as_str()
                       .parse()?,
                   efoy1: captures.name("efoy1")
                       .unwrap()
                       .as_str()
                       .parse()?,
                   efoy2: captures.name("efoy2")
                       .unwrap()
                       .as_str()
                       .parse()?,
                   are_riegl_systems_on: captures.name("riegl_switch").unwrap().as_str() == "on",
               })
        } else {
            Err(Error::Heartbeat("Unable to parse heartbeat".to_string()))
        }
    }
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
        use sbd::storage::{FilesystemStorage, Storage};
        let storage = FilesystemStorage::open(&self.path)?;
        let mut messages = Vec::new();
        if self.imeis.is_empty() {
            messages = storage.messages()?;
        } else {
            for imei in &self.imeis {
                messages.extend(storage.messages_from_imei(imei)?);
            }
        }
        messages.sort_by(|a, b| a.time_of_session().cmp(&b.time_of_session()));
        Ok(ReadSbd {
               iter: messages.into_iter(),
               versions: self.versions.clone(),
           })
    }
}

impl Iterator for ReadSbd {
    type Item = Result<Heartbeat>;

    fn next(&mut self) -> Option<Self::Item> {
        use sutron::Message;
        let mut message = Message::new();
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
                                    message = Message::new();
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn heartbeats() {
        let read_sbd = SbdSource::new("data").iter().unwrap();
        let heartbeats = read_sbd.collect::<Vec<Result<Heartbeat>>>();
        assert_eq!(3, heartbeats.len());
    }

    #[test]
    fn heartbeat_parsing() {
        let read_sbd = SbdSource::new("data").iter().unwrap();
        let heartbeat = read_sbd.skip(1)
            .next()
            .unwrap()
            .unwrap();
        assert_eq!(3, heartbeat.version);
        assert_eq!(Utc.ymd(2017, 8, 1).and_hms(0, 0, 55), heartbeat.datetime);
        assert_eq!(94.208, heartbeat.soc1);
        assert_eq!(94.947, heartbeat.soc2);
        assert!(heartbeat.are_riegl_systems_on);

        let efoy1 = heartbeat.efoy1;
        assert_eq!(efoy::State::AutoOff, efoy1.state);
        assert_eq!("1.1", efoy1.cartridge);
        assert_eq!(3.741, efoy1.consumed);
        assert_eq!(26.63, efoy1.voltage);
        assert_eq!(-0.03, efoy1.current);

        let efoy2 = heartbeat.efoy2;
        assert_eq!(efoy::State::AutoOff, efoy2.state);
        assert_eq!("1.1", efoy2.cartridge);
        assert_eq!(3.687, efoy2.consumed);
        assert_eq!(26.64, efoy2.voltage);
        assert_eq!(-0.02, efoy2.current);
    }
}
