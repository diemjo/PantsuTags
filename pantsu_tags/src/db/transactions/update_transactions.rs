use std::collections::HashSet;
use std::iter::FromIterator;
use log::warn;
use rusqlite::Connection;
use crate::common::image_info::ImageInfo;
use crate::common::pantsu_tag::PantsuTagAuthor;
use crate::db::db_calls;
use crate::error::Result;
use crate::{PantsuTag, ImageHandle};
use crate::sauce::Sauce;

pub struct UpdateImagesTransaction<'a> {
    connection: &'a mut Connection,
    images: HashSet<&'a ImageHandle>,
    sauce: Option<&'a Sauce>,
    tags_to_add: HashSet<&'a PantsuTag>,
    tag_author: &'a PantsuTagAuthor,
    tags_to_remove: HashSet<&'a PantsuTag>,
}

impl<'a> UpdateImagesTransaction<'a> {
    pub(crate) fn new(connection: &'a mut Connection) -> Self {
        UpdateImagesTransaction {
            connection,
            images: HashSet::new(),
            sauce: None,
            tags_to_add: HashSet::new(),
            tag_author: &PantsuTagAuthor::User,
            tags_to_remove: HashSet::new(),
        }
    }

    pub fn for_image(mut self, image: &'a ImageHandle) -> Self {
        self.images.insert(image);
        self
    }

    pub fn for_images(mut self, images: &'a Vec<ImageHandle>) -> Self {
        self.images.extend(images);
        self
    }

    pub fn update_sauce(mut self, sauce: &'a Sauce) -> Self {
        self.sauce = Some(sauce);
        self
    }

    pub fn add_tag(mut self, tag: &'a PantsuTag) -> Self {
        self.tags_to_add.insert(tag);
        self
    }

    pub fn add_tags(mut self, tags: &'a Vec<PantsuTag>) -> Self {
        self.tags_to_add.extend(tags);
        self
    }

    pub fn tag_author(mut self, tag_author: &'a PantsuTagAuthor) -> Self {
        self.tag_author = tag_author;
        self
    }

    pub fn remove_tag(mut self, tag: &'a PantsuTag) -> Self {
        self.tags_to_remove.insert(tag);
        self
    }

    pub fn remove_tags(mut self, tags: &'a Vec<PantsuTag>) -> Self {
        self.tags_to_remove.extend(tags);
        self
    }

//impl<'a> PantsuTransaction<()> for UpdateImagesTransaction<'a> {
    pub fn execute(self) -> Result<u32> {
        if self.images.is_empty() {
            eprintln!("[UpdateImagesTransaction] warning: no images specified");
            warn!("Updating 0 images");
            return Ok(0);
        }
        if self.sauce.is_none() && self.tags_to_add.is_empty() && self.tags_to_remove.is_empty() {
            eprintln!("[UpdateImagesTransaction] warning: no update operations");
            warn!("No update operation specified");
            return Ok(0);
        }

        let tags_to_add = Vec::from_iter(self.tags_to_add.clone());
        let tags_to_remove = Vec::from_iter(self.tags_to_remove.clone());
        let images = Vec::from_iter(self.images.clone());

        let images = images.into_iter()
            .map(|i| Ok((i, db_calls::get_image(&self.connection, i)?)))
            .collect::<Result<Vec<(&ImageHandle, Option<ImageInfo>)>>>()?;

        let transaction = self.connection.transaction()?;
        let mut count = 0;
        for (arg_image, db_image) in images {
            if db_image.is_none() {
                warn!("Trying to update image not in database: {}", arg_image.get_filename());
                continue;
            }
            if self.sauce.is_some() {
                db_calls::update_image_source(&transaction, arg_image, self.sauce.unwrap())?;
            }
            if !tags_to_remove.is_empty() {
                db_calls::remove_tags_from_images(&transaction, arg_image, &tags_to_remove)?;
                db_calls::remove_unused_tags(&transaction)?;
            }
            if !tags_to_add.is_empty() {
                db_calls::add_tags_to_tag_list(&transaction, &tags_to_add)?;
                db_calls::add_tags_to_image(&transaction, arg_image, &tags_to_add, &self.tag_author)?;
            }
            db_calls::modify_image(&transaction, arg_image)?;
            count += 1;
        }
        transaction.commit()?;
        Ok(count)
    }
}