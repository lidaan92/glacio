use iron::{self, Request, IronResult, Response};
use router::Router;

/// Creates a new API router.
pub fn create_router() -> Router {
    let handler = Handler::new();
    router!(cameras: get "/cameras" => handler)
}

struct Handler;

impl Handler {
    fn new() -> Handler {
        Handler
    }
}

impl iron::Handler for Handler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        unimplemented!()
    }
}
