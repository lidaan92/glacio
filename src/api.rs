use iron::{Request, IronResult, Response, status};
use iron::mime::Mime;
use router::Router;
use serde_json;
use Camera;

/// Creates a new API router.
pub fn create_router() -> Router {
    let world = World::new();
    router!(cameras: get "/cameras" => move |request: &mut Request| world.cameras(request))
}

fn json_response(json: &str) -> Response {
    let content_type = "application/json".parse::<Mime>().unwrap();
    Response::with((content_type, status::Ok, json))
}

struct World {
    cameras: Vec<Camera>,
}

impl World {
    fn new() -> World {
        World { cameras: Vec::new() }
    }

    fn cameras(&self, _: &mut Request) -> IronResult<Response> {
        Ok(json_response(&itry!(serde_json::to_string(&self.cameras))))
    }
}
