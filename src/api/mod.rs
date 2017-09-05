//! HTTP API for glacio data.
//!
//! # API methods
//!
//! All API calls return JSON. Collections of objects are returned as arrays, e.g.:
//!
//! ```json
//! [{"name":"Thing 1"},{"name":"Thing 2"}]
//! ```
//!
//! ## `/cameras`
//!
//! Returns a summary of all remote cameras, as defined by `glacio::api::CameraSummary`.
//!
//! ## `/cameras/<name>`
//!
//! Returns detailed information about the camera named `<name>`, as defined by
//! `glacio::api::CameraDetail`.
//!
//! ## `/cameras/<name>/images`
//!
//! Returns a list of all images associated with this camera, as defined by
//! `glacio::api::ImageSummary`.
//!
//! ## `/atlas/status`
//!
//! Returns the status of the ATLAS system, as defined by `glacio::api::AtlasStatus`.
//!
//! ## `/atlas/power/history`
//!
//! Returns the power history of the ATLAS system, as defined by an array of
//! `glacio::api::AtlasPowerHistory`.

mod atlas;
mod camera;
mod config;
mod handlers;
mod pagination;

pub use self::atlas::{PowerHistory as AtlasPowerHistory, Status as AtlasStatus};
pub use self::camera::{Detail as CameraDetail, ImageSummary, Summary as CameraSummary};
pub use self::config::Config;
use self::config::PersistentConfig;
use self::pagination::Paginate;
use Result;
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
    /// # use glacio::Api;
    /// let api = Api::from_path("data/rdcrlpjg.toml").unwrap();
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
