use Result;
use atlas::handlers::Atlas;
use cameras::handlers::Cameras;
use config::Config;
use iron::{Chain, Handler, IronResult, Request, Response};
use iron::headers::AccessControlAllowOrigin;
use logger::Logger;
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

    /// Creates a new api from a Config.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio_http::{Api, Config};
    /// let config = Config::new();
    /// let api = Api::new(config);
    /// ```
    pub fn new(config: Config) -> Result<Api> {
        let mut router = Router::new();
        let cameras = Cameras::from(config.cameras);
        router.get("/cameras",
                   {
                       let cameras = cameras.clone();
                       move |r: &mut Request| cameras.summary(r)
                   },
                   "cameras");
        router.get("/cameras/:name",
                   {
                       let cameras = cameras.clone();
                       move |r: &mut Request| cameras.detail(r)
                   },
                   "camera");
        router.get("/cameras/:name/images",
                   {
                       let cameras = cameras.clone();
                       move |r: &mut Request| cameras.images(r)
                   },
                   "camera-images");
        router.get("/cameras/:name/images/latest/redirect",
                   {
                       let cameras = cameras.clone();
                       move |r: &mut Request| cameras.latest_image_redirect(r)
                   },
                   "camera-latest-image-redirect");

        let atlas = Atlas::from(config.atlas);
        router.get("/atlas/status",
                   move |r: &mut Request| atlas.status(r),
                   "atlas-status");

        let mut chain = Chain::new(router);
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
