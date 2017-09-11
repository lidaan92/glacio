use {Error, Result};
use glacio::camera::{Camera, Server};

/// Global configuration for our remote cameras.
#[derive(Default, Clone, Deserialize, Debug)]
pub struct Config {
    /// The document root that is used to turn local paths into a url.
    pub document_root: String,
    /// A vector of cameras.
    pub cameras: Vec<CameraConfig>,
}

/// Configuration for a single camera.
///
/// Every seperate image directory gets its own camera. This means that dual cameras have two
/// camera configurations.
#[derive(Default, Clone, Deserialize, Debug)]
pub struct CameraConfig {
    /// The name of the camera.
    pub name: String,
    /// A multi-sentence description for the camera.
    pub description: String,
    /// The local directory that holds the camera images.
    pub path: String,
    /// The expected hourly interval between pictures.
    pub interval: f32,
}

impl Config {
    /// Returns the image server for this configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio_http::cameras::Config;
    /// let config = Config::default();
    /// let server = config.server();
    /// ```
    pub fn server(&self) -> Result<Server> {
        Server::new(&self.document_root).map_err(Error::from)
    }
}

impl CameraConfig {
    /// Returns the glacio camera for this configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio_http::cameras::CameraConfig;
    /// let config = CameraConfig { path: ".".to_string(), ..Default::default() };
    /// let camera = config.to_camera().unwrap();
    /// ```
    pub fn to_camera(&self) -> Result<Camera> {
        Camera::new(&self.path).map_err(Error::from)
    }
}
