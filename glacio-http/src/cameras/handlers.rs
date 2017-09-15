//! Iron handlers for our remote camera systems.

use {Error, Paginate, Result};
use cameras::{CameraConfig, Config, camera, image};
use iron::{IronResult, Request, Response, status};
use iron::headers::Location;
use json;
use router::Router;

/// A multi-route handler for camera-based requests.
///
/// The router works cleanest (IMO) if we can dispatch to a different handler with each route, but
/// there's functionality that we'd like to share between different handlers. This structure brings
/// together all camera-based handler functions. The Iron `Handler` trait is not actually
/// implemented here, since we just pass these methods as closure-wrapped functions to our router
/// setup.
#[derive(Clone, Debug)]
pub struct Cameras {
    config: Config,
}

impl From<Config> for Cameras {
    fn from(config: Config) -> Cameras {
        Cameras { config: config }
    }
}

impl Cameras {
    /// Returns a list of all configured cameras.
    pub fn summary(&self, request: &mut Request) -> IronResult<Response> {
        json::response(self.config
                           .cameras
                           .iter()
                           .map(|config| camera::Summary::new(request, config))
                           .collect::<Vec<_>>())
    }

    /// Returns detail about one camera, as requested in the parameters.
    pub fn detail(&self, request: &mut Request) -> IronResult<Response> {
        let camera_config = iexpect!(self.camera_config(request));
        json::response(itry!(camera::Detail::new(request, camera_config, &self.config)))
    }

    /// Returns a (paginated) list of images associated with the asked-for camera, starting with
    /// the most recent images.
    pub fn images(&self, request: &mut Request) -> IronResult<Response> {
        let camera_config = iexpect!(self.camera_config(request));
        let mut images = itry!(camera_config.to_camera()
                                   .and_then(|camera| camera.images().map_err(Error::from))
                                   .and_then(|images| {
                                                 images.map(|r| r.map_err(Error::from))
                                                     .collect::<Result<Vec<_>>>()
                                             }));
        images.sort_by(|a, b| b.cmp(a));
        let image_summaries =
            itry!(images.into_iter().paginate(request).and_then(|iter| {
                                                                                      iter
                      .map(|image| image::Summary::new(&image, &self.config))
                      .collect::<Result<Vec<_>>>()
                                                                                  }));
        json::response(image_summaries)
    }

    /// Returns a redirect to the src url for the latest image for this camera.
    pub fn latest_image_redirect(&self, request: &mut Request) -> IronResult<Response> {
        let camera_config = iexpect!(self.camera_config(request));
        let camera = itry!(camera_config.to_camera());
        let image = iexpect!(camera.latest_image());
        let server = itry!(self.config.server());
        let url = itry!(server.url_for(&image));
        let mut response = Response::with(status::Found);
        response.headers.set(Location(url.to_string()));
        Ok(response)
    }

    fn name(&self, request: &mut Request) -> Option<String> {
        request.extensions
            .get::<Router>()
            .unwrap()
            .find("name")
            .map(|s| s.to_string())
    }

    fn camera_config(&self, request: &mut Request) -> Option<&CameraConfig> {
        self.name(request).and_then(|name| {
                                        self.config
                                            .cameras
                                            .iter()
                                            .find(|config| config.name == name)
                                    })
    }
}

#[cfg(test)]
mod tests {
    use {Api, Config};
    use cameras::CameraConfig;
    use iron::Headers;
    use iron::headers::Location;
    use iron::status::Status;
    use iron_test::{ProjectBuilder, request, response};
    use serde_json::{self, Value};

    fn build_api(builder: &ProjectBuilder) -> Api {
        let mut config = Config::new();
        config.cameras.document_root = builder.root().to_string_lossy().into_owned();
        config.cameras.cameras.push(CameraConfig {
                                        name: "ATLAS_CAM".to_string(),
                                        description: "Great camera".to_string(),
                                        path: format!("{}/ATLAS_CAM", builder.root().display()),
                                        interval: 3.,
                                        ..Default::default()
                                    });
        Api::new(config).unwrap()
    }

