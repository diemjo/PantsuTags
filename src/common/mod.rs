use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;
use crate::common::error::Error;

pub mod error;

pub struct PantsuTag {
    pub tag_name: String,
    pub tag_type: PantsuTagType
}

#[derive(Debug)]
pub enum PantsuTagType {
    Artist,
    Source,
    Character,
    Generic,
    Custom
}

impl fmt::Display for PantsuTagType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let str = match self {
            PantsuTagType::Artist => "artist",
            PantsuTagType::Source => "source",
            PantsuTagType::Character => "character",
            PantsuTagType::Generic => "generic",
            PantsuTagType::Custom => "custom"
        };
        write!(f, "{}", str)
    }
}

impl FromStr for PantsuTagType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "artist" => Ok(PantsuTagType::Artist),
            "source" => Ok(PantsuTagType::Source),
            "character" => Ok(PantsuTagType::Character),
            "generic" => Ok(PantsuTagType::Generic),
            "custom" => Ok(PantsuTagType::Custom),
            other => Err(Error::InvalidTagType(String::from(other)))
        }
    }
}