//! EFOY fuel cells power our ATLAS system in the winter.
//!
//! The EFOYs provide their own status information via their own type of heartbeats (contained in
//! the full ATLAS heartbeat messages). In order to construct the history of the EFOY systems, we
//! need to process the full stream of heartbeats for a season.

use atlas::{Error, Result};
use regex::Regex;
use std::slice::Iter;
use std::str::FromStr;

lazy_static! {
    static ref HEARTBEAT_REGEX: Regex = Regex::new(r"(?x)^
        (?P<state>.*),
        cartridge\s(?P<cartridge>.*)\sconsumed\s(?P<consumed>\d+\.\d+)l,
        (?P<voltage>.*),
        (?P<current>.*)
        $").unwrap();
}

/// Instantaneous status report from one of our EFOY fuel cell systems.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Heartbeat {
    /// The state of the efoy system at time of heartbeat.
    pub state: State,
    /// The active cartridge.
    ///
    /// The ATLAS EFOYs have four cartridges, named "1.1", "1.2", "2.1", and "2.2".
    pub cartridge: String,
    /// The fuel consumed so far by the active cartridge.
    pub consumed: f32,
    /// The voltage level of the efoy.
    pub voltage: f32,
    /// The current level of the efoy.
    pub current: f32,
}

/// The operating state/mode of an EFOY fuel cell system.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum State {
    /// The efoy is in auto mode, and is off.
    AutoOff,

    /// The efoy is in auto mode, and is on.
    AutoOn,

    /// The efoy is in an error state.
    Error,

    /// The efoy is heating itself to avoid freezing.
    FreezeProtection,
}

/// Stateful representation of an EFOY system.
///
/// Used to calculate the fuel status of an EFOY through time from a series of `Heartbeat`s.
#[derive(Clone, Debug)]
pub struct Efoy {
    cartridges: Vec<Cartridge>,
}

/// An efoy cartridge.
#[derive(Clone, Debug)]
pub struct Cartridge {
    name: String,
    capacity: f32,
    consumed: f32,
    emptied: bool,
}

/// An iterator over an EFOY's cartridges.
#[derive(Debug)]
pub struct Cartridges<'a> {
    iter: Iter<'a, Cartridge>,
}

impl FromStr for Heartbeat {
    type Err = Error;
    fn from_str(s: &str) -> Result<Heartbeat> {
        if let Some(ref captures) = HEARTBEAT_REGEX.captures(s) {
            Ok(Heartbeat {
                state: parse_name_from_captures!(captures, "state"),
                cartridge: captures.name("cartridge").unwrap().as_str().to_string(),
                consumed: parse_name_from_captures!(captures, "consumed"),
                voltage: parse_name_from_captures!(captures, "voltage"),
                current: parse_name_from_captures!(captures, "current"),
            })
        } else {
            Err(Error::EfoyHeartbeatFormat(s.to_string()))
        }
    }
}

impl Heartbeat {
    /// Returns true if this efoy is on.
    pub fn is_on(&self) -> bool {
        match self.state {
            State::AutoOn => true,
            _ => false,
        }
    }
}

impl Default for State {
    fn default() -> State {
        State::AutoOff
    }
}

impl FromStr for State {
    type Err = Error;
    fn from_str(s: &str) -> Result<State> {
        match s {
            "auto off" => Ok(State::AutoOff),
            "auto on" => Ok(State::AutoOn),
            "error" => Ok(State::Error),
            "freeze protection" => Ok(State::FreezeProtection),
            _ => Err(Error::UnknownEfoyState(s.to_string())),
        }
    }
}

impl From<State> for String {
    fn from(efoy_state: State) -> String {
        match efoy_state {
            State::AutoOff => "auto off".to_string(),
            State::AutoOn => "auto on".to_string(),
            State::Error => "error".to_string(),
            State::FreezeProtection => "freeze protection".to_string(),
        }
    }
}

impl Efoy {
    /// Creates a new efoy with no fuel cartridges.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::atlas::Efoy;
    /// let efoy = Efoy::new();
    /// ```
    pub fn new() -> Efoy {
        Default::default()
    }

