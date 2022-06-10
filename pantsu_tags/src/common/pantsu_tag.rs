use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;
use enum_iterator::IntoEnumIterator;
use crate::common::error::Error;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct PantsuTag {
    pub tag_name: String,
    pub tag_type: PantsuTagType
}

impl fmt::Display for PantsuTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.tag_type, self.tag_name)
    }
}

impl FromStr for PantsuTag {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splitter = s.splitn(2, ':');
        let first = splitter.next().unwrap();
        match splitter.next() {
            Some(second) => Ok(PantsuTag { tag_name: String::from(second), tag_type: String::from(first).parse()? }),
            None => Err(Error::InvalidTagFormat(String::from(s)))
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoEnumIterator)]
pub enum PantsuTagType {
    Artist,
    Source,
    Character,
    General,
    Rating,
    Custom
}

impl fmt::Display for PantsuTagType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let str = match self {
            PantsuTagType::Artist => "artist",
            PantsuTagType::Source => "source",
            PantsuTagType::Character => "character",
            PantsuTagType::General => "general",
            PantsuTagType::Rating => "rating",
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
            "general" => Ok(PantsuTagType::General),
            "rating" => Ok(PantsuTagType::Rating),
            "custom" => Ok(PantsuTagType::Custom),
            other => Err(Error::InvalidTagType(String::from(other)))
        }
    }
}
