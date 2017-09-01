use Result;
use api::atlas::Config as AtlasConfig;
use api::camera::Config as CameraConfig;
use camera::Server as CameraServer;
use iron::typemap::Key;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub atlas: AtlasConfig,
    image_document_root: String,
    cameras: Vec<CameraConfig>,
}

#[derive(Copy, Clone, Debug)]
pub struct PersistentConfig;

impl Config {
    pub fn cameras(&self) -> HashMap<String, CameraConfig> {
        self.cameras
            .iter()
            .map(|config| (config.name.clone(), config.clone()))
            .collect()
    }

    pub fn image_server(&self) -> Result<CameraServer> {
        CameraServer::new(&self.image_document_root)
    }
}

impl Key for PersistentConfig {
    type Value = Config;
}
