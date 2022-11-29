use std::collections::HashSet;
use std::iter::FromIterator;
use log::warn;
use rusqlite::Connection;
use crate::common::image_info::ImageInfo;
use crate::common::pantsu_tag::PantsuTagInfo;
use crate::db::{AspectRatio, db_calls, SauceType};
use crate::{ImageHandle, PantsuTag, PantsuTagType};
use crate::error::Result;

pub struct SelectImageTransaction<'a> {
    connection: &'a Connection,
    image: &'a ImageHandle,
}

impl<'a> SelectImageTransaction<'a> {
    pub(crate) fn new(connection: &'a Connection, image: &'a ImageHandle) -> Self {
        SelectImageTransaction {
            connection,
            image,
        }
    }

//impl<'a> PantsuTransaction<Option<ImageHandle>> for SelectImageTransaction<'a> {
    pub fn execute(self) -> Result<Option<ImageInfo>> {
        db_calls::get_image(self.connection, self.image)
    }
}

//#####################################################################################

pub struct SelectImagesTransaction<'a> {
    connection: &'a Connection,
    include_tags: HashSet<PantsuTag>,
    exclude_tags: HashSet<PantsuTag>,
    ratio: AspectRatio,
    sauce_type: SauceType,
}

impl<'a> SelectImagesTransaction<'a> {
    pub(crate) fn new(connection: &'a Connection) -> Self {
        SelectImagesTransaction {
            connection,
            include_tags: HashSet::new(),
            exclude_tags: HashSet::new(),
            ratio: AspectRatio::Any,
            sauce_type: SauceType::Any,
        }
    }

    pub fn including_tag(mut self, tag: &'a PantsuTag) -> Self {
        self.include_tags.insert(tag.clone());
        self
    }

    pub fn including_tags(mut self, tags: &'a Vec<PantsuTag>) -> Self {
        self.include_tags.extend(tags.clone());
        self
    }

    pub fn excluding_tag(mut self, tag: &'a PantsuTag) -> Self {
        self.exclude_tags.insert(tag.clone());
        self
    }

    pub fn excluding_tags(mut self, tags: &'a Vec<PantsuTag>) -> Self {
        self.exclude_tags.extend(tags.clone());
        self
    }

    pub fn with_ratio(mut self, ratio: AspectRatio) -> Self {
        self.ratio = ratio;
        self
    }

    pub fn with_not_checked_sauce(mut self) -> Self {
        self.sauce_type = SauceType::NotChecked;
        self
    }

    pub fn with_not_existing_sauce(mut self) -> Self {
        self.sauce_type = SauceType::NotExisting;
        self
    }

    pub fn with_existing_sauce(mut self) -> Self {
        self.sauce_type = SauceType::Existing;
        self
    }

//impl<'a> PantsuTransaction<Vec<ImageHandle>> for SelectImagesTransaction<'a> {
    pub fn execute(self) -> Result<Vec<ImageInfo>> {
        let images = db_calls::get_images(self.connection, &Vec::from_iter(self.include_tags), &Vec::from_iter(self.exclude_tags), self.sauce_type)?;
        let images = match self.ratio {
            AspectRatio::Any => images,
            AspectRatio:: Max(max) => images.into_iter()
                .filter(|f| {
                    let (w,h) = f.get_res();
                    (w as f64)/(h as f64) <= max as f64
                }).collect(),
            AspectRatio::Min(min) => images.into_iter()
                .filter(|f| {
                    let (w,h) = f.get_res();
                    (w as f64)/(h as f64) >= min as f64
                }).collect(),
            AspectRatio::Range(min, max) => images.into_iter()
                .filter(|f| {
                    let (w,h) = f.get_res();
                    let ratio = (w as f64)/(h as f64);
                    ratio <= (max as f64) && ratio >= (min as f64)
                }).collect(),
        };
        Ok(images)
    }
}

//#############################################################################################

pub struct SelectTagsTransaction<'a> {
    connection: &'a Connection,
    types: HashSet<PantsuTagType>,
}

impl<'a> SelectTagsTransaction<'a> {
    pub(crate) fn new(connection: &'a Connection) -> Self {
        SelectTagsTransaction {
            connection,
            types: HashSet::new(),
        }
    }

    pub fn with_types(mut self, types: &Vec<PantsuTagType>) -> Self {
        self.types.extend(types);
        self
    }

    pub fn execute(self) -> Result<Vec<PantsuTag>> {
        if self.types.len()==0 {
            db_calls::get_all_tags(self.connection)
        } else {
            db_calls::get_tags_with_types(self.connection, &Vec::from_iter(self.types))
        }
    }
}

//#############################################################################################


pub struct SelectImageTagsTransaction<'a> {
    connection: &'a Connection,
    image: &'a ImageHandle,
    types: HashSet<PantsuTagType>,
}

impl<'a> SelectImageTagsTransaction<'a> {
    pub(crate) fn new(connection: &'a Connection, image: &'a ImageHandle) -> Self {
        SelectImageTagsTransaction {
            connection,
            image: image,
            types: HashSet::new(),
        }
    }

    pub fn with_types(mut self, types: &Vec<PantsuTagType>) -> Self {
        self.types.extend(types);
        self
    }

//impl<'a> PantsuTransaction<Vec<PantsuTag>> for SelectTagsTransaction<'a> {
    pub fn execute(self) -> Result<Vec<PantsuTagInfo>> {
        let db_image = db_calls::get_image(self.connection, self.image)?;
        match db_image {
            Some(_) => {
                if self.types.len()==0 {
                    db_calls::get_tags_for_image(self.connection, self.image)
                } else {
                    db_calls::get_tags_for_image_with_types(self.connection, self.image, &Vec::from_iter(self.types))
                }
            },
            None => {
                warn!("Querying tags for not existing image: {}", self.image.get_filename());
                Ok(vec![])
            },
        }
    }
}