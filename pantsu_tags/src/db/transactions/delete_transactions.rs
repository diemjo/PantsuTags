use std::collections::HashSet;
use log::warn;
use rusqlite::Connection;
use crate::ImageHandle;
use crate::common::image_info::ImageInfo;
use crate::db::db_calls;
use crate::error::{Result};

pub struct DeleteImagesTransaction<'a> {
    connection: &'a mut Connection,
    images: HashSet<&'a ImageHandle>,
}

impl<'a> DeleteImagesTransaction<'a> {
    pub(crate) fn new(connection: &'a mut Connection) -> Self {
        DeleteImagesTransaction {
            connection,
            images: HashSet::new(),
        }
    }

    pub fn remove_image(mut self, image: &'a ImageHandle) -> Self {
        self.images.insert(image);
        self
    }

    pub fn remove_images(mut self, images: &'a Vec<ImageHandle>) -> Self {
        self.images.extend(images);
        self
    }

//impl<'a> PantsuTransaction<()> for DeleteImagesTransaction<'a> {
    pub fn execute(self) -> Result<u32> {
        let images = self.images.iter()
            .map(|&i| Ok((i, db_calls::get_image(&self.connection, &i)?)))
            .collect::<Result<Vec<(&ImageHandle, Option<ImageInfo>)>>>()?;
        let transaction = self.connection.transaction()?;
        let mut count = 0;
        for (arg_image, db_image) in images {
            match db_image {
                Some(_) => {
                    db_calls::remove_all_tags_from_image(&transaction, arg_image)?;
                    db_calls::remove_image_from_images(&transaction, arg_image)?;
                    count += 1;
                },
                None => {
                    warn!("Cannot remove image not in database: {}", arg_image.get_filename());
                }
            }
            
        }
        db_calls::remove_unused_tags(&transaction)?;
        transaction.commit()?;
        Ok(count)
    }
}