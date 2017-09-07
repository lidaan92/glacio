//! Configuration objects for the ATLAS system.

use {Error, Result};
use glacio::atlas::{Efoy, Heartbeat, ReadSbd, SbdSource};

/// ATLAS configuration.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    /// The path to the SBD storage.
    pub path: String,
    /// The IMEI number of the modem that provides the SBD data.
    pub imei: String,
    /// The heartbeat versions that are supported.
    pub versions: Vec<u8>,
    /// The EFOY configuration.
    ///
    /// For now, we assume all EFOYs have the same setup.
    pub efoy: EfoyConfig,
}

/// EFOY configuration.
///
/// This applies to all EFOYs on the ATLAS system.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct EfoyConfig {
    /// A list of the cartridges in the EFOY.
    ///
    /// Order matters, the earlier cartridges are assumed to be emptied first.
    pub cartridges: Vec<EfoyCartridgeConfig>,
}

/// EFOY cartridge configuration.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct EfoyCartridgeConfig {
    /// The name of the cartridge.
    pub name: String,
    /// The capacity of the cartridge.
    pub capacity: f32,
}

impl Config {
    /// Returns this config's heartbeats, with errors filtered out.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio_http::atlas::Config;
    /// let mut config = Config::default();
    /// config.path = "../glacio/data".to_string();
    /// let heartbeats = config.heartbeats().unwrap();
    /// ```
    pub fn heartbeats(&self) -> Result<Vec<Heartbeat>> {
        let heartbeats = self.read_sbd()?
            .flat_map(|r| r.ok())
            .collect::<Vec<_>>();
        if heartbeats.is_empty() {
            Err(Error::Config(format!("No heartbeats in configured path: {}", self.path)))
        } else {
            Ok(heartbeats)
        }
    }

    /// Returns an iterator over this config's `Result<Heartbeat>`s.
    ///
    /// Can be used to query this config's heartbeats while not throwing out errors.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio_http::atlas::Config;
    /// let mut config = Config::default();
    /// config.path = "../glacio/data".to_string();
    /// for result in config.read_sbd().unwrap() {
    ///     match result {
    ///         Ok(heartbeat) => println!("Heartbeat parsed ok: {:?}", heartbeat),
    ///         Err(err) => println!("Problem while parsing heartbeat: {}", err),
    ///     }
    /// }
    /// ```
    pub fn read_sbd(&self) -> Result<ReadSbd> {
        SbdSource::new(&self.path)
            .imeis(&[&self.imei])
            .versions(&self.versions)
            .iter()
            .map_err(Error::from)
    }

    /// Returns a properly-configured `Efoy`.
    ///
    /// Configuration, in this case, means adding the cartridges as defined in this configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio_http::atlas::Config;
    /// let mut config = Config::default();
    /// config.efoy.cartridges.push(("1.1".to_string(), 8.0).into());
    /// let efoy = config.efoy().unwrap();
    /// ```
    pub fn efoy(&self) -> Result<Efoy> {
        let mut efoy = Efoy::new();
        for config in &self.efoy.cartridges {
            efoy.add_cartridge(&config.name, config.capacity)?;
        }
        Ok(efoy)
    }

    /// Returns all efoy cartridge names.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio_http::atlas::Config;
    /// let mut config = Config::default();
    /// config.efoy.cartridges.push(("1.1".to_string(), 8.0).into());
    /// config.efoy.cartridges.push(("1.2".to_string(), 8.0).into());
    /// assert_eq!(vec!["1.1", "1.2"], config.efoy_cartridge_names());
    /// ```
    pub fn efoy_cartridge_names(&self) -> Vec<&str> {
        self.efoy
            .cartridges
            .iter()
            .map(|config| config.name.as_str())
            .collect()
    }
}

impl From<(String, f32)> for EfoyCartridgeConfig {
    fn from((name, capacity): (String, f32)) -> EfoyCartridgeConfig {
        EfoyCartridgeConfig {
            name: name,
            capacity: capacity,
        }
    }
}
