use Image;
use failure::Error;
use std::path::{Path, PathBuf};
use std::{fmt, io};

/// A remote camera.
#[derive(Debug)]
pub struct Camera {
    name: String,
    path: PathBuf,
}

/// An error raised when there is no name for the camera.
#[derive(Debug, Fail)]
pub struct NoName(PathBuf);

impl Camera {
    /// Creates a camera from a filesystem path and a subcamera prefix.
    ///
    /// The subcamera prefix is used to walk up the directory chain in the case of dual cameras.
    ///
    /// # Examples
    ///
    /// ```
    /// use camera::Camera;
    /// let camera = Camera::from_path("fixtures/ATLAS_CAM", "").unwrap();
    /// let camera = Camera::from_path("fixtures/HEL_DUAL/StarDot1", "StarDot").unwrap();
    /// assert_eq!("HEL_DUAL_1", camera.name());
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P, subcamera_prefix: &str) -> Result<Camera, Error> {
        let path = path.as_ref();
        let mut name = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| NoName(path.to_path_buf()))?
            .to_string();
        if name.starts_with(subcamera_prefix) {
            let s = name.trim_left_matches(subcamera_prefix).to_string();
            name = path.parent()
                .and_then(|p| p.file_stem())
                .and_then(|s| s.to_str())
                .ok_or_else(|| NoName(path.to_path_buf()))?
                .to_string();
            name.push('_');
            name.push_str(&s);
        }
        Ok(Camera {
            name: name.to_string(),
            path: path.to_path_buf(),
        })
    }

    /// Returns this camera's name.
    ///
    /// # Examples
    ///
    /// ```
    /// use camera::Camera;
    /// let camera = Camera::from_path("fixtures/ATLAS_CAM", "StarDot").unwrap();
    /// assert_eq!("ATLAS_CAM", camera.name());
    /// let camera = Camera::from_path("fixtures/HEL_DUAL/StarDot1", "StarDot").unwrap();
    /// assert_eq!("HEL_DUAL_1", camera.name());
    /// ```
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns a vector of all of this camera's images, sorted by ascending datetime.
    ///
    /// # Examples
    ///
    /// ```
    /// use camera::Camera;
    /// let camera = Camera::from_path("fixtures/ATLAS_CAM", "StarDot").unwrap();
    /// let images = camera.images().unwrap();
    /// assert_eq!(1, images.len());
    /// ```
    pub fn images(&self) -> Result<Vec<Image>, io::Error> {
        let mut images = Vec::new();
        for result in self.path.read_dir()? {
            let entry = result?;
            if let Ok(image) = Image::from_path(entry.path()) {
                images.push(image);
            }
        }
        images.sort_by_key(|image| image.datetime());
        Ok(images)
    }
}

impl fmt::Display for NoName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Could not extract camera name from path: {}",
            self.0.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_name() {
        let camera = Camera::from_path("fixtures/ATLAS_CAM", "StarDot").unwrap();
        assert_eq!("ATLAS_CAM", camera.name());

        let camera = Camera::from_path("fixtures/HEL_DUAL/StarDot1", "StarDot").unwrap();
        assert_eq!("HEL_DUAL_1", camera.name());
        let camera = Camera::from_path("fixtures/HEL_DUAL/StarDot2", "StarDot").unwrap();
        assert_eq!("HEL_DUAL_2", camera.name());

        assert!(Camera::from_path("StarDot1", "StarDot").is_err());
    }
}
