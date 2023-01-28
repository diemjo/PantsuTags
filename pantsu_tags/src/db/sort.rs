
// General Sorting ###############################################################################################################

use std::{fmt::Display, str::FromStr};
use lazy_static::lazy_static;
use crate::{Result, Error};

pub struct SortOrder<T> {
    options: Vec<T>
}

impl<T> SortOrder<T>
where T: PartialEq + Display
{
    pub fn new(options: Vec<T>) -> Result<Self> {
        if options.is_empty() {
            return Err(Error::NoSortingOptionSpecified)
        }
        let mut v = Vec::new();
        for option in options {
            if v.contains(&option) {
                return Err(Error::RepeatedSortingOption(option.to_string()));
            } else {
                v.push(option);
            }
        }
        Ok(Self { options: v })
    }
}

impl<T> std::fmt::Display for SortOrder<T>
where T: Display
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.options.iter().map(|o| o.to_string()).collect::<Vec<_>>().join(", "))
    }
}

pub enum SortDirection {
    Asc,
    Desc
}

impl std::fmt::Display for SortDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        })
    }
}

// ImageInfo Sorting ###############################################################################################################

lazy_static! {
    pub static ref DEFAULT_IMAGE_SORT: SortOrder<ImageSortOption> = SortOrder { options: vec![ ImageSortOption::DateAdded(SortDirection::Desc), ImageSortOption::Name(SortDirection::Asc)] };
}

pub enum ImageSortOption {
    Name(SortDirection),
    DateAdded(SortDirection),
    DateModified(SortDirection),
    Sauce(SortDirection),
}

impl PartialEq for ImageSortOption {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl std::fmt::Display for ImageSortOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Name(d) => "filename ".to_string() + &d.to_string(),
            Self::DateAdded(d) => "date_added ".to_string() + &d.to_string(),
            Self::DateModified(d) => "date_modified ".to_string() + &d.to_string(),
            Self::Sauce(d) => "image_source_type ".to_string() + &d.to_string(),
        })
    }
}

const VALID_IMAGE_OPTIONS: &str = "{name, date_added, date_modified, sauce}:{asc, desc}";

impl FromStr for ImageSortOption {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let split = s.split_once(':');
        match split {
            Some((variant, direction)) => {
                let direction = match direction {
                    "asc" => Ok(SortDirection::Asc),
                    "desc" => Ok(SortDirection::Desc),
                    _ => Err(Error::InvalidSortingOption(s.to_string(), VALID_IMAGE_OPTIONS.to_string()))
                }?;
                match variant {
                    "name" => Ok(Self::Name(direction)),
                    "date_added" => Ok(Self::DateAdded(direction)),
                    "date_modified" => Ok(Self::DateModified(direction)),
                    "sauce" => Ok(Self::Sauce(direction)),
                    _ => Err(Error::InvalidSortingOption(s.to_string(), VALID_IMAGE_OPTIONS.to_string()))
                }
            },
            None => Err(Error::InvalidSortingOption(s.to_string(), VALID_IMAGE_OPTIONS.to_string())),
        }
    }
}

// PantsuTagInfo Sorting ###############################################################################################################

lazy_static!{
    pub static ref DEFAULT_TAG_SORT: SortOrder<TagSortOption> = SortOrder { options: vec![ TagSortOption::Type(SortDirection::Asc), TagSortOption::Name(SortDirection::Asc) ] };
}

pub enum TagSortOption {
    Name(SortDirection),
    Type(SortDirection),
    Author(SortDirection),
    DateAdded(SortDirection),
}

impl PartialEq for TagSortOption {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl std::fmt::Display for TagSortOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Name(d) => "image_tags.tag ".to_string() + &d.to_string(),
            Self::Type(d) => "image_tags.tag_type ".to_string() + &d.to_string(),
            Self::DateAdded(d) => "image_tags.date_added ".to_string() + &d.to_string(),
            Self::Author(d) => "image_tags.tag_author ".to_string() + &d.to_string(),
        })
    }
}

const VALID_TAG_OPTIONS: &str = "{name, type, author, date_added}:{asc, desc}";

impl FromStr for TagSortOption {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let split = s.split_once(':');
        match split {
            Some((variant, direction)) => {
                let direction = match direction {
                    "asc" => Ok(SortDirection::Asc),
                    "desc" => Ok(SortDirection::Desc),
                    _ => Err(Error::InvalidSortingOption(s.to_string(), VALID_TAG_OPTIONS.to_string()))
                }?;
                match variant {
                    "name" => Ok(Self::Name(direction)),
                    "type" => Ok(Self::Type(direction)),
                    "author" => Ok(Self::Author(direction)),
                    "date_added" => Ok(Self::DateAdded(direction)),
                    _ => Err(Error::InvalidSortingOption(s.to_string(), VALID_TAG_OPTIONS.to_string()))
                }
            },
            None => Err(Error::InvalidSortingOption(s.to_string(), VALID_TAG_OPTIONS.to_string())),
        }
    }
}