use std::collections::HashSet;
use std::iter::FromIterator;
use rusqlite::Connection;
use crate::db::db_calls;
use crate::error::Result;
use crate::{PantsuTag, Sauce};

pub struct UpdateImagesTransaction<'a> {
    connection: &'a mut Connection,
    filenames: HashSet<String>,
    sauce: Option<Sauce>,
    tags_to_add: HashSet<PantsuTag>,
    tags_to_remove: HashSet<String>,
}

impl<'a> UpdateImagesTransaction<'a> {
    pub(crate) fn new(connection: &'a mut Connection) -> Self {
        UpdateImagesTransaction {
            connection,
            filenames: HashSet::new(),
            sauce: None,
            tags_to_add: HashSet::new(),
            tags_to_remove: HashSet::new(),
        }
    }

    pub fn for_image(mut self, filename: &str) -> Self {
        self.filenames.insert(filename.to_string());
        self
    }

    pub fn for_images(mut self, filenames: &Vec<String>) -> Self {
        self.filenames.extend(filenames.clone());
        self
    }

    pub fn update_sauce(mut self, sauce: &Sauce) -> Self {
        self.sauce = Some(sauce.clone());
        self
    }

    pub fn add_tag(mut self, tag: &PantsuTag) -> Self {
        self.tags_to_add.insert(tag.clone());
        self
    }

    pub fn add_tags(mut self, tags: &Vec<PantsuTag>) -> Self {
        self.tags_to_add.extend(tags.clone());
        self
    }

    pub fn remove_tag(mut self, tag: &str) -> Self {
        self.tags_to_remove.insert(tag.to_string());
        self
    }

    pub fn remove_tags(mut self, tags: &Vec<String>) -> Self {
        self.tags_to_remove.extend(tags.clone());
        self
    }

//impl<'a> PantsuTransaction<()> for UpdateImagesTransaction<'a> {
    pub fn execute(self) -> Result<()> {
        if self.filenames.is_empty() {
            eprintln!("[UpdateImagesTransaction] warning: no files specified");
            return Ok(());
        }
        if self.sauce.is_none() && self.tags_to_add.is_empty() && self.tags_to_remove.is_empty() {
            eprintln!("[UpdateImagesTransaction] warning: no update operations");
            return Ok(());
        }
        let transaction = self.connection.transaction()?;
        let tags_to_add = Vec::from_iter(self.tags_to_add);
        let tags_to_remove = Vec::from_iter(self.tags_to_remove);

        for filename in self.filenames {
            if self.sauce.is_some() {
                db_calls::update_file_source(&transaction, filename.as_str(), self.sauce.as_ref().unwrap())?;
            }


            if !tags_to_remove.is_empty() {
                db_calls::remove_tags_from_file(&transaction, filename.as_str(), &tags_to_remove)?;
                db_calls::remove_unused_tags(&transaction)?;
            }
            if !tags_to_add.is_empty() {
                db_calls::add_tags_to_tag_list(&transaction, &tags_to_add)?;
                db_calls::add_tags_to_file(&transaction, filename.as_str(), &tags_to_add)?;
            }
        }
        transaction.commit()?;
        Ok(())
    }
}