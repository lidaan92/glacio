use Result;
use chrono::{DateTime, Utc};
use iron::Url;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

const DEFAULT_IMAGE_EXTENSIONS: &'static [&'static str] = &[".jpg"];

/// A remote camera that transmits pictures back to home.
#[derive(Clone, Debug)]
pub struct Camera {
    name: String,
    description: String,
    path: PathBuf,
    image_extensions: Vec<OsString>,
}

/// An image on the filesystem.
#[derive(Debug)]
pub struct Image;

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
        }
    }

    /// Returns this camera's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns this camera's description.
    pub fn description(&self) -> &str {
        &self.description
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
                                      Some(Image::new(path))
                                  } else {
                                      None
                                  })
    }

    fn is_valid_image_extension(&self, extension: &OsStr) -> bool {
        self.image_extensions.iter().any(|valid_extension| extension == valid_extension)
    }
}

impl Image {
    fn new<P: AsRef<Path>>(path: P) -> Result<Image> {
        unimplemented!()
    }

    fn url(&self) -> Url {
        unimplemented!()
    }

    fn datetime(&self) -> DateTime<Utc> {
        unimplemented!()
    }
}
