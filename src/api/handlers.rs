use api::PersistentConfig;
use iron::{IronResult, Plugin, Request, Response, status};
use iron::headers::ContentType;
use persistent::Read;
use router::Router;
use serde::Serialize;
use serde_json;

fn json_response<S: Serialize>(data: &S) -> IronResult<Response> {
    let mut response = Response::with((status::Ok, itry!(serde_json::to_string(&data))));
    response.headers.set(ContentType::json());
    Ok(response)
}

pub fn cameras(request: &mut Request) -> IronResult<Response> {
    let arc = request.get::<Read<PersistentConfig>>().unwrap();
    let config = arc.as_ref();
    let cameras = config.cameras()
        .values()
        .map(|camera| camera.summary(request))
        .collect::<Vec<_>>();
    json_response(&cameras)
}

pub fn camera(request: &mut Request) -> IronResult<Response> {
    let arc = request.get::<Read<PersistentConfig>>().unwrap();
    let config = arc.as_ref();
    let name = request.extensions
        .get::<Router>()
        .unwrap()
        .find("name")
        .unwrap();
    let cameras = config.cameras();
    let camera = iexpect!(cameras.get(name));
    json_response(&camera.detail(request))
}

pub fn camera_images(request: &mut Request) -> IronResult<Response> {
    let arc = request.get::<Read<PersistentConfig>>().unwrap();
    let config = arc.as_ref();
    let name = request.extensions
        .get::<Router>()
        .unwrap()
        .find("name")
        .unwrap()
        .to_string();
    let cameras = config.cameras();
    let camera = iexpect!(cameras.get(&name));
    json_response(&itry!(camera.images(request, &itry!(config.image_server()))))
}

pub fn atlas_status(request: &mut Request) -> IronResult<Response> {
    let arc = request.get::<Read<PersistentConfig>>().unwrap();
    let config = arc.as_ref();
    json_response(&itry!(config.atlas.status(request)))
}

pub fn atlas_power_history(request: &mut Request) -> IronResult<Response> {
    let arc = request.get::<Read<PersistentConfig>>().unwrap();
    let config = arc.as_ref();
    json_response(&itry!(config.atlas.power_history(request)))
}
