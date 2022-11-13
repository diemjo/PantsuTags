use std::path::{PathBuf};
use log::info;
use pantsu_tags::db::PantsuDB;
use pantsu_tags::Error;
use crate::common::{AppResult, valid_filename_from_path};
use crate::CONFIGURATION;

pub fn remove_tags(tags: Vec<String>, images: Vec<PathBuf>) -> AppResult<()> {
    let mut db = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    for image in images {
        let image_name = valid_filename_from_path(&image)?;
        let image = db.get_image_transaction(&image_name)
            .execute()?
            .ok_or_else(|| Error::ImageNotFoundInDB(image_name.clone()))?;
        db.update_images_transaction()
            .for_image(image.get_filename())
            .remove_tags(&tags)
            .execute()?;
        info!("Removed tags [{}] from image: '{}'", tags.join(", "), &image_name);
    }
    Ok(())
}