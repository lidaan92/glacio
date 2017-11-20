use {Error, Result, atlas, cameras};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use toml;

/// Configuration for the API.
///
/// All of the paths and other configurations required to drive the entire glacio api. This maps
/// (thanks to serde) onto a TOML configuration file.
#[derive(Clone, Deserialize, Default, Debug)]
pub struct Config {
    /// The configuration for the ATLAS system.
    pub atlas: atlas::Config,
    /// Configuration for our remote cameras.
    pub cameras: cameras::Config,
}

impl Config {
    /// Creates a new configuration from a toml file.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio_http::Config;
    /// let config = Config::from_path("../data/rdcrlpjg.toml").unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut s = String::new();
        File::open(path).and_then(
            |mut read| read.read_to_string(&mut s),
        )?;
        toml::from_str(&s).map_err(Error::from)
    }

    /// Creates a new, default configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio_http::Config;
    /// let config = Config::new();
    /// ```
    pub fn new() -> Config {
        Default::default()
    }
}
