use rusqlite::{Transaction};
use crate::{ImageHandle, Sauce};
use crate::common::error::Result;
use crate::db::db_calls;
use crate::db::transactions::{PantsuDBTransaction, UpdateImageTransaction};

impl UpdateImageTransaction {
    pub fn new(image: ImageHandle) -> Self {
        UpdateImageTransaction {
            image
        }
    }

    pub fn with_sauce(mut self, sauce: Sauce) -> Self {
        self.image = self.image.clone_with_sauce(sauce);
        self
    }
}

impl PantsuDBTransaction<ImageHandle> for UpdateImageTransaction {
    fn execute(self, transaction: &mut Transaction) -> Result<ImageHandle> {
        db_calls::update_file_info(&transaction, &self.image)?;
        Ok(self.image)
    }
}