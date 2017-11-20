//! Remote cameras located all over the world.
//!
//! These cameras are installed in remote locations, e.g. Greenland or Alaska. They take pictures
//! at regular intervals, then send those pictures back to a home server via a satellite
//! connection. The images are served via HTTP, right now by http://iridiumcam.lidar.io.

use chrono::{self, DateTime, TimeZone, Utc};
use std::{error, io, result};
use std::cmp::Ordering;
use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::fs::ReadDir;
use std::path::{Path, PathBuf, StripPrefixError};
use url::{self, Url};

const DEFAULT_EXTENSIONS: &'static [&'static str] = &["jpg"];
const DEFAULT_SERVER_BASE_URL: &'static str = "http://iridiumcam.lidar.io";

/// A custom error enum for cameras.
#[derive(Debug)]
pub enum Error {
    /// Wrapper around `chrono::ParseError`.
    ChronoParse(chrono::ParseError),
    /// The file stem is too short to parse for a datetime.
    FileStemTooShort(String),
    /// Wrapper around `std::io::Error`.
    Io(io::Error),
    /// No file stem for the provided path.
    NoFileStem(PathBuf),
    /// Wrapper around `std::path::StripPrefixError`.
    StripPrefix(StripPrefixError),
    /// Wrapper around `url::ParseError`.
    UrlParse(url::ParseError),
}

/// Our custom result type.
pub type Result<T> = result::Result<T, Error>;

/// A remote camera, usually used to take pictures of glaciers or other cool stuff.
#[derive(Debug)]
pub struct Camera {
    path: PathBuf,
    extensions: Vec<OsString>,
}

/// An iterator over a camera's images, wrapped in a `Result` in case something goes wrong parsing
/// the image path.
///
/// # Examples
///
/// ```
/// # use glacio::Camera;
/// let camera = Camera::new("data/ATLAS_CAM").unwrap();
/// for result in camera.images().unwrap() {
///     println!("{}", result.unwrap().path().display());
/// }
/// ```
#[derive(Debug)]
pub struct Images {
    read_dir: ReadDir,
    extensions: Vec<OsString>,
}

/// An image taken by a remote camera and stored on the local filesystem.
///
/// Date and time information are assumed to be stored in the image's filename.
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct Image {
    datetime: DateTime<Utc>,
    path: PathBuf,
}

/// An image server, used to translate a local image path to a url.
#[derive(Debug)]
pub struct Server {
    base_url: Url,
    document_root: PathBuf,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<StripPrefixError> for Error {
    fn from(err: StripPrefixError) -> Error {
        Error::StripPrefix(err)
    }
}

impl From<chrono::ParseError> for Error {
    fn from(err: chrono::ParseError) -> Error {
        Error::ChronoParse(err)
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Error {
        Error::UrlParse(err)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ChronoParse(ref err) => err.description(),
            Error::FileStemTooShort(_) => "file stem is too short",
            Error::Io(ref err) => err.description(),
            Error::NoFileStem(_) => "no file stem for path",
            Error::StripPrefix(ref err) => err.description(),
            Error::UrlParse(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::ChronoParse(ref err) => Some(err),
            Error::FileStemTooShort(_) => None,
            Error::Io(ref err) => Some(err),
            Error::NoFileStem(_) => None,
            Error::StripPrefix(ref err) => Some(err),
            Error::UrlParse(ref err) => Some(err),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Error::ChronoParse(ref err) => err.fmt(f),
            Error::FileStemTooShort(ref file_stem) => {
                write!(
                    f,
                    "file stem is too short for datetime parsing: {}",
                    file_stem
                )
            }
            Error::Io(ref err) => err.fmt(f),
            Error::NoFileStem(ref path) => write!(f, "no file stem for path: {}", path.display()),
            Error::StripPrefix(ref err) => err.fmt(f),
            Error::UrlParse(ref err) => err.fmt(f),
        }
    }
}

impl Camera {
    /// Creates a new camera whose images are located under the provided path.
    ///
    /// The local image path is canonicalized. The path is *not* searched recursively — all images
    /// must be located directly under the path.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// # use glacio::Camera;
    /// let camera = Camera::new("data/ATLAS_CAM").unwrap();
    /// assert_eq!(Path::new("data/ATLAS_CAM").canonicalize().unwrap(), camera.path());
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Camera> {
        Ok(Camera {
            path: path.as_ref().canonicalize()?,
            extensions: DEFAULT_EXTENSIONS.iter().map(|&s| s.into()).collect(),
        })
    }

    /// Returns an iterator over this camera's images.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::Camera;
    /// let camera = Camera::new("data/ATLAS_CAM").unwrap();
    /// let images = camera.images().unwrap().collect::<Vec<_>>();
    /// ```
    pub fn images(&self) -> Result<Images> {
        self.path
            .read_dir()
            .map(|read_dir| {
                Images {
                    read_dir: read_dir,
                    extensions: self.extensions.clone(),
                }
            })
            .map_err(Error::from)
    }

    /// Returns this camera's latest image, or None if there are no images for this camera.
    ///
    /// Images are ordered by their time of capture, as determined by their filename.
    ///
    /// Any underlying errors in the image iterator are turned into `None`. If you need to see the
    /// errors, use `Camera::images()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::Camera;
    /// let camera = Camera::new("data/ATLAS_CAM").unwrap();
    /// let image = camera.latest_image().unwrap();
    /// ```
    pub fn latest_image(&self) -> Option<Image> {
        if let Ok(images) = self.images() {
            let mut images = images.filter_map(|r| r.ok()).collect::<Vec<_>>();
            images.sort();
            images.pop()
        } else {
            None
        }
    }

    /// Returns this camera's path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::Camera;
    /// let camera = Camera::new("data/ATLAS_CAM").unwrap();
    /// let path = camera.path();
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Iterator for Images {
    type Item = Result<Image>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(result) = self.read_dir.next() {
            match result {
                Ok(dir_entry) => {
                    if let Some(extension) = dir_entry.path().extension() {
                        if self.extensions.iter().any(|lhs| lhs == extension) {
                            return Some(Image::new(dir_entry.path()));
                        }
                    }
                }
                Err(err) => return Some(Err(err.into())),
            }
        }
        None
    }
}

impl Image {
    /// Creates a new image from the path, which is canonicalized.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// # use glacio::Image;
    /// let image = Image::new("data/ATLAS_CAM/ATLAS_CAM_20170806_152500.jpg").unwrap();
    /// assert_eq!(
    ///     Path::new("data/ATLAS_CAM/ATLAS_CAM_20170806_152500.jpg").canonicalize().unwrap(),
    ///     image.path()
    /// );
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Image> {
        let path = path.as_ref().canonicalize()?;
        if let Some(file_stem) = path.file_stem().and_then(|file_stem| file_stem.to_str()) {
            if file_stem.len() <= 15 {
                Err(Error::FileStemTooShort(file_stem.to_string()))
            } else {
                let (_, s) = file_stem.split_at(file_stem.len() - 15);
                Utc.datetime_from_str(s, "%Y%m%d_%H%M%S")
                    .map_err(Error::from)
                    .map(|datetime| {
                        Image {
                            datetime: datetime,
                            path: path.clone(),
                        }
                    })
            }
        } else {
            Err(Error::NoFileStem(path.clone()))
        }
    }

    /// Returns this image's local filesystem path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::Image;
    /// let image = Image::new("data/ATLAS_CAM/ATLAS_CAM_20170806_152500.jpg").unwrap();
    /// let path = image.path();
    /// assert!(path.is_absolute());
    /// assert_eq!("ATLAS_CAM_20170806_152500.jpg", path.file_name().unwrap());
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns this image's datetime.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate chrono;
    /// # extern crate glacio;
    /// # use glacio::Image;
    /// # use chrono::{Utc, TimeZone};
    /// # fn main() {
    /// let image = Image::new("data/ATLAS_CAM/ATLAS_CAM_20170806_152500.jpg").unwrap();
    /// let datetime = image.datetime();
    /// assert_eq!(Utc.ymd(2017, 8, 6).and_hms(15, 25, 0), datetime);
    /// # }
    /// ```
    pub fn datetime(&self) -> DateTime<Utc> {
        self.datetime
    }
}

impl Ord for Image {
    fn cmp(&self, other: &Image) -> Ordering {
        self.datetime.cmp(&other.datetime)
    }
}

impl Server {
    /// Creates a new server, defaulting to our lidar.io url as the remote base url.
    ///
    /// The server document root is canonicalized.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// # use glacio::camera::Server;
    /// let server = Server::new("data").unwrap();
    /// assert_eq!(Path::new("data").canonicalize().unwrap(), server.document_root());
    /// ```
    pub fn new<P: AsRef<Path>>(document_root: P) -> Result<Server> {
        Ok(Server {
            document_root: document_root.as_ref().canonicalize()?,
            base_url: Url::parse(DEFAULT_SERVER_BASE_URL).unwrap(),
        })
    }

    /// Returns the url for the provided image.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::camera::{Image, Server};
    /// let image = Image::new("data/ATLAS_CAM/ATLAS_CAM_20170806_152500.jpg").unwrap();
    /// let server = Server::new("data").unwrap();
    /// let url = server.url_for(&image).unwrap();
    /// assert_eq!("http://iridiumcam.lidar.io/ATLAS_CAM/ATLAS_CAM_20170806_152500.jpg",
    ///            url.as_str());
    /// ```
    pub fn url_for(&self, image: &Image) -> Result<Url> {
        let input = image.path().strip_prefix(&self.document_root)?;
        self.base_url.join(&input.to_string_lossy()).map_err(
            Error::from,
        )
    }

    /// Returns this server's document root.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::camera::Server;
    /// let server = Server::new("data").unwrap();
    /// let document_root = server.document_root();
    /// assert!(document_root.is_absolute());
    /// assert_eq!("data", document_root.file_name().unwrap());
    /// ```
    pub fn document_root(&self) -> &Path {
        &self.document_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_camera() {
        Camera::new("data/ATLAS_CAM").unwrap();
    }

    #[test]
    fn camera_images() {
        let camera = Camera::new("data/ATLAS_CAM").unwrap();
        let images = camera.images().unwrap();
        assert_eq!(1, images.count());

        let mut images = camera.images().unwrap();
        let image = images.next().unwrap().unwrap();
        assert_eq!(Utc.ymd(2017, 8, 6).and_hms(15, 25, 0), image.datetime);
    }

    #[test]
    fn server_url() {
        let server = Server::new("data").unwrap();
        let camera = Camera::new("data/ATLAS_CAM").unwrap();
        let image = camera.images().unwrap().next().unwrap().unwrap();
        let url = server.url_for(&image).unwrap();
        assert_eq!(
            "http://iridiumcam.lidar.io/ATLAS_CAM/ATLAS_CAM_20170806_152500.jpg",
            url.as_str()
        );
    }

    #[test]
    fn server_url_subdirectory() {
        let server = Server::new(Path::new("data").canonicalize().unwrap()).unwrap();
        let camera = Camera::new("data/HEL_BERGCAM3/StarDot1").unwrap();
        let image = camera.images().unwrap().next().unwrap().unwrap();
        let url = server.url_for(&image).unwrap();
        assert_eq!(
            "http://iridiumcam.lidar.io/HEL_BERGCAM3/StarDot1/HEL_BERGCAM3_StarDot1_20170825_120000.jpg",
            url.as_str()
        );
    }

    #[test]
    fn server_url_mixing_absolute_and_relative() {
        let server = Server::new("data").unwrap();
        let camera = Camera::new("data/ATLAS_CAM").unwrap();
        let image = camera.images().unwrap().next().unwrap().unwrap();
        let url = server.url_for(&image).unwrap();
        assert_eq!(
            "http://iridiumcam.lidar.io/ATLAS_CAM/ATLAS_CAM_20170806_152500.jpg",
            url.as_str()
        );
    }
}