    #[test]
    fn cameras() {
        let builder = ProjectBuilder::new("cameras");
        let handler = build_api(&builder);
        let response = request::get("http://localhost:3000/cameras", Headers::new(), &handler)
            .unwrap();
        let json: Value = serde_json::from_str(&response::extract_body_to_string(response))
            .unwrap();
        let camera = json.get(0).unwrap();
        assert_eq!("ATLAS_CAM", camera.get("name").unwrap());
        assert_eq!("Great camera", camera.get("description").unwrap());
        assert_eq!(3.0, *camera.get("interval").unwrap());
        assert_eq!("http://localhost:3000/cameras/ATLAS_CAM",
                   camera.get("url").unwrap());
        assert_eq!("http://localhost:3000/cameras/ATLAS_CAM/images",
                   camera.get("images_url").unwrap());
        assert_eq!("http://localhost:3000/cameras/ATLAS_CAM/images/latest/redirect",
                   camera.get("latest_image_redirect_url").unwrap());
    }

    #[test]
    fn camera() {
        let builder = ProjectBuilder::new("camera").file("ATLAS_CAM/ATLAS_CAM_20170806_152500.jpg",
                                                         "");
        builder.build();
        let handler = build_api(&builder);
        let response = request::get("http://localhost:3000/cameras/ATLAS_CAM",
                                    Headers::new(),
                                    &handler)
                .unwrap();
        let camera: Value = serde_json::from_str(&response::extract_body_to_string(response))
            .unwrap();
        assert_eq!("ATLAS_CAM", camera.get("name").unwrap());
        assert_eq!("Great camera", camera.get("description").unwrap());
        assert_eq!("http://localhost:3000/cameras/ATLAS_CAM",
                   camera.get("url").unwrap());
        assert_eq!("http://localhost:3000/cameras/ATLAS_CAM/images",
                   camera.get("images_url").unwrap());
        assert_eq!(3.0, *camera.get("interval").unwrap());
        let image = camera.get("latest_image").unwrap();
        assert_eq!("2017-08-06T15:25:00+00:00", image.get("datetime").unwrap());
        assert_eq!("http://iridiumcam.lidar.io/ATLAS_CAM/ATLAS_CAM_20170806_152500.jpg",
                   image.get("url").unwrap());
    }

    #[test]
    fn camera_images() {
        let mut builder = ProjectBuilder::new("camera");
        for i in 0..10 {
            builder = builder.file(format!("ATLAS_CAM/ATLAS_CAM_20170806_15250{}.jpg", i), "");
        }
        builder.build();
        let handler = build_api(&builder);
        let response = request::get("http://localhost:3000/cameras/ATLAS_CAM/images?per_page=2&page=2",
                                    Headers::new(),
                                    &handler)
                .unwrap();
        let images: Value = serde_json::from_str(&response::extract_body_to_string(response))
            .unwrap();
        let image = images.get(0).unwrap();
        assert_eq!("2017-08-06T15:25:07+00:00", image.get("datetime").unwrap());
        assert_eq!("http://iridiumcam.lidar.io/ATLAS_CAM/ATLAS_CAM_20170806_152507.jpg",
                   image.get("url").unwrap());
        let image = images.get(1).unwrap();
        assert_eq!("2017-08-06T15:25:06+00:00", image.get("datetime").unwrap());
        assert_eq!("http://iridiumcam.lidar.io/ATLAS_CAM/ATLAS_CAM_20170806_152506.jpg",
                   image.get("url").unwrap());
        assert_eq!(None, images.get(2));
    }

    #[test]
    fn camera_latest_image_src() {
        let mut builder = ProjectBuilder::new("camera");
        for i in 0..10 {
            builder = builder.file(format!("ATLAS_CAM/ATLAS_CAM_20170806_15250{}.jpg", i), "");
        }
        builder.build();
        let handler = build_api(&builder);
        let response = request::get("http://localhost:3000/cameras/ATLAS_CAM/images/latest/redirect",
                                    Headers::new(),
                                    &handler)
                .unwrap();
        assert_eq!(Some(Status::Found), response.status);
        assert_eq!(&Location("http://iridiumcam.lidar.io/ATLAS_CAM/ATLAS_CAM_20170806_152509.jpg"
                                 .to_string()),
                   response.headers.get::<Location>().unwrap());
    }
}
