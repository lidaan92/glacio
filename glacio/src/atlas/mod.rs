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

pub mod battery;
pub mod efoy;
pub mod scanner;

mod heartbeat;

pub use self::efoy::Efoy;
pub use self::heartbeat::{Heartbeat, ReadSbd, SbdSource};
use chrono::ParseError;
use sbd;
use std::{error, result};
use std::fmt::{self, Display, Formatter};
use std::num::{ParseFloatError, ParseIntError};
use sutron;

/// A custom error enum for ATLAS issues.
#[derive(Debug)]
pub enum Error {
    /// The efoy cartridge name is invalid.
    CartridgeName(String),
    /// Wrapper around `chrono::ParseError`.
    ChronoParse(ParseError),
    /// The efoy cartridge name is already present in the efoy.
    DuplicateEfoyCartridge(String),
    /// The efoy cartridge is already empty, it can't be emptied again.
    EmptyCartridge(String),
    /// The efoy heartbeat message is in an invalid format.
    EfoyHeartbeatFormat(String),
    /// The format of the heartbeat message could not be recognized.
    HeartbeatFormat(String),
    /// Wrapper around `std::num::ParseFloatError`.
    ParseFloat(ParseFloatError),
    /// Wrapper around `std::num::ParseIntError`.
    ParseInt(ParseIntError),
    /// Wrapper around `sbd::Error`.
    Sbd(sbd::Error),
    /// The scanner power on text is invalid.
    ScannerPowerOnFormat(String),
    /// The stop scan text is invalid.
    StopScanFormat(String),
    /// Wrapper around `glacio::sutron::message::Error`.
    SutronMessage(sutron::message::Error),
    /// The efoy state, as reported, is not recognized.
    UnknownEfoyState(String),
}

/// A custom result type for ATLAS.
pub type Result<T> = result::Result<T, Error>;

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Error {
        Error::ParseInt(err)
    }
}

impl From<ParseFloatError> for Error {
    fn from(err: ParseFloatError) -> Error {
        Error::ParseFloat(err)
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Error {
        Error::ChronoParse(err)
    }
}

impl From<sbd::Error> for Error {
    fn from(err: sbd::Error) -> Error {
        Error::Sbd(err)
    }
}

impl From<sutron::message::Error> for Error {
    fn from(err: sutron::message::Error) -> Error {
        Error::SutronMessage(err)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::CartridgeName(_) => "invalid EFOY cartridge name",
            Error::ChronoParse(ref err) => err.description(),
            Error::DuplicateEfoyCartridge(_) => {
                "a cartridge with this name has already been added to this efoy"
            }
            Error::EmptyCartridge(_) => "the cartridge is already empty, cannot empty it again",
            Error::EfoyHeartbeatFormat(_) => "the format of this efoy heartbeat message is invalid",
            Error::HeartbeatFormat(_) => "the format of this heartbeat message is invalid",
            Error::ParseFloat(ref err) => err.description(),
            Error::ParseInt(ref err) => err.description(),
            Error::Sbd(ref err) => err.description(),
            Error::ScannerPowerOnFormat(_) => {
                "the format of the scanner power on message is invalid"
            }
            Error::StopScanFormat(_) => "the format of the stop scan message is invalid",
            Error::SutronMessage(ref err) => err.description(),
            Error::UnknownEfoyState(_) => "the efoy state string is not recognized",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::ChronoParse(ref err) => Some(err),
            Error::ParseFloat(ref err) => Some(err),
            Error::ParseInt(ref err) => Some(err),
            Error::Sbd(ref err) => Some(err),
            Error::SutronMessage(ref err) => Some(err),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Error::CartridgeName(ref name) => write!(f, "invalid EFOY cartridge name: {}", name),
            Error::ChronoParse(ref err) => err.fmt(f),
            Error::DuplicateEfoyCartridge(ref name) => {
                write!(
                    f,
                    "a cartridge with name {} has already been added to this efoy",
                    name
                )
            }
            Error::EmptyCartridge(ref name) => {
                write!(
                    f,
                    "efoy cartridge {} is empty, cannot be emptied again",
                    name
                )
            }
            Error::EfoyHeartbeatFormat(ref s) => write!(f, "invalid efoy heartbeat format: {}", s),
            Error::HeartbeatFormat(ref s) => write!(f, "invalid heartbeat format: {}", s),
            Error::ParseFloat(ref err) => err.fmt(f),
            Error::ParseInt(ref err) => err.fmt(f),
            Error::Sbd(ref err) => err.fmt(f),
            Error::ScannerPowerOnFormat(ref s) => {
                write!(f, "invalid scanner power on format: {}", s)
            }
            Error::StopScanFormat(ref s) => write!(f, "invalid stop scan format: {}", s),
            Error::SutronMessage(ref err) => err.fmt(f),
            Error::UnknownEfoyState(ref state) => write!(f, "efoy state {} not recognized", state),
        }
    }
}
