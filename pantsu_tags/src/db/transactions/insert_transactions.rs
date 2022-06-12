use rusqlite::Connection;
use crate::db::db_calls;
use crate::{file_handler, ImageHandle};
use crate::error::{Result, Error};

pub struct InsertImagesTransaction<'a> {
    connection: &'a mut Connection,
    images: Vec<&'a ImageHandle>,
}

impl<'a> InsertImagesTransaction<'a> {
    pub(crate) fn new(connection: &'a mut Connection) -> Self {
        InsertImagesTransaction {
            connection,
            images: vec![],
        }
    }

    pub fn add_image(mut self, image: &'a ImageHandle) -> Self {
        self.images.push(image);
        self
    }

    pub fn add_images(mut self, images: &'a Vec<ImageHandle>) -> Self {
        self.images.extend(images);
        self
    }

//impl<'a> PantsuTransaction<()> for InsertImagesTransaction<'a> {
    pub fn execute(self) -> Result<()> {
        let transaction = self.connection.transaction()?;
        for image in self.images {
            if !file_handler::filename_is_valid(image.get_filename()) {
                return Err(Error::InvalidFilename(String::from(image.get_filename())))
            }
            db_calls::add_file_to_file_list(&transaction, &image)?;
        }
        transaction.commit()?;
        Ok(())
    }
}