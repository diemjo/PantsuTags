use std::collections::HashSet;
use rusqlite::Connection;
use crate::db::db_calls;
use crate::error::{Result};

pub struct DeleteImagesTransaction<'a> {
    connection: &'a mut Connection,
    filenames: HashSet<&'a str>,
}

impl<'a> DeleteImagesTransaction<'a> {
    pub(crate) fn new(connection: &'a mut Connection) -> Self {
        DeleteImagesTransaction {
            connection,
            filenames: HashSet::new(),
        }
    }

    pub fn remove_image(mut self, filename: &'a str) -> Self {
        self.filenames.insert(filename);
        self
    }

    pub fn remove_images(mut self, filenames: &'a Vec<String>) -> Self {
        let filenames : Vec<&str> = filenames.iter()
            .map(|f|f.as_str())
            .collect();
        self.filenames.extend(filenames);
        self
    }

//impl<'a> PantsuTransaction<()> for DeleteImagesTransaction<'a> {
    pub fn execute(self) -> Result<()> {
        let transaction = self.connection.transaction()?;
        for filename in self.filenames {
            db_calls::remove_all_tags_from_file(&transaction, filename)?;
            db_calls::remove_file_from_file_list(&transaction, filename)?;
        }
        db_calls::remove_unused_tags(&transaction)?;
        transaction.commit()?;
        Ok(())
    }
}