use std::path::{Path, PathBuf};
use pantsu_tags::db::PantsuDB;
use crate::common::{AppResult, valid_filename_from_path};

pub fn remove_images(images: Vec<PathBuf>) -> AppResult<()> {
    let pdb_path = Path::new("./pantsu_tags.db");
    let mut pdb = PantsuDB::new(pdb_path)?;
    let images: Vec<String> = images.into_iter()
        .map(|i| valid_filename_from_path(i.as_path()))
        .collect::<AppResult<Vec<String>>>()?;
    pdb.remove_image_transaction()
        .remove_images(&images)
        .execute()?;
    Ok(())
}