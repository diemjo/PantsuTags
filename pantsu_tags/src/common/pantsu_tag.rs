use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;
use chrono::{NaiveDateTime};
use enum_iterator::IntoEnumIterator;
use crate::common::error::{Result, Error};

use super::image_info::DATE_TIME_FORMAT;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct PantsuTag {
    pub tag_type: PantsuTagType,
    pub tag_name: String,
}

impl PantsuTag {
    pub fn new(tag_name: String, tag_type: PantsuTagType) -> Self {
        PantsuTag { tag_name, tag_type }
    }
    
    pub fn display_vec(vec: &Vec<PantsuTag>) -> String {
        String::from("[") + &vec.iter().map(|t|t.to_string()).collect::<Vec<String>>().join(", ") + "]"
    }

    pub fn serialize(&self) -> String {
        format!("{}:{}", self.tag_type.serialize(), self.tag_name)
    }

    pub fn deserialize(text: &str) -> Result<Self> {
        let split = text.split_once(':');
        match split {
            Some((tag_type, tag_name)) => Ok(PantsuTag { tag_type: PantsuTagType::deserialize(tag_type)?, tag_name: tag_name.to_string() }),
            None => Err(Error::InvalidTagFormat(text.to_string()))
        }
    }
}

impl fmt::Display for PantsuTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.tag_type, self.tag_name)
    }
}

impl FromStr for PantsuTag {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let split = s.split_once(':');
        match split {
            Some((tag_type, tag_name)) => Ok(PantsuTag { tag_type: PantsuTagType::from_str(tag_type)?, tag_name: tag_name.to_string() }),
            None => Err(Error::InvalidTagFormat(String::from(s))),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct PantsuTagInfo {
    pub tag: PantsuTag,
    pub tag_author: PantsuTagAuthor,
    pub date_added: NaiveDateTime
}

impl PantsuTagInfo {
    pub fn serialize(&self) -> String {
        format!("{};{};{}", self.tag.serialize(), self.tag_author.serialize(), self.date_added.format(DATE_TIME_FORMAT))
    }

    pub fn deserialize(text: &str) -> Result<Self> {
        let reg = text.rsplitn(3, ';').collect::<Vec<_>>();
        if reg.len()<3 {
            Err(Error::InvalidTagFormat(text.to_string()))
        } else {
            Ok(PantsuTagInfo {
                tag: PantsuTag::deserialize(reg[2])?,
                tag_author: PantsuTagAuthor::deserialize(reg[1])?,
                date_added: NaiveDateTime::parse_from_str(reg[0], DATE_TIME_FORMAT)
                    .or_else(|e| Err(Error::InvalidTagFormat(e.to_string())))?
            })
        }
    }
}

/*impl fmt::Display for PantsuTagInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.tag.to_string(), self.tag_author)
    }
}*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoEnumIterator)]
pub enum PantsuTagType {
    Artist,
    Source,
    Character,
    General,
    Rating,
    Custom
}

impl PantsuTagType {
    pub fn serialize(&self) -> String {
        let str = match self {
            PantsuTagType::Artist => "artist",
            PantsuTagType::Source => "source",
            PantsuTagType::Character => "character",
            PantsuTagType::General => "general",
            PantsuTagType::Rating => "rating",
            PantsuTagType::Custom => "custom"
        };
        String::from(str)
    }

    pub fn deserialize(text: &str) -> Result<Self> {
        match text {
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

impl fmt::Display for PantsuTagType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.serialize())
    }
}

impl FromStr for PantsuTagType {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::deserialize(s)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum PantsuTagAuthor {
    Gelbooru,
    User
}

impl PantsuTagAuthor {
    pub fn serialize(&self) -> String {
        let str = match self {
            PantsuTagAuthor::Gelbooru => "gelbooru",
            PantsuTagAuthor::User => "user",
        };
        String::from(str)
    }

    pub fn deserialize(text: &str) -> Result<Self> {
        match text {
            "gelbooru" => Ok(PantsuTagAuthor::Gelbooru),
            "user" => Ok(PantsuTagAuthor::User),
            other => Err(Error::InvalidTagAuthor(String::from(other)))
        }
    }
}

/*impl fmt::Display for PantsuTagAuthor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
                PantsuTagAuthor::Gelbooru => "gelbooru",
                PantsuTagAuthor::User => "user",
            }
        )
    }
}

impl FromStr for PantsuTagAuthor {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gelbooru" => Ok(PantsuTagAuthor::Gelbooru),
            "user" => Ok(PantsuTagAuthor::User),
            other => Err(Error::InvalidTagAuthor(String::from(other)))
        }
    }
}*/