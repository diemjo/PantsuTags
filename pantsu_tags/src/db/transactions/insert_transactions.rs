use rusqlite::Connection;
use crate::db::db_calls;
use crate::{ImageHandle};
use crate::error::{Result};

pub struct InsertImagesTransaction<'a> {
    connection: &'a mut Connection,
    images: Vec<(&'a ImageHandle, (u32, u32))>,
}

impl<'a> InsertImagesTransaction<'a> {
    pub(crate) fn new(connection: &'a mut Connection) -> Self {
        InsertImagesTransaction {
            connection,
            images: vec![],
        }
    }

    pub fn add_image(mut self, image: &'a ImageHandle, res: (u32, u32)) -> Self {
        self.images.push((image, res));
        self
    }

    pub fn execute(self) -> Result<u32> {
        let transaction = self.connection.transaction()?;
        let mut count = 0;
        for (image, res) in self.images {
            db_calls::add_image_to_images(&transaction, &image, res)?;
            count += 1;
        }
        transaction.commit()?;
        Ok(count)
    }
}