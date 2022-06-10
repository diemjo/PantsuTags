use std::collections::HashSet;
use rusqlite::Connection;
use crate::db::db_calls;
use crate::error::{Result};

pub struct DeleteImagesTransaction<'a> {
    connection: &'a mut Connection,
    filenames: HashSet<String>,
}

impl<'a> DeleteImagesTransaction<'a> {
    pub(crate) fn new(connection: &'a mut Connection) -> Self {
        DeleteImagesTransaction {
            connection,
            filenames: HashSet::new(),
        }
    }

    pub fn remove_image(mut self, filename: &str) -> Self {
        self.filenames.insert(filename.to_string());
        self
    }

    pub fn remove_images(mut self, filenames: &Vec<String>) -> Self {
        self.filenames.extend(filenames.clone());
        self
    }

//impl<'a> PantsuTransaction<()> for DeleteImagesTransaction<'a> {
    pub fn execute(self) -> Result<()> {
        let transaction = self.connection.transaction()?;
        for filename in self.filenames {
            db_calls::remove_all_tags_from_file(&transaction, filename.as_str())?;
            db_calls::remove_file_from_file_list(&transaction, filename.as_str())?;
        }
        db_calls::remove_unused_tags(&transaction)?;
        transaction.commit()?;
        Ok(())
    }
}