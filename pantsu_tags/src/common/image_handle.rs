use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::path::{Path, PathBuf};

use crate::{file_handler, Error, Result};
use crate::image_similarity::NamedImage;


#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ImageHandle {
    filename: String,
}

impl ImageHandle {
    pub fn new(filename: String) -> Result<ImageHandle> {
        if file_handler::filename_is_valid(&filename) {
            Ok(ImageHandle { filename })
        } else {
            Err(Error::InvalidFilename(String::from(filename)))
        }
    }

    pub fn get_filename(&self) -> &str {
        self.filename.as_str()
    }

    pub fn get_path(&self, lib_path: &Path) -> PathBuf { lib_path.join(&self.filename) }
}

impl Display for ImageHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.filename)
    }
}

impl NamedImage for ImageHandle {
    fn get_name(&self) -> &str {
        self.get_filename()
    }
}

/*impl FromStr for Sauce {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            NOT_EXISTING_FLAG => Ok(Sauce::NotExisting),
            NOT_CHECKED_FLAG => Ok(Sauce::NotChecked),
            other if Url::parse(other).is_ok() => Ok(Sauce::Match(other.to_string())),
            other => Err(Error::InvalidSauce(other.to_string()))
            // only match http(s) urls?
        }
    }
}*/