//! Battery systems powering ATLAS.

use atlas::{Error, Result};
use std::str::FromStr;

/// A battery's heartbeat information.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Heartbeat {
    /// The state of charge of a battery, as a percentage out of 100.
    pub state_of_charge: f32,
}

impl FromStr for Heartbeat {
    type Err = Error;
    fn from_str(s: &str) -> Result<Heartbeat> {
        Ok(Heartbeat { state_of_charge: s.parse()? })
    }
}
