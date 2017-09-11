use {Error, Result};
use cameras::{CameraConfig, Config, image};
use iron::Request;

/// A serializable summary of a camera.
#[derive(Serialize, Debug)]
pub struct Summary {
    /// The name of the camera.
    pub name: String,
    /// A description of the camera's location and its use.
    pub description: String,
    /// The url to retrieve detailed information about this camera.
    pub url: String,
    /// The url for this camera's images.
    pub images_url: String,
    /// The hourly interval that this camera takes pictures.
    pub interval: f32,
}

/// A serializable detail about camera data.
#[derive(Serialize, Debug)]
pub struct Detail {
    /// The name of the camera.
    pub name: String,
    /// A description of the camera's location and its use.
    pub description: String,
    /// The url to retrieve detailed information about this camera.
    pub url: String,
    /// The url for this camera's images.
    pub images_url: String,
    /// The most recent image captured by this camera.
    pub latest_image: image::Summary,
    /// The hourly interval that this camera takes pictures.
    pub interval: f32,
}

impl Summary {
    /// Creates a new summary from a configuration and a request.
    pub fn new(request: &mut Request, camera: &CameraConfig) -> Summary {
        Summary {
            name: camera.name.clone(),
            description: camera.description.clone(),
            url: url_for!(request, "camera", "name" => camera.name.clone()).as_ref().to_string(),
            images_url: url_for!(request, "camera-images", "name" => camera.name.clone())
                .as_ref()
                .to_string(),
            interval: camera.interval,
        }
    }
}

impl Detail {
    /// Creates a new detail from a configuration and a request.
    pub fn new(request: &mut Request,
               camera_config: &CameraConfig,
               config: &Config)
               -> Result<Detail> {
        let summary = Summary::new(request, camera_config);
        let camera = camera_config.to_camera()?;
        let mut images = camera.images()?
            .filter_map(|result| result.ok())
            .collect::<Vec<_>>();
        if images.is_empty() {
            return Err(Error::Config(format!("No images found for camera: {:?}", camera)));
        }
        images.sort();
        Ok(Detail {
               name: summary.name,
               description: summary.description,
               url: summary.url,
               images_url: summary.images_url,
               latest_image: image::Summary::new(&images.pop().unwrap(), &config)?,
               interval: summary.interval,
           })
    }
}
