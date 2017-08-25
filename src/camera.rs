use {Error, Result};
use chrono::{DateTime, TimeZone, Utc};
use std::ffi::OsString;
use std::fs::{DirEntry, ReadDir};
use std::path::{Path, PathBuf};

const DEFAULT_EXTENSIONS: &'static [&'static str] = &["jpg"];

/// A remote camera, usually used to take pictures of glaciers or other cool stuff.
#[derive(Debug)]
pub struct Camera {
    directory: PathBuf,
    extensions: Vec<OsString>,
}

/// An iterator over a camera's images.
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
    pub datetime: DateTime<Utc>,
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
                            return Some(Image::new(dir_entry));
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
    fn new(dir_entry: DirEntry) -> Result<Image> {
        if let Some(file_stem) = dir_entry.path().file_stem().and_then(|file_stem| {
                                                                           file_stem.to_str()
                                                                       }) {
            if file_stem.len() <= 15 {
                Err(Error::ImageFilename(format!("File stem {} is too short", file_stem)))
            } else {
                let (_, s) = file_stem.split_at(file_stem.len() - 15);
                Utc.datetime_from_str(s, "%Y%m%d_%H%M%S")
                    .map_err(Error::from)
                    .map(|datetime| Image { datetime: datetime})
            }
        } else {
            Err(Error::ImageFilename(format!("No file stem found for dir_entry: {:?}", dir_entry)))
        }
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
}
