use std::fmt;

use chrono::NaiveDateTime;

use crate::{ImageHandle, Sauce, Error, Result, sauce::{NOT_EXISTING_FLAG, NOT_CHECKED_FLAG}};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ImageInfo {
    image_handle: ImageHandle,
    image_sauce: Sauce,
    image_res: (u32, u32),
    date_added: NaiveDateTime,
    date_modified: NaiveDateTime,
}

pub(crate) const DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

impl ImageInfo {
    pub(crate) fn new(image_handle: ImageHandle, image_sauce: Sauce, image_res: (u32, u32), date_added: NaiveDateTime, date_modified: NaiveDateTime) -> Self {
        ImageInfo { image_handle, image_sauce, image_res, date_added, date_modified }
    }

    pub fn get_image(&self) -> &ImageHandle {
        &self.image_handle
    }

    pub fn get_sauce(&self) -> &Sauce {
        &self.image_sauce
    }

    pub fn get_res(&self) -> (u32, u32) {
        self.image_res
    }

    pub fn get_date_added(&self) -> &NaiveDateTime {
        &self.date_added
    }

    pub fn get_date_modified(&self) -> &NaiveDateTime {
        &self.date_modified
    }

    pub fn serialize(&self) -> String {
        format!("{};{};{};{};{};{}",
            self.image_handle.get_filename(),
            self.image_sauce,
            self.image_res.0,
            self.image_res.1,
            self.date_added.format(DATE_TIME_FORMAT),
            self.date_modified.format(DATE_TIME_FORMAT)
        )
    }

    pub fn deserialize(text: &str) -> Result<Self> {
        let split = text.splitn(6, ';').collect::<Vec<_>>();
        if split.len()!=6 {
            Err(Error::InvalidImportFileLineFormat(text.to_string()))
        } else {
            Ok(ImageInfo {
                image_handle: ImageHandle::new(split[0].to_string())?,
                image_sauce: match split[1] {
                    NOT_EXISTING_FLAG => Sauce::NotExisting,
                    NOT_CHECKED_FLAG => Sauce::NotChecked,
                    other => Sauce::Match(crate::sauce::url_from_str(other)?)
                },
                image_res: (
                    split[2].parse::<u32>().or_else(|_| Err(Error::InvalidImportFileLineFormat(text.to_string())))?,
                    split[3].parse::<u32>().or_else(|_| Err(Error::InvalidImportFileLineFormat(text.to_string())))?
                ),
                date_added: NaiveDateTime::parse_from_str(split[4], DATE_TIME_FORMAT).or_else(|_| Err(Error::InvalidImportFileLineFormat(text.to_string())))?,
                date_modified: NaiveDateTime::parse_from_str(split[5], DATE_TIME_FORMAT).or_else(|_| Err(Error::InvalidImportFileLineFormat(text.to_string())))?,
            })
        }
    }
}

impl fmt::Display for ImageInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, res=({}x{}), sauce={}, date={}", self.image_handle.get_filename(), &self.image_res.0, self.image_res.1, self.image_sauce, self.date_added.format(DATE_TIME_FORMAT))
    }
}