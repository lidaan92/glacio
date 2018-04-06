//! Manage data from remote cameras.

#![deny(missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
        trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
        unused_qualifications)]

extern crate chrono;
#[macro_use]
extern crate failure;
extern crate walkdir;

mod camera;
mod image;

pub use camera::Camera;
use failure::Error;
pub use image::Image;
use std::path::Path;
use walkdir::WalkDir;

/// Creates a vector of cameras from a filesystem directory.
///
/// # Examples
///
/// ```
/// let cameras = camera::from_path("fixtures", "StarDot").unwrap();
/// assert_eq!(3, cameras.len());
/// ```
pub fn from_path<P: AsRef<Path>>(path: P, subcamera_prefix: &str) -> Result<Vec<Camera>, Error> {
    let mut cameras = Vec::new();
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| e.file_type().is_dir())
    {
        let entry = entry?;
        if entry
            .path()
            .read_dir()?
            .filter_map(|r| r.ok())
            .any(|dir_entry| Image::from_path(dir_entry.path()).is_ok())
        {
            cameras.push(Camera::from_path(entry.path(), subcamera_prefix)?);
        }
    }
    Ok(cameras)
}

#[cfg(test)]
mod tests {
    #[test]
    fn from_path() {
        let cameras = super::from_path("fixtures", "StarDot").unwrap();
        assert_eq!(3, cameras.len());
    }
}
