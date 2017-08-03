use {Camera, Error, Result};
use iron::{IronResult, Request, Response, status};
use iron::headers::AccessControlAllowOrigin;
use iron::mime::Mime;
use router::Router;
use serde_json;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use toml;

/// Creates a new API router.
pub fn create_router(config: &Config) -> Router {
    let server = Server::new(config);
    router!(
        cameras: get "/cameras" => move |request: &mut Request| {
            info!("/cameras");
            server.cameras(request)
        }
    )
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

struct Server {
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

impl CameraConfig {
    fn to_camera(&self) -> Camera {
        Camera::new(&self.name)
    }
}

impl Server {
    fn new(config: &Config) -> Server {
        Server {
            cameras: config.cameras
                .iter()
                .map(|config| config.to_camera())
                .collect(),
        }
    }

    fn cameras(&self, _: &mut Request) -> IronResult<Response> {
        Ok(json_response(&itry!(serde_json::to_string(&self.cameras))))
    }
}