    /// Adds a cartridge to this EFOY.
    ///
    /// Order matters. Because of the way heartbeats work, we don't get an explicit "this cartridge
    /// is empty" message, the EFOY just moves on to the next cartridge. Therefore, once we've
    /// moved on to a "later" cartridge, all cartridges "before" it are emptied.
    ///
    /// Returns an error if that a cartridge already exists with that name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::atlas::Efoy;
    /// let mut efoy = Efoy::new();
    /// efoy.add_cartridge("1.1", 8.0).unwrap();
    /// ```
    pub fn add_cartridge(&mut self, name: &str, capacity: f32) -> Result<()> {
        if self.cartridges.iter().any(
            |cartridge| cartridge.name == name,
        )
        {
            return Err(Error::DuplicateEfoyCartridge(name.to_string()));
        }
        self.cartridges.push(Cartridge::new(name, capacity));
        Ok(())
    }

    /// Returns the fuel level for the named cartridge.
    ///
    /// Returns none if there is no cartridge with the provided name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::atlas::Efoy;
    /// let mut efoy = Efoy::new();
    /// efoy.add_cartridge("1.1", 8.0);
    /// assert_eq!(Some(8.0), efoy.fuel("1.1"));
    /// assert_eq!(None, efoy.fuel("not a cartridge"));
    /// ```
    pub fn fuel(&self, name: &str) -> Option<f32> {
        self.cartridge(name).map(|cartridge| cartridge.fuel())
    }

    /// Returns the fuel in a cartridge as a percentage of its capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::atlas::Efoy;
    /// let mut efoy = Efoy::new();
    /// efoy.add_cartridge("1.1", 8.0);
    /// assert_eq!(Some(100.0), efoy.fuel_percentage("1.1"));
    /// ```
    pub fn fuel_percentage(&self, name: &str) -> Option<f32> {
        self.cartridge(name).map(
            |cartridge| cartridge.fuel_percentage(),
        )
    }

    /// Returns the total fuel reamining in this EFOY.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::atlas::Efoy;
    /// let mut efoy = Efoy::new();
    /// efoy.add_cartridge("1.1", 8.0);
    /// assert_eq!(8.0, efoy.total_fuel());
    /// efoy.add_cartridge("1.2", 8.0);
    /// assert_eq!(16.0, efoy.total_fuel());
    /// ```
    pub fn total_fuel(&self) -> f32 {
        self.cartridges.iter().map(|c| c.fuel()).sum()
    }

    /// Returns the total fuel in this EFOY as a percentage of full capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::atlas::Efoy;
    /// let mut efoy = Efoy::new();
    /// efoy.add_cartridge("1.1", 8.0);
    /// assert_eq!(100.0, efoy.total_fuel_percentage());
    /// ```
    pub fn total_fuel_percentage(&self) -> f32 {
        let (fuel, capacity) = self.cartridges.iter().fold((0., 0.), |(fuel, capacity),
         cartridge| {
            (fuel + cartridge.fuel(), capacity + cartridge.capacity)
        });
        100. * fuel / capacity
    }

    /// Process an efoy heartbeat.
    ///
    /// The named cartridge is set to the starting fuel level minus the consumed fuel. All
    /// "earlier" cartridges are set to zero. Order is defined by the order the cartridges were
    /// added to the efoy.
    ///
    /// If a "later" cartridge has already been processed, returns an error.
    ///
    /// ```
    /// # use glacio::atlas::efoy::{Efoy, Heartbeat};
    /// let heartbeat = Heartbeat {
    ///     cartridge: "1.1".to_string(),
    ///     consumed: 4.2,
    ///     ..Default::default()
    /// };
    /// let mut efoy = Efoy::new();
    /// efoy.add_cartridge("1.1", 8.0);
    /// efoy.process(&heartbeat).unwrap();
    /// assert_eq!(8.0 - 4.2, efoy.fuel("1.1").unwrap());
    /// ```
    pub fn process(&mut self, heartbeat: &Heartbeat) -> Result<()> {
        if let Some(cartridge) = self.cartridge(&heartbeat.cartridge) {
            if cartridge.emptied {
                return Err(Error::EmptyCartridge(cartridge.name.clone()));
            }
        } else {
            return Err(Error::CartridgeName(heartbeat.cartridge.to_string()));
        }
        for cartridge in self.cartridges.iter_mut() {
            if cartridge.name == heartbeat.cartridge {
                cartridge.consumed = heartbeat.consumed;
                return Ok(());
            } else {
                cartridge.empty();
            }
        }
        unreachable!()
    }

