use chrono::{DateTime, ParseError, Utc};
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::{Path, PathBuf};

const IMAGE_EXTENSION: &'static str = "jpg";

/// A remote image.
#[derive(Clone, Copy, Debug)]
pub struct Image {
    datetime: DateTime<Utc>,
}

/// Image-specific errors.
#[derive(Debug, Fail)]
pub struct Error {
    path: PathBuf,
    error_kind: ErrorKind,
}

/// The type of error for this image.
#[derive(Debug)]
enum ErrorKind {
    ChronoParse(ParseError),
    Directory,
    DoesNotExit,
    Extension(OsString),
}

impl Image {
    /// Creates an image from a filesystem path.
    ///
    /// # Examples
    ///
    /// ```
    /// use camera::Image;
    /// let image = Image::from_path("fixtures/ATLAS_CAM/ATLAS_CAM_20171004_182500.jpg").unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Image, Error> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(Error::new(ErrorKind::DoesNotExit, path.to_path_buf()));
        }
        if path.is_dir() {
            return Err(Error::new(ErrorKind::Directory, path.to_path_buf()));
        }
        let extension = path.extension().unwrap_or(OsStr::new(""));
        if extension != IMAGE_EXTENSION {
            return Err(Error::new(
                ErrorKind::Extension(extension.to_os_string()),
                path.to_path_buf(),
            ));
        }
        let datetime = datetime_from_path(path)?;
        Ok(Image { datetime: datetime })
    }

    /// Returns this image's datetime.
    ///
    /// # Examples
    ///
    /// ```
    /// use camera::Image;
    /// let image = Image::from_path("fixtures/ATLAS_CAM/ATLAS_CAM_20171004_182500.jpg").unwrap();
    /// let datetime = image.datetime();
    /// ```
    pub fn datetime(&self) -> DateTime<Utc> {
        self.datetime
    }
}

impl Error {
    fn new(error_kind: ErrorKind, path: PathBuf) -> Error {
        Error {
            error_kind: error_kind,
            path: path,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Image error for path {}: ", self.path.display())?;
        match self.error_kind {
            ErrorKind::ChronoParse(ref err) => write!(f, "could not parse datetime: {}", err),
            ErrorKind::Directory => write!(f, "path is a directory"),
            ErrorKind::DoesNotExit => write!(f, "does not exit"),
            ErrorKind::Extension(ref extension) => {
                write!(f, "invalid extension '{}'", extension.to_string_lossy())
            }
        }
    }
}

fn datetime_from_path<P: AsRef<Path>>(path: P) -> Result<DateTime<Utc>, Error> {
    use chrono::TimeZone;
    let path = path.as_ref();
    let s: String = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .split('_')
        .rev()
        .take(2)
        .collect();
    Utc.datetime_from_str(&s, "%H%M%S%Y%m%d")
        .map_err(|e| Error::new(ErrorKind::ChronoParse(e), path.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_path() {
        assert!(Image::from_path("fixtures/ATLAS_CAM/ATLAS_CAM_20171004_182500.jpg").is_ok());
        assert!(Image::from_path("notafile").is_err());
        assert!(Image::from_path("fixtures/ATLAS_CAM").is_err());
        assert!(Image::from_path("fixtures/ATLAS_CAM/ATLAS_CAM_20171004_182500.png").is_err());
        assert!(Image::from_path("fixtures/ATLAS_CAM/ATLAS_CAM_20171004_182500 copy.jpg").is_err());
    }
}
