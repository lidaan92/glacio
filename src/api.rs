use {Camera, Error, Result};
use iron::{Request, Response, status};
use iron::headers::AccessControlAllowOrigin;
use iron::mime::Mime;
use router::Router;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use toml;

/// Creates a new API router.
pub fn create_router(config: &Config) -> Router {
    let world = World::new(config);
    let mut router = Router::new();
    let cameras = world.cameras.clone();
    router.get("/cameras",
               move |_: &mut Request| {
        Ok(json_response(&itry!(serde_json::to_string(&cameras.iter()
                                                           .map(|(_, camera)| camera)
                                                           .collect::<Vec<_>>()))))
    },
               "cameras");
    router
}

fn json_response(json: &str) -> Response {
    let content_type = "application/json".parse::<Mime>().unwrap();
    let mut response = Response::with((content_type, status::Ok, json));
    response.headers.set(AccessControlAllowOrigin::Any);
    response
}

#[derive(Debug, Deserialize)]
pub struct Config {
    cameras: Vec<CameraConfig>,
}

#[derive(Debug, Deserialize)]
struct CameraConfig {
    name: String,
}

#[derive(Clone, Debug)]
struct World {
    cameras: HashMap<String, Camera>,
}

impl Config {
    /// Reads a configuration from a toml file.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut file = File::open(path)?;
        let mut config = String::new();
        file.read_to_string(&mut config)?;
        toml::from_str(&config).map_err(Error::from)
    }
}

impl CameraConfig {
    fn to_camera(&self) -> Camera {
        Camera::new(&self.name)
    }
}

impl World {
    fn new(config: &Config) -> World {
        World {
            cameras: config.cameras
                .iter()
                .map(|config| {
                         let camera = config.to_camera();
                         (camera.name().to_string(), camera)
                     })
                .collect(),
        }
    }
}
