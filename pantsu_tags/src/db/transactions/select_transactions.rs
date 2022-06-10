use std::collections::HashSet;
use std::iter::FromIterator;
use rusqlite::Connection;
use crate::db::{AspectRatio, db_calls};
use crate::{ImageHandle, PantsuTag, PantsuTagType};
use crate::error::Result;

pub struct SelectImageTransaction<'a> {
    connection: &'a Connection,
    filename: String,
}

impl<'a> SelectImageTransaction<'a> {
    pub(crate) fn new(connection: &'a Connection, filename: &str) -> Self {
        SelectImageTransaction {
            connection,
            filename: filename.to_string(),
        }
    }

//impl<'a> PantsuTransaction<Option<ImageHandle>> for SelectImageTransaction<'a> {
    pub fn execute(self) -> Result<Option<ImageHandle>> {
        db_calls::get_file(self.connection, self.filename.as_str())
    }
}

//#####################################################################################

pub struct SelectImagesTransaction<'a> {
    connection: &'a Connection,
    include_tags: HashSet<String>,
    exclude_tags: HashSet<String>,
    ratio: AspectRatio
}

impl<'a> SelectImagesTransaction<'a> {
    pub(crate) fn new(connection: &'a Connection) -> Self {
        SelectImagesTransaction {
            connection,
            include_tags: HashSet::new(),
            exclude_tags: HashSet::new(),
            ratio: AspectRatio::Any
        }
    }

    pub fn including_tag(mut self, tag: &str) -> Self {
        self.include_tags.insert(tag.to_string());
        self
    }

    pub fn including_tags(mut self, tags: &Vec<String>) -> Self {
        self.include_tags.extend(tags.clone());
        self
    }

    pub fn excluding_tag(mut self, tag: &str) -> Self {
        self.exclude_tags.insert(tag.to_string());
        self
    }

    pub fn excluding_tags(mut self, tags: &Vec<String>) -> Self {
        self.exclude_tags.extend(tags.clone());
        self
    }

    pub fn with_ratio(mut self, ratio: AspectRatio) -> Self {
        self.ratio = ratio;
        self
    }

//impl<'a> PantsuTransaction<Vec<ImageHandle>> for SelectImagesTransaction<'a> {
    pub fn execute(self) -> Result<Vec<ImageHandle>> {
        let files = db_calls::get_files(self.connection, &Vec::from_iter(self.include_tags), &Vec::from_iter(self.exclude_tags))?;
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
    filename: Option<String>,
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

    pub fn for_image(mut self, filename: &str) -> Self {
        self.filename = Some(filename.to_string());
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
                db_calls::get_tags_for_file(self.connection, filename.as_str())
            } else {
                db_calls::get_tags_for_file_with_types(self.connection, filename.as_str(), &Vec::from_iter(self.types))
            },
            None => if self.types.len()==0 {
                db_calls::get_all_tags(self.connection)
            } else {
                db_calls::get_tags_with_types(self.connection, &Vec::from_iter(self.types))
            }
        }
    }
}