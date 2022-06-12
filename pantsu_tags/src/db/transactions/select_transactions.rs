use std::collections::HashSet;
use std::iter::FromIterator;
use rusqlite::Connection;
use crate::db::{AspectRatio, db_calls, SauceType};
use crate::{ImageHandle, PantsuTag, PantsuTagType};
use crate::error::Result;

pub struct SelectImageTransaction<'a> {
    connection: &'a Connection,
    filename: &'a str,
}

impl<'a> SelectImageTransaction<'a> {
    pub(crate) fn new(connection: &'a Connection, filename: &'a str) -> Self {
        SelectImageTransaction {
            connection,
            filename,
        }
    }

//impl<'a> PantsuTransaction<Option<ImageHandle>> for SelectImageTransaction<'a> {
    pub fn execute(self) -> Result<Option<ImageHandle>> {
        db_calls::get_file(self.connection, self.filename)
    }
}

//#####################################################################################

pub struct SelectImagesTransaction<'a> {
    connection: &'a Connection,
    include_tags: HashSet<&'a str>,
    exclude_tags: HashSet<&'a str>,
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

    pub fn including_tag(mut self, tag: &'a str) -> Self {
        self.include_tags.insert(tag);
        self
    }

    pub fn including_tags(mut self, tags: &'a Vec<String>) -> Self {
        let tags : Vec<&str> = tags.iter()
            .map(|t|t.as_str())
            .collect();
        self.include_tags.extend(tags);
        self
    }

    pub fn excluding_tag(mut self, tag: &'a str) -> Self {
        self.exclude_tags.insert(tag);
        self
    }

    pub fn excluding_tags(mut self, tags: &'a Vec<String>) -> Self {
        let tags : Vec<&str> = tags.iter()
            .map(|t|t.as_str())
            .collect();
        self.exclude_tags.extend(tags);
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
    pub fn execute(self) -> Result<Vec<ImageHandle>> {
        let files = db_calls::get_files(self.connection, &Vec::from_iter(self.include_tags), &Vec::from_iter(self.exclude_tags), self.sauce_type)?;
        let files = match self.ratio {
            AspectRatio::Any => files,
            AspectRatio:: Max(max) => files.into_iter()
                .filter(|f| {
                    let (w,h) = f.get_res();
                    (w as f64)/(h as f64) <= max as f64
                }).collect(),
            AspectRatio::Min(min) => files.into_iter()
                .filter(|f| {
                    let (w,h) = f.get_res();
                    (w as f64)/(h as f64) >= min as f64
                }).collect(),
            AspectRatio::Range(min, max) => files.into_iter()
                .filter(|f| {
                    let (w,h) = f.get_res();
                    let ratio = (w as f64)/(h as f64);
                    ratio <= (max as f64) && ratio >= (min as f64)
                }).collect(),
        };
        Ok(files)
    }
}

//#############################################################################################

pub struct SelectTagsTransaction<'a> {
    connection: &'a Connection,
    filename: Option<&'a str>,
    types: HashSet<PantsuTagType>,
}

impl<'a> SelectTagsTransaction<'a> {
    pub(crate) fn new(connection: &'a Connection) -> Self {
        SelectTagsTransaction {
            connection,
            filename: None,
            types: HashSet::new(),
        }
    }

    pub fn for_image(mut self, filename: &'a str) -> Self {
        self.filename = Some(filename);
        self
    }

    pub fn with_types(mut self, types: &Vec<PantsuTagType>) -> Self {
        self.types.extend(types);
        self
    }

//impl<'a> PantsuTransaction<Vec<PantsuTag>> for SelectTagsTransaction<'a> {
    pub fn execute(self) -> Result<Vec<PantsuTag>> {
        match self.filename {
            Some(filename) => if self.types.len()==0 {
                db_calls::get_tags_for_file(self.connection, filename)
            } else {
                db_calls::get_tags_for_file_with_types(self.connection, filename, &Vec::from_iter(self.types))
            },
            None => if self.types.len()==0 {
                db_calls::get_all_tags(self.connection)
            } else {
                db_calls::get_tags_with_types(self.connection, &Vec::from_iter(self.types))
            }
        }
    }
}