use {Camera, Error, Result};
use iron::{Chain, IronResult, Plugin, Request, Response, status};
use iron::headers::AccessControlAllowOrigin;
use iron::mime::Mime;
use iron::typemap::Key;
use persistent::Read as PersistentRead;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use toml;

/// Creates a new API handler.
pub fn create_handler(config: &Config) -> Chain {
    let cameras = config.cameras
        .iter()
        .map(|config| {
                 let camera = config.to_camera();
                 (camera.name().to_string(), camera)
             })
        .collect::<HashMap<_, _>>();

    let router = router!(
        cameras: get "/cameras" => handle_cameras,
        camera_detail: get "/cameras/:name" => handle_camera_detail,
    );
    let mut chain = Chain::new(router);
    chain.link(PersistentRead::<Cameras>::both(cameras));
    chain
}

fn handle_cameras(request: &mut Request) -> IronResult<Response> {
    let arc = request.get::<PersistentRead<Cameras>>().unwrap();
    json_response(arc.as_ref().values().collect::<Vec<_>>())
}

fn handle_camera_detail(request: &mut Request) -> IronResult<Response> {
    unimplemented!()
}

fn json_response<S: Serialize>(value: S) -> IronResult<Response> {
    let content_type = "application/json".parse::<Mime>().unwrap();
    let json = itry!(serde_json::to_string(&value));
    let mut response = Response::with((content_type, status::Ok, json));
    response.headers.set(AccessControlAllowOrigin::Any);
    Ok(response)
}

#[derive(Debug, Deserialize)]
pub struct Config {
    cameras: Vec<CameraConfig>,
}

#[derive(Debug, Deserialize)]
struct CameraConfig {
    name: String,
}

#[derive(Copy, Clone, Debug)]
struct Cameras;

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

impl Key for Cameras {
    type Value = HashMap<String, Camera>;
}
