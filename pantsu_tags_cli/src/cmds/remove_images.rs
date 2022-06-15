use std::path::{PathBuf};
use pantsu_tags::db::PantsuDB;
use crate::common::{AppResult, valid_filename_from_path};
use crate::CONFIGURATION;

pub fn remove_images(images: Vec<PathBuf>) -> AppResult<()> {
    let mut pdb = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    let images: Vec<String> = images.into_iter()
        .map(|i| valid_filename_from_path(i.as_path()))
        .collect::<AppResult<Vec<String>>>()?;
    pdb.remove_image_transaction()
        .remove_images(&images)
        .execute()?;
    Ok(())
}