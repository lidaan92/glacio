use {Camera, Image, Result};
use api::Paginate;
use camera::Server;
use iron::Request;

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub name: String,
    pub description: String,
    pub path: String,
}

/// High level information about a remote camera.
#[derive(Serialize, Debug)]
pub struct Summary {
    /// The name of the camera.
    ///
    /// This name uniquely identifies the camera in the glacio system.
    pub name: String,
    /// A description of the camera's location and use.
    pub description: String,
    /// The url to retrieve detailed information about this camera.
    pub url: String,
    /// The url of this camera's images.
    pub images_url: String,
}

/// Detailed information about the camera.
#[derive(Serialize, Debug)]
pub struct Detail {
    /// The name of the camera.
    ///
    /// This name uniquely identifies the camera in the glacio system.
    pub name: String,
    /// A description of the camera's location and use.
    pub description: String,
    /// The url to retrieve detailed information about this camera.
    pub url: String,
    /// The url of this camera's images.
    pub images_url: String,
}

/// A summary of infromation about an image.
#[derive(Serialize, Debug)]
pub struct ImageSummary {
    /// The name of the camera that took this image.
    pub camera_name: String,
    /// The date and time that this image was taken.
    pub datetime: String,
    /// The url that can be used to retrieve this image itself from the lidar.io image server.
    pub url: String,
}

impl Config {
    pub fn summary(&self, request: &Request) -> Summary {
        let url = url_for!(request, "camera", "name" => self.name.clone());
        let images_url = url_for!(request, "camera_images", "name" => self.name.clone());
        Summary {
            name: self.name.clone(),
            description: self.description.clone(),
            url: url.as_ref().to_string(),
            images_url: images_url.as_ref().to_string(),
        }
    }

    pub fn detail(&self, request: &Request) -> Detail {
        self.summary(request).into()
    }

    pub fn images(&self, request: &mut Request, server: &Server) -> Result<Vec<ImageSummary>> {
        let mut images = self.camera()
            .and_then(|camera| camera.images())
            .and_then(|images| images.collect::<Result<Vec<_>>>())?;
        images.sort_by(|a, b| b.cmp(a));
        images.into_iter()
            .paginate(request)?
            .map(|image| self.image_summary(request, server, &image))
            .collect()
    }

    fn camera(&self) -> Result<Camera> {
        Camera::new(&self.path)
    }

    fn image_summary(&self, _: &Request, server: &Server, image: &Image) -> Result<ImageSummary> {
        Ok(ImageSummary {
               camera_name: self.name.to_string(),
               datetime: image.datetime().to_rfc3339(),
               url: server.url_for(image)?.to_string(),
           })
    }
}

impl From<Summary> for Detail {
    fn from(summary: Summary) -> Detail {
        Detail {
            name: summary.name,
            description: summary.description,
            url: summary.url,
            images_url: summary.images_url,
        }
    }
}
