use {Error, Result, atlas, camera};
use glacio::camera::Server;
use iron::typemap::Key;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use toml;

/// The api configuration.
#[derive(Deserialize, Debug)]
pub struct Config {
    /// The configuration for the ATLAS system.
    pub atlas: atlas::Config,
    image_document_root: String,
    cameras: Vec<camera::Config>,
}

#[derive(Copy, Clone, Debug)]
pub struct PersistentConfig;

impl Config {
    /// Creates a new configuration from a toml file.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut s = String::new();
        File::open(path).and_then(|mut read| read.read_to_string(&mut s))?;
        toml::from_str(&s).map_err(Error::from)
    }

    /// Returns a hash map of all configured cameras, keyed by their names.
    pub fn cameras(&self) -> HashMap<String, camera::Config> {
        self.cameras
            .iter()
            .map(|config| (config.name.clone(), config.clone()))
            .collect()
    }

    /// Returns the configured image server for iridiumcam images.
    pub fn image_server(&self) -> Result<Server> {
        Server::new(&self.image_document_root).map_err(Error::from)
    }
}

impl Key for PersistentConfig {
    type Value = Config;
}
