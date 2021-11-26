use rusqlite::{Transaction};
use crate::{ImageHandle, PantsuTag, Sauce};
use crate::common::error::Result;
use crate::db::db_calls;
use crate::db::transactions::{AddImageTransaction, AddTagsTransaction, PantsuDBTransaction};

// INSERT IMAGE ##########################################
impl<'t> AddImageTransaction<'t> {
    pub fn new(image: &'t ImageHandle) -> Self {
        AddImageTransaction {
            image
        }
    }
}

impl<'t> PantsuDBTransaction<()> for AddImageTransaction<'t> {
    fn execute(self, transaction: &mut Transaction) -> Result<()> {
        db_calls::add_file_to_file_list(&transaction, self.image)?;
        Ok(())
    }
}

// INSERT TAG ###########################################
impl<'t, 'p> AddTagsTransaction<'t, 'p> {
    pub fn new(image: &'t ImageHandle) -> Self {
        AddTagsTransaction {
            image,
            tags: Vec::new(),
            update_source: None
        }
    }

    pub fn with_tags(mut self, tags: &'p Vec<PantsuTag>) -> Self {
        for tag in tags {
            self.tags.push(tag);
        }
        self
    }

    pub fn update_image_source(mut self, sauce: Sauce) -> Self {
        self.update_source = Some(sauce);
        self
    }
}

impl<'t, 'p> PantsuDBTransaction<()> for AddTagsTransaction<'t, 'p> {
    fn execute(self, transaction: &mut Transaction) -> Result<()> {
        match self.update_source {
            None => {
                db_calls::add_tags_to_tag_list(transaction, &self.tags)?;
                db_calls::add_tags_to_file_tags(&transaction, self.image, &self.tags)?;
            }
            Some(sauce) => {
                let handle = self.image.clone_with_sauce(sauce);
                db_calls::update_file_info(transaction, &handle)?;
                db_calls::add_tags_to_tag_list(transaction, &self.tags)?;
                db_calls::add_tags_to_file_tags(&transaction, &handle, &self.tags)?;
            }
        };
        Ok(())
    }
}