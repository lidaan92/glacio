use {Camera, Error, Result};
use iron::{Chain, Handler, IronResult, Plugin, Request, Response, status};
use iron::headers::ContentType;
use iron::typemap::Key;
use persistent::Read;
use router::Router;
use serde::Serialize;
use serde_json::{self, Value};
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
    cameras: Vec<CameraConfig>,
}

#[derive(Clone, Deserialize, Debug)]
struct CameraConfig {
    name: String,
    description: String,
    path: String,
}

#[derive(Copy, Clone, Debug)]
struct Cameras;
impl Key for Cameras {
    type Value = HashMap<String, CameraConfig>;
}

trait ToJson {
    fn summary(&self, request: &Request) -> Value;
    fn detail(&self, request: &Request) -> Value {
        self.summary(request)
    }
}

fn json_response<S: Serialize>(data: S) -> IronResult<Response> {
    let mut response = Response::with((status::Ok, itry!(serde_json::to_string(&data))));
    response.headers.set(ContentType::json());
    Ok(response)
}

fn cameras(request: &mut Request) -> IronResult<Response> {
    let arc = request.get::<Read<Cameras>>().unwrap();
    let cameras = arc.as_ref();
    let cameras = cameras.values().map(|camera| camera.summary(request)).collect::<Vec<_>>();
    json_response(cameras)
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
    json_response(camera.detail(request))
}

fn camera_images(request: &mut Request) -> IronResult<Response> {
    let arc = request.get::<Read<Cameras>>().unwrap();
    let cameras = arc.as_ref();
    let name = request.extensions
        .get::<Router>()
        .unwrap()
        .find("name")
        .unwrap();
    let camera = iexpect!(cameras.get(name));
    let camera = itry!(Camera::new(&camera.path));
    unimplemented!()
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

        let mut chain = Chain::new(router);
        let cameras = config.cameras();
        chain.link(Read::<Cameras>::both(cameras));

        Ok(Api { chain: chain })
    }
}

impl Config {
    fn cameras(&self) -> HashMap<String, CameraConfig> {
        self.cameras
            .iter()
            .map(|&ref config| (config.name.clone(), config.clone()))
            .collect()
    }
}

impl Handler for Api {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        self.chain.handle(request)
    }
}

impl ToJson for CameraConfig {
    fn summary(&self, request: &Request) -> Value {
        let url = url_for!(request, "camera", "name" => self.name.to_string());
        let images_url = url_for!(request, "camera_images", "name" => self.name.to_string());
        json!({
            "name": self.name,
            "url": url.as_ref().as_str(),
            "images_url": images_url.as_ref().as_str(),
        })
    }
}
