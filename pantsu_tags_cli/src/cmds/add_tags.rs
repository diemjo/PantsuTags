use std::path::{PathBuf};
use std::str::FromStr;
use log::info;
use pantsu_tags::db::PantsuDB;
use pantsu_tags::{Error, PantsuTag};
use crate::common::{AppResult};
use crate::common;
use crate::{CONFIGURATION};

pub fn add_tags(tags: Vec<String>, images: Vec<PathBuf>) -> AppResult<()> {
    let mut db = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    let tags = tags.iter()
        .map(|t| PantsuTag::from_str(t).or_else(|_| Ok(PantsuTag::new(t.to_string(), pantsu_tags::PantsuTagType::General))))
        .collect::<AppResult<Vec<PantsuTag>>>()?;
    for image in images {
        let image_handle = common::image_handle_from_path(image.as_path())?;
        let _ = db.get_image_transaction(&image_handle)
            .execute()?
            .ok_or_else(|| Error::ImageNotFoundInDB(image_handle.get_filename().to_string()))?;
        db.update_images_transaction()
            .for_image(&image_handle)
            .add_tags(&tags)
            .execute()?;
        info!("Added tags {} to image: '{}'", PantsuTag::display_vec(&tags), &image_handle.get_filename());
    }
    Ok(())
}