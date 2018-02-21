//! The VZ-6000 inside of the housing.
//!
//! The scanner is powered on by the housing, then takes over control of the housing/scanner system
//! via RiSCRIPT. The scanner logs messages via HTTP on the data logger, and these messages are
//! used to populate the scanner information in the heartbeat messages.

use atlas::{Error, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use std::str::FromStr;

lazy_static! {
    static ref SCANNER_POWER_ON_REGEX: Regex = Regex::new(r"(?x)^
        (?P<datetime>.*),
        (?P<voltage>.*),
        (?P<temperature>.*),
        (?P<memory_external>.*),
        (?P<memory_internal>.*)
        $").unwrap();

    static ref SCAN_STOP_REGEX: Regex = Regex::new(r"(?x)^
        (?P<datetime>.*),
        (?P<num_points>.*),
        (?P<range_min>.*),
        (?P<range_max>.*),
        (?P<file_size>.*),
        (?P<amplitude_min>.*),
        (?P<amplitude_max>.*),
        (?P<roll>.*),
        (?P<pitch>.*)
        $").unwrap();
}

/// Data provided when the scanner powers on.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize)]
pub struct ScannerPowerOn {
    /// The date and time the scanner was powered on.
    pub datetime: DateTime<Utc>,
    /// The scanner voltage, in volts.
    pub voltage: f32,
    /// The scanner internal temperature, in °C.
    pub temperature: f32,
    /// The external memory available, in kB.
    pub memory_external: f64,
    /// The internal memory available, in kB.
    pub memory_internal: f64,
}

/// A log of the end of a scan.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize)]
pub struct ScanStop {
    /// The date and time the scan stopped.
    pub datetime: DateTime<Utc>,
    /// The number of points in the scan.
    pub num_points: usize,
    /// The minimum range of points in the scan.
    ///
    /// Since this scan hasn't been MTA processed, this is probably not a good value.
    pub range_min: f64,
    /// The maximum range of points in the scan.
    ///
    /// Since this scan hasn't been MTA processed, this is probably not a good value.
    pub range_max: f64,
    /// The size of the file, in bytes.
    pub file_size: f64,
    /// The minimum amplitude of the points.
    pub amplitude_min: usize,
    /// The maximum amplitude of the points.
    pub amplitude_max: usize,
    /// The roll of the scanner.
    pub roll: f32,
    /// The pitch of the scanner.
    pub pitch: f32,
}

impl FromStr for ScannerPowerOn {
    type Err = Error;
    fn from_str(s: &str) -> Result<ScannerPowerOn> {
        use sutron;

        if let Some(ref captures) = SCANNER_POWER_ON_REGEX.captures(s) {
            Ok(ScannerPowerOn {
                datetime: sutron::parse_datetime::<Error>(
                    captures.name("datetime").unwrap().as_str(),
                )?,
                voltage: parse_name_from_captures!(captures, "voltage"),
                temperature: parse_name_from_captures!(captures, "temperature"),
                memory_internal: parse_name_from_captures!(captures, "memory_internal"),
                memory_external: parse_name_from_captures!(captures, "memory_external"),
            })
        } else {
            Err(Error::ScannerPowerOnFormat(s.to_string()))
        }
    }
}

impl FromStr for ScanStop {
    type Err = Error;
    fn from_str(s: &str) -> Result<ScanStop> {
        use sutron;

        if let Some(ref captures) = SCAN_STOP_REGEX.captures(s) {
            Ok(ScanStop {
                datetime: sutron::parse_datetime::<Error>(
                    captures.name("datetime").unwrap().as_str(),
                )?,
                num_points: parse_name_from_captures!(captures, "num_points"),
                range_min: parse_name_from_captures!(captures, "range_min"),
                range_max: parse_name_from_captures!(captures, "range_max"),
                file_size: parse_name_from_captures!(captures, "file_size"),
                amplitude_min: parse_name_from_captures!(captures, "amplitude_min"),
                amplitude_max: parse_name_from_captures!(captures, "amplitude_max"),
                roll: parse_name_from_captures!(captures, "roll"),
                pitch: parse_name_from_captures!(captures, "pitch"),
            })
        } else {
            return Err(Error::StopScanFormat(s.to_string()));
        }
    }
}
