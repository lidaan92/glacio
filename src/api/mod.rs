mod camera;
mod heartbeat;
mod pagination;

pub use self::pagination::Pagination;

use {Error, Result};
use api::camera::Config as CameraConfig;
use api::heartbeat::Config as HeartbeatConfig;
use iron::{Chain, Handler, IronResult, Plugin, Request, Response, status};
use iron::headers::{AccessControlAllowOrigin, ContentType};
use iron::typemap::Key;
use persistent::Read;
use router::Router;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read as IoRead;
use std::path::Path;
use toml;

/// Iron JSON api for glacio.
#[allow(missing_debug_implementations)]
pub struct Api {
    chain: Chain,
}

#[derive(Deserialize, Debug)]
struct Config {
    heartbeats: HeartbeatConfig,
    image_document_root: String,
    cameras: Vec<CameraConfig>,
}

#[derive(Copy, Clone, Debug)]
struct Cameras;
impl Key for Cameras {
    type Value = HashMap<String, CameraConfig>;
}

#[derive(Copy, Clone, Debug)]
struct ImageServer;
impl Key for ImageServer {
    type Value = super::camera::Server;
}

#[derive(Copy, Clone, Debug)]
struct Heartbeats;
impl Key for Heartbeats {
    type Value = HeartbeatConfig;
}

fn json_response<S: Serialize>(data: &S) -> IronResult<Response> {
    let mut response = Response::with((status::Ok, itry!(serde_json::to_string(&data))));
    response.headers.set(ContentType::json());
    Ok(response)
}

fn cameras(request: &mut Request) -> IronResult<Response> {
    let arc = request.get::<Read<Cameras>>().unwrap();
    let cameras = arc.as_ref();
    let cameras = cameras.values().map(|camera| camera.summary(request)).collect::<Vec<_>>();
    json_response(&cameras)
}

fn camera(request: &mut Request) -> IronResult<Response> {
    let arc = request.get::<Read<Cameras>>().unwrap();
    let cameras = arc.as_ref();
    let name = request.extensions
        .get::<Router>()
        .unwrap()
        .find("name")
        .unwrap();
    let camera = iexpect!(cameras.get(name));
    json_response(&camera.detail(request))
}

fn camera_images(request: &mut Request) -> IronResult<Response> {
    let cameras_arc = request.get::<Read<Cameras>>().unwrap();
    let cameras = cameras_arc.as_ref();
    let image_server_arc = request.get::<Read<ImageServer>>().unwrap();
    let image_server = image_server_arc.as_ref();
    let name = request.extensions
        .get::<Router>()
        .unwrap()
        .find("name")
        .unwrap()
        .to_string();
    let camera = iexpect!(cameras.get(&name));
    json_response(&itry!(camera.images(request, image_server)))
}

fn atlas_status(request: &mut Request) -> IronResult<Response> {
    let heartbeats_arc = request.get::<Read<Heartbeats>>().unwrap();
    let heartbeats = heartbeats_arc.as_ref();
    json_response(&itry!(heartbeats.status(request)))
}

impl Api {
    /// Creates a new api from the provided path to a toml config file.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::Api;
    /// let api = Api::from_path("data/rdcrlpjg.toml").unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Api> {
        let mut s = String::new();
        File::open(path).and_then(|mut read| read.read_to_string(&mut s))?;
        toml::from_str(&s).map_err(Error::from).and_then(|config| Api::new(&config))
    }

    fn new(config: &Config) -> Result<Api> {
        let mut router = Router::new();
        router.get("/cameras", cameras, "cameras");
        router.get("/cameras/:name", camera, "camera");
        router.get("/cameras/:name/images", camera_images, "camera_images");
        router.get("/atlas/status", atlas_status, "atlas_status");

        let mut chain = Chain::new(router);
        let cameras = config.cameras();
        let image_server = config.image_server()?;
        chain.link(Read::<Cameras>::both(cameras));
        chain.link(Read::<ImageServer>::both(image_server));
        chain.link(Read::<Heartbeats>::both(config.heartbeats.clone()));

        Ok(Api { chain: chain })
    }
}

impl Config {
    fn cameras(&self) -> HashMap<String, CameraConfig> {
        self.cameras
            .iter()
            .map(|config| (config.name.clone(), config.clone()))
            .collect()
    }

    fn image_server(&self) -> Result<super::camera::Server> {
        super::camera::Server::new(&self.image_document_root)
    }
}

impl Handler for Api {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        self.chain
            .handle(request)
            .map(|mut response| {
                     response.headers.set(AccessControlAllowOrigin::Any);
                     response
                 })
            .map_err(|mut iron_error| {
                         iron_error.response.headers.set(AccessControlAllowOrigin::Any);
                         iron_error
                     })
    }
}
