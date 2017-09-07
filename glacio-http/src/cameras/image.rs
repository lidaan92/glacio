use Result;
use cameras::Config;
use glacio::camera::Image;

/// A summary of information about an image.
#[derive(Debug, Serialize)]
pub struct Summary {
    /// The image's date and time, as a string.
    pub datetime: String,
    /// The image's url on a remote server.
    pub url: String,
}

impl Summary {
    /// Creates a new summary from a server and an `Image`.
    pub fn new(image: &Image, config: &Config) -> Result<Summary> {
        let server = config.server()?;
        Ok(Summary {
               datetime: image.datetime().to_rfc3339(),
               url: server.url_for(image)?
                   .as_ref()
                   .to_string(),
           })
    }
}
