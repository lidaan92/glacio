use Result;
use config::{Config, PersistentConfig};
use handlers;
use iron::{Chain, Handler, IronResult, Request, Response};
use iron::headers::AccessControlAllowOrigin;
use logger::Logger;
use persistent::Read;
use router::Router;
use std::path::Path;

/// The Iron JSON api handler.
#[allow(missing_debug_implementations)]
pub struct Api {
    chain: Chain,
}

impl Api {
    /// Creates a new api from the provided path to a toml config file.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio_http::Api;
    /// let api = Api::from_path("../data/rdcrlpjg.toml").unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Api> {
        Config::from_path(path).and_then(|config| Api::new(config))
    }

    fn new(config: Config) -> Result<Api> {
        let mut router = Router::new();
        router.get("/cameras", handlers::cameras, "cameras");
        router.get("/cameras/:name", handlers::camera, "camera");
        router.get("/cameras/:name/images",
                   handlers::camera_images,
                   "camera_images");
        router.get("/atlas/status", handlers::atlas_status, "atlas_status");
        router.get("/atlas/power/history",
                   handlers::atlas_power_history,
                   "atlas_power_history");

        let mut chain = Chain::new(router);
        chain.link(Read::<PersistentConfig>::both(config));
        chain.link(Logger::new(None));

        Ok(Api { chain: chain })
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
