use iron::{Request, IronResult, Response, status};
use iron::mime::Mime;
use router::Router;
use std::path::Path;
use std::fs::File;
use serde_json;
use {Error, Camera, Result};
use toml;
use std::io::Read;

/// Creates a new API router.
pub fn create_router(config: &Config) -> Router {
    let world = World::new(config);
    router!(cameras: get "/cameras" => move |request: &mut Request| world.cameras(request))
}

fn json_response(json: &str) -> Response {
    let content_type = "application/json".parse::<Mime>().unwrap();
    Response::with((content_type, status::Ok, json))
}

#[derive(Debug, Deserialize)]
pub struct Config {
    cameras: Vec<CameraConfig>,
}

#[derive(Debug, Deserialize)]
struct CameraConfig {
    name: String,
}

struct World {
    cameras: Vec<Camera>,
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

impl World {
    fn new(config: &Config) -> World {
        World {
            cameras: config.cameras
                .iter()
                .map(|camera| Camera::new(&camera.name))
                .collect(),
        }
    }

    fn cameras(&self, _: &mut Request) -> IronResult<Response> {
        Ok(json_response(&itry!(serde_json::to_string(&self.cameras))))
    }
}