    /// Returns an iterator over this efoy's cartridges.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::atlas::Efoy;
    /// let mut efoy = Efoy::new();
    /// efoy.add_cartridge("1.1", 8.0);
    /// assert_eq!(1, efoy.iter().count());
    pub fn iter(&self) -> Cartridges {
        Cartridges { iter: self.cartridges.iter() }
    }

    fn cartridge(&self, name: &str) -> Option<&Cartridge> {
        self.cartridges.iter().find(
            |&cartridge| cartridge.name == name,
        )
    }
}

impl Default for Efoy {
    fn default() -> Efoy {
        Efoy { cartridges: Vec::new() }
    }
}

impl Cartridge {
    fn new(name: &str, capacity: f32) -> Cartridge {
        Cartridge {
            name: name.to_string(),
            capacity: capacity,
            consumed: 0.,
            emptied: false,
        }
    }

    /// Returns the name of this cartridge.
    pub fn name(&self) -> &str {
        &self.name
    }

    fn fuel(&self) -> f32 {
        self.capacity - self.consumed
    }

    /// Returns the fuel percentage of this cartridge.
    pub fn fuel_percentage(&self) -> f32 {
        100. * self.fuel() / self.capacity
    }

    fn empty(&mut self) {
        self.consumed = self.capacity;
        self.emptied = true;
    }
}

impl<'a> Iterator for Cartridges<'a> {
    type Item = &'a Cartridge;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn efoy_add_cartridge() {
        let mut efoy = Efoy::new();
        assert!(efoy.add_cartridge("1.1", 8.0).is_ok());
        assert!(efoy.add_cartridge("1.2", 8.0).is_ok());
        assert!(efoy.add_cartridge("1.1", 8.0).is_err());
    }

    #[test]
    fn efoy_fuel() {
        let mut efoy = Efoy::new();
        efoy.add_cartridge("1.1", 8.0).unwrap();
        assert_eq!(Some(8.0), efoy.fuel("1.1"));
        assert_eq!(None, efoy.fuel("1.2"));
        assert_eq!(Some(100.0), efoy.fuel_percentage("1.1"));
        assert_eq!(8.0, efoy.total_fuel());
        assert_eq!(100.0, efoy.total_fuel_percentage());
        assert!(efoy.add_cartridge("1.2", 8.0).is_ok());
        assert_eq!(Some(8.0), efoy.fuel("1.2"));
        assert_eq!(16.0, efoy.total_fuel());
        assert_eq!(100.0, efoy.total_fuel_percentage());
    }

    #[test]
    fn efoy_process() {
        let mut efoy = Efoy::new();
        efoy.add_cartridge("1.1", 8.0).unwrap();
        efoy.add_cartridge("1.2", 8.0).unwrap();
        efoy.add_cartridge("2.1", 8.0).unwrap();
        efoy.add_cartridge("2.2", 8.0).unwrap();
        let mut heartbeat = Heartbeat {
            cartridge: "1.1".to_string(),
            consumed: 4.2,
            ..Default::default()
        };
        efoy.process(&heartbeat).unwrap();
        assert_eq!(8.0 - 4.2, efoy.fuel("1.1").unwrap());
        assert_eq!(32.0 - 4.2, efoy.total_fuel());
        assert_eq!(
            100. * (8.0 - 4.2) / 8.0,
            efoy.fuel_percentage("1.1").unwrap()
        );
        assert_eq!(100. * ((32.0 - 4.2) / 32.0), efoy.total_fuel_percentage());

        heartbeat.cartridge = "3.1".to_string();
        assert!(efoy.process(&heartbeat).is_err());
        assert_eq!(8.0 - 4.2, efoy.fuel("1.1").unwrap());

        heartbeat.cartridge = "1.2".to_string();
        efoy.process(&heartbeat).unwrap();
        assert_eq!(0.0, efoy.fuel("1.1").unwrap());
        assert_eq!(8.0 - 4.2, efoy.fuel("1.2").unwrap());

        heartbeat.cartridge = "1.1".to_string();
        assert!(efoy.process(&heartbeat).is_err());
    }
}
