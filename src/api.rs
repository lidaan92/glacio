use Camera;
use iron::{Chain, Handler, IronResult, Plugin, Request, Response, status};
use iron::headers::ContentType;
use iron::typemap::Key;
use persistent::Read;
use router::Router;
use serde::Serialize;
use serde_json::{self, Value};
use std::collections::HashMap;

/// Iron JSON api for glacio.
#[allow(missing_debug_implementations)]
pub struct Api {
    chain: Chain,
}

#[derive(Copy, Clone, Debug)]
struct Cameras;
impl Key for Cameras {
    type Value = HashMap<String, Camera>;
}

trait Summary {
    fn summary(&self) -> Value;
}

fn json_response<S: Serialize>(data: S) -> IronResult<Response> {
    let mut response = Response::with((status::Ok, itry!(serde_json::to_string(&data))));
    response.headers.set(ContentType::json());
    Ok(response)
}

fn cameras(request: &mut Request) -> IronResult<Response> {
    let arc = request.get::<Read<Cameras>>().unwrap();
    let cameras = arc.as_ref();
    let cameras = cameras.values().map(|camera| camera.summary()).collect::<Vec<_>>();
    json_response(cameras)
}

impl Api {
    /// Creates a new api.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::Api;
    /// let api = Api::new();
    /// ```
    pub fn new() -> Api {
        let mut router = Router::new();
        router.get("/cameras", cameras, "cameras");
        let mut chain = Chain::new(router);
        let cameras = HashMap::new();
        chain.link(Read::<Cameras>::both(cameras));
        Api { chain: chain }
    }
}

impl Handler for Api {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        self.chain.handle(request)
    }
}

impl Summary for Camera {
    fn summary(&self) -> Value {
        unimplemented!()
    }
}
