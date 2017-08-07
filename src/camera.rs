use Result;
use chrono::{DateTime, Utc};
use iron::Url;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

const DEFAULT_IMAGE_EXTENSIONS: &'static [&'static str] = &["jpg"];
const DEFAULT_IMAGE_SERVER: &'static str = "http://iridiumcam.lidar.io";

/// A remote camera that transmits pictures back to home.
#[derive(Clone, Debug)]
pub struct Camera {
    name: String,
    description: String,
    path: PathBuf,
    image_extensions: Vec<OsString>,
    image_server: Url,
    document_root: PathBuf,
}

/// An image on the filesystem.
#[derive(Debug)]
pub struct Image {
    pub url: Url,
    pub datetime: DateTime<Utc>,
}

impl Camera {
    /// Creates a new camera that references the given local directory.
    ///
    /// The camera's name is set to the directory name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::Camera;
    /// let camera = Camera::new("data/ATLAS_CAM");
    /// assert_eq!("ATLAS_CAM", camera.name());
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Camera {
        Camera {
            name: path.as_ref()
                .file_name()
                .map(|os_str| os_str.to_string_lossy().into_owned())
                .unwrap_or(String::new()),
            description: String::new(),
            path: path.as_ref().to_path_buf(),
            image_extensions: DEFAULT_IMAGE_EXTENSIONS.iter().map(|&s| s.into()).collect(),
            image_server: Url::parse(DEFAULT_IMAGE_SERVER).unwrap(),
            document_root: path.as_ref()
                .parent()
                .unwrap_or(&PathBuf::new())
                .to_path_buf(),
        }
    }

    /// Returns this camera's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets this camera's name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::Camera;
    /// let camera = Camera::new("").set_name("ATLAS_CAM");
    /// assert_eq!("ATLAS_CAM", camera.name());
    /// ```
    pub fn set_name(mut self, name: &str) -> Camera {
        self.name = name.to_string();
        self
    }

    /// Returns this camera's description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Sets this camera's description.
    pub fn set_description(mut self, description: &str) -> Camera {
        self.description = description.to_string();
        self
    }

    /// Returns this camera's path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn images(&self) -> Result<Vec<Image>> {
        self.path()
            .read_dir()?
            .filter_map(|result| match result {
                            Ok(dir_entry) => self.create_image(&dir_entry.path()),
                            Err(err) => Some(Err(err.into())),
                        })
            .collect()
    }

    pub fn latest_image(&self) -> Result<Option<Image>> {
        self.images().map(|images| images.into_iter().last())
    }

    fn create_image(&self, path: &Path) -> Option<Result<Image>> {
        path.extension().and_then(|extension| if self.is_valid_image_extension(extension) {
                                      unimplemented!()
                                  } else {
                                      None
                                  })
    }

    fn is_valid_image_extension(&self, extension: &OsStr) -> bool {
        self.image_extensions.iter().any(|valid_extension| extension == valid_extension)
    }
}
