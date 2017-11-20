//! Utilities for working with Sutron-style data.
//!
//! This includes stuff like datetime parsing and SBD message reconstruction.

pub mod message;

pub use self::message::Message;
use chrono::{DateTime, ParseError, TimeZone, Utc};

/// The format of Sutron datetimes.
pub const DATETIME_FORMAT: &'static str = "%m/%d/%y %H:%M:%S";

/// Parse a Sutron datetime, as a string, into a `chrono::DateTime<Utc>`.
pub fn parse_datetime<E>(s: &str) -> Result<DateTime<Utc>, E>
where
    E: From<ParseError>,
{
    Utc.datetime_from_str(s, DATETIME_FORMAT).map_err(E::from)
}
