use std::path::{PathBuf};
use log::{info, warn};
use pantsu_tags::db::PantsuDB;
use pantsu_tags::{Error, ImageHandle};
use crate::common::{AppResult, self};
use crate::CONFIGURATION;

pub fn remove_images(images: Vec<PathBuf>) -> AppResult<()> {
    let mut pdb = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    let images = images.into_iter()
        .map(|i| common::image_handle_from_path(&i))
        .collect::<AppResult<Vec<ImageHandle>>>()?;
    pdb.remove_image_transaction()
        .remove_images(&images)
        .execute()?;
    let lib_path = CONFIGURATION.library_path.as_path();
    for image in &images {
        let path = image.get_path(lib_path);
        if path.is_file() {
            std::fs::remove_file(path).or_else(|e|
                Err(Error::FileNotFound(e, image.get_filename().to_string()))
            )?;
            info!("Removed image: '{}'", image.get_filename());
        } else {
            println!("Attempting to remove image not in the library: '{}'", image.get_filename());
            warn!("Attempting to remove image not in the library: '{}'", image.get_filename());
        }
    }
    Ok(())
}