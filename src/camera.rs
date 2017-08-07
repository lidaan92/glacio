use {Error, Result};
use std::fs::{DirEntry, ReadDir};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Camera {
    directory: PathBuf,
}

/// An iterator over a camera's images.
pub struct Images {
    read_dir: ReadDir,
}

#[derive(Debug)]
pub struct Image;

impl Camera {
    /// Creates a new camera for the provided path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glacio::Camera;
    /// let camera = Camera::new("data/ATLAS_CAM");
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Camera {
        Camera { directory: path.as_ref().to_path_buf() }
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
            .map(|read_dir| Images { read_dir: read_dir })
            .map_err(Error::from)
    }
}

impl Iterator for Images {
    type Item = Result<Image>;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_dir.next().map(|result| {
                                     result.map_err(Error::from).and_then(|dir_entry| {
                                                                              Image::new(dir_entry)
                                                                          })
                                 })
    }
}

impl Image {
    fn new(dir_entry: DirEntry) -> Result<Image> {
        Ok(Image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_camera() {
        let camera = Camera::new("data/ATLAS_CAM");
    }

    #[test]
    fn camera_images() {
        let camera = Camera::new("data/ATLAS_CAM");
        let images = camera.images().unwrap();
        assert_eq!(1, images.count());
    }
}
