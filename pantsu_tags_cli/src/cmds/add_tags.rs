use std::path::{PathBuf};
use log::info;
use pantsu_tags::db::PantsuDB;
use pantsu_tags::{Error, PantsuTag};
use crate::common::{AppResult, valid_filename_from_path};
use crate::{CONFIGURATION};

pub fn add_tags(tags: Vec<PantsuTag>, images: Vec<PathBuf>) -> AppResult<()> {
    let mut db = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    for image in images {
        let image_name = valid_filename_from_path(image.as_path())?;
        let image_handle = db.get_image_transaction(&image_name)
            .execute()?
            .ok_or_else(|| Error::ImageNotFoundInDB(image_name.clone()))?;
        db.update_images_transaction()
            .for_image(image_handle.get_filename())
            .add_tags(&tags)
            .execute()?;
        info!("Added tags {} to image: '{}'", PantsuTag::vec_to_string(&tags), &image_name);
    }
    Ok(())
}