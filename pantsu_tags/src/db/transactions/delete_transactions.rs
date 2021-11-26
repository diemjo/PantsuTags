use rusqlite::{Transaction};
use crate::{Error, ImageHandle};
use crate::common::error::Result;
use crate::db::db_calls;
use crate::db::transactions::{PantsuDBTransaction, RemoveImageTransaction, RemoveTagsTransaction};

// rm image
impl RemoveImageTransaction {
    pub fn new(image: ImageHandle) -> Self {
        RemoveImageTransaction {
            image
        }
    }
}

impl PantsuDBTransaction<()> for RemoveImageTransaction {
    fn execute(self, transaction: &mut Transaction) -> Result<()> {
        db_calls::remove_file_from_file_list(&transaction, &self.image)?;
        db_calls::remove_all_tags_from_file(&transaction, &self.image)?;
        db_calls::remove_unused_tags(&transaction)?;
        Ok(())
    }
}

// rm tags
impl<'t, 'p> RemoveTagsTransaction<'t, 'p> {
    pub fn new(image: &'t ImageHandle) -> Self {
        RemoveTagsTransaction {
            image,
            tags: Vec::new(),
            remove_all: false
        }
    }

    pub fn removing_all(mut self) -> Self {
        self.remove_all = true;
        self
    }

    pub fn with_tags(mut self, tags: &'p Vec<String>) -> Self {
        for tag in tags {
            self.tags.push(tag);
        }
        self
    }
}

impl<'t, 'p> PantsuDBTransaction<()> for RemoveTagsTransaction<'t, 'p> {
    fn execute(self, transaction: &mut Transaction) -> Result<()> {
        if !self.remove_all && self.tags.len()==0 {
            return Err(Error::NoTagProvided)
        }
        if self.remove_all {
            db_calls::remove_all_tags_from_file(&transaction, self.image)?;
        } else {
            db_calls::remove_tags_from_file(&transaction, self.image, self.tags)?;
        }
        Ok(())
    }
}