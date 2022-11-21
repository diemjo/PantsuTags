use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::path::Path;

use crate::sauce::Sauce;
use crate::common;
use crate::image_similarity::NamedImage;


#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ImageHandle {
    filename: String,
    file_source: Sauce,
    file_res: (u32, u32),
}

impl ImageHandle {
    pub(crate) fn new(filename: String, file_source: Sauce, file_res: (u32, u32)) -> ImageHandle {
        ImageHandle {
            filename,
            file_source,
            file_res
        }
    }

    pub fn get_filename(&self) -> &str {
        self.filename.as_str()
    }

    pub fn get_path(&self, lib_path: &Path) -> String { common::get_path(&lib_path.join(&self.filename)) }

    pub fn get_sauce(&self) -> &Sauce {
        &self.file_source
    }

    pub fn get_res(&self) -> (u32, u32) {
        self.file_res
    }
}

impl Display for ImageHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: res={:>12}, sauce='{}'", self.filename, format!("({},{})", self.file_res.0, self.file_res.1), self.get_sauce())
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