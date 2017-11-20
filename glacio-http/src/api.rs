use Result;
use atlas::handlers::Atlas;
use cameras::handlers::Cameras;
use config::Config;
use iron::{AfterMiddleware, Chain, Handler, IronError, IronResult, Request, Response, Url};
use iron::headers::AccessControlAllowOrigin;
use logger::Logger;
use router::Router;
use std::path::Path;

/// The Iron JSON api handler.
#[allow(missing_debug_implementations)]
pub struct Api {
    chain: Chain,
}

struct Custom404;

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
        router.get("/", root, "root");

        let cameras = Cameras::from(config.cameras);
        router.get(
            "/cameras",
            {
                let cameras = cameras.clone();
                move |r: &mut Request| cameras.summary(r)
            },
            "cameras",
        );
        router.get(
            "/cameras/:name",
            {
                let cameras = cameras.clone();
                move |r: &mut Request| cameras.detail(r)
            },
            "camera",
        );
        router.get(
            "/cameras/:name/images",
            {
                let cameras = cameras.clone();
                move |r: &mut Request| cameras.images(r)
            },
            "camera-images",
        );
        router.get(
            "/cameras/:name/images/nearest/:datetime",
            {
                let cameras = cameras.clone();
                move |r: &mut Request| cameras.nearest_image(r)
            },
            "camera-nearest-image",
        );
        router.get(
            "/cameras/:name/images/latest/redirect",
            {
                let cameras = cameras.clone();
                move |r: &mut Request| cameras.latest_image_redirect(r)
            },
            "camera-latest-image-redirect",
        );

        let atlas = Atlas::from(config.atlas);
        router.get(
            "/atlas/status",
            move |r: &mut Request| atlas.status(r),
            "atlas-status",
        );

        let mut chain = Chain::new(router);
        chain.link(Logger::new(None));

        chain.link_after(Custom404);

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
                iron_error.response.headers.set(
                    AccessControlAllowOrigin::Any,
                );
                iron_error
            })
    }
}

impl AfterMiddleware for Custom404 {
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
        use router::NoRoute;
        use iron::status;
        use serde_json;
        use iron::headers::ContentType;

        if let Some(_) = err.error.downcast::<NoRoute>() {
            let mut response = Response::with((
                status::NotFound,
                serde_json::to_string(&json!({"message": "Not found"}))
                    .unwrap(),
            ));
            response.headers.set(ContentType::json());
            Ok(response)
        } else {
            Err(err)
        }
    }
}

fn root(request: &mut Request) -> IronResult<Response> {
    use json;
    let data = json!({
        "cameras_url": url_for!(request, "cameras").as_ref().to_string(),
        "camera_url": decode(url_for!(request, "camera", "name" => "{name}")),
        "camera_images_url": decode(url_for!(request, "camera-images", "name" => "{name}")),
        "camera_latest_image_redirect_url": decode(url_for!(request, "camera-latest-image-redirect", "name" => "{name}")),
        "atlas_status_url": url_for!(request, "atlas-status").as_ref().to_string(),
    });
    json::response(data)
}

fn decode(url: Url) -> String {
    use percent_encoding;
    percent_encoding::percent_decode(url.as_ref().as_str().as_ref())
        .decode_utf8_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iron::Headers;
    use iron_test::{request, response};
    use serde_json::{self, Value};

    #[test]
    fn root() {
        let api = Api::new(Config::new()).unwrap();
        let response = request::get("http://localhost:3000/", Headers::new(), &api).unwrap();
        let json: Value = serde_json::from_str(&response::extract_body_to_string(response))
            .unwrap();
        assert_eq!("http://localhost:3000/cameras", json["cameras_url"]);
        assert_eq!("http://localhost:3000/cameras/{name}", json["camera_url"]);
        assert_eq!("http://localhost:3000/cameras/{name}/images", json["camera_images_url"]);
        assert_eq!("http://localhost:3000/cameras/{name}/images/latest/redirect", json["camera_latest_image_redirect_url"]);
        assert_eq!("http://localhost:3000/atlas/status", json["atlas_status_url"]);
    }
}
