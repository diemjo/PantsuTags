use std::path::{PathBuf};
use std::str::FromStr;
use log::info;
use pantsu_tags::db::PantsuDB;
use pantsu_tags::{Error, PantsuTag};
use crate::common::{AppResult, self};
use crate::CONFIGURATION;

pub fn remove_tags(tags: Vec<String>, images: Vec<PathBuf>) -> AppResult<()> {
    let mut db = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    let tags = tags.iter()
        .map(|t| PantsuTag::from_str(t).or_else(|_| Ok(PantsuTag::new(t.to_string(), pantsu_tags::PantsuTagType::General))))
        .collect::<AppResult<Vec<PantsuTag>>>()?;
    for image in images {
        let image = common::image_handle_from_path(&image)?;
        let _ = db.get_image_transaction(&image)
            .execute()?
            .ok_or_else(|| Error::ImageNotFoundInDB(image.get_filename().to_string()))?;
        db.update_images_transaction()
            .for_image(&image)
            .remove_tags(&tags)
            .execute()?;
        info!("Removed tags {} from image: '{}'", PantsuTag::display_vec(&tags), image.get_filename());
    }
    Ok(())
}