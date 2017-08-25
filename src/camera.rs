//! Remote cameras.
//!
//! These cameras are installed in remote locations, e.g. Greenland or Alaska. They take pictures
//! at regular intervals, then send those pictures back to a home server via a satellite
//! connection. The images are served via HTTP, right now by http://iridiumcam.lidar.io.

use {Error, Result};
use chrono::{DateTime, TimeZone, Utc};
use std::ffi::OsString;
use std::fs::{DirEntry, ReadDir};
use std::path::{Path, PathBuf};
use url::Url;

const DEFAULT_EXTENSIONS: &'static [&'static str] = &["jpg"];
const DEFAULT_SERVER_BASE_URL: &'static str = "http://iridiumcam.lidar.io";

/// A remote camera, usually used to take pictures of glaciers or other cool stuff.
#[derive(Debug)]
pub struct Camera {
    directory: PathBuf,
    extensions: Vec<OsString>,
}

/// An iterator over a camera's images.
#[derive(Debug)]
pub struct Images {
    read_dir: ReadDir,
    extensions: Vec<OsString>,
}

/// A remote camera image.
///
/// These exist on local filesystems and are served via remote servers (e.g.
/// http://iridiumcam.lidar.io).
#[derive(Debug)]
pub struct Image {
    datetime: DateTime<Utc>,
    path: PathBuf,
}

/// An image server.
///
/// A server translates a local path to a url that can be used to fetch the image.
#[derive(Debug)]
pub struct Server {
    base_url: Url,
    document_root: PathBuf,
}

impl Camera {
    /// Creates a new camera with a local image path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::Camera;
    /// let camera = Camera::new("data/ATLAS_CAM");
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Camera {
        Camera {
            directory: path.as_ref().to_path_buf(),
            extensions: DEFAULT_EXTENSIONS.iter().map(|&s| s.into()).collect(),
        }
    }

    /// Returns an iterator over this camera's images.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::Camera;
    /// let camera = Camera::new("data/ATLAS_CAM");
    /// let images = camera.images().unwrap().collect::<Vec<_>>();
    /// ```
    pub fn images(&self) -> Result<Images> {
        self.directory
            .read_dir()
            .map(|read_dir| {
                     Images {
                         read_dir: read_dir,
                         extensions: self.extensions.clone(),
                     }
                 })
            .map_err(Error::from)
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
    /// Creates a new image from a path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::Image;
    /// let image = Image::new("data/ATLAS_CAM/ATLAS_CAM_20170806_152500.jpg").unwrap();
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Image> {
        if let Some(file_stem) = path.as_ref().file_stem().and_then(|file_stem| {
                                                                        file_stem.to_str()
                                                                    }) {
            if file_stem.len() <= 15 {
                Err(Error::ImageFilename(format!("File stem {} is too short", file_stem)))
            } else {
                let (_, s) = file_stem.split_at(file_stem.len() - 15);
                Utc.datetime_from_str(s, "%Y%m%d_%H%M%S")
                    .map_err(Error::from)
                    .map(|datetime| Image { datetime: datetime, path: path.as_ref().to_path_buf()})
            }
        } else {
            Err(Error::ImageFilename(format!("No file stem found for path: {:?}", path.as_ref())))
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
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Server {
    /// Creates a new server.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::camera::Server;
    /// let server = Server::new("data");
    /// ```
    pub fn new<P: AsRef<Path>>(document_root: P) -> Server {
        Server {
            document_root: document_root.as_ref().to_path_buf(),
            base_url: Url::parse(DEFAULT_SERVER_BASE_URL).unwrap(),
        }
    }

    /// Returns the url for the provided image.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::camera::{Image, Server};
    /// let image = Image::new("data/ATLAS_CAM/ATLAS_CAM_20170806_152500.jpg").unwrap();
    /// let server = Server::new("data");
    /// let url = server.url_for(&image).unwrap();
    /// ```
    pub fn url_for(&self, image: &Image) -> Result<Url> {
        let input = image.path().strip_prefix(&self.document_root)?;
        self.base_url.join(&input.to_string_lossy()).map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_camera() {
        Camera::new("data/ATLAS_CAM");
    }

    #[test]
    fn camera_images() {
        let camera = Camera::new("data/ATLAS_CAM");
        let images = camera.images().unwrap();
        assert_eq!(1, images.count());

        let mut images = camera.images().unwrap();
        let image = images.next().unwrap().unwrap();
        assert_eq!(Utc.ymd(2017, 8, 6).and_hms(15, 25, 0), image.datetime);
    }

    #[test]
    fn server_url() {
        let server = Server::new("data");
        let camera = Camera::new("data/ATLAS_CAM");
        let image = camera.images()
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        let url = server.url_for(&image).unwrap();
        assert_eq!("http://iridiumcam.lidar.io/ATLAS_CAM/ATLAS_CAM_20170806_152500.jpg",
                   url.as_str());
    }
}
