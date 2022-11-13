use std::path::{PathBuf};
use log::{info, warn};
use pantsu_tags::db::PantsuDB;
use pantsu_tags::Error;
use crate::common::{AppResult, valid_filename_from_path};
use crate::CONFIGURATION;

pub fn remove_images(images: Vec<PathBuf>) -> AppResult<()> {
    let mut pdb = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    let images: Vec<String> = images.into_iter()
        .map(|i| valid_filename_from_path(&i))
        .collect::<AppResult<Vec<String>>>()?;
    pdb.remove_image_transaction()
        .remove_images(&images)
        .execute()?;
    let lib_path = CONFIGURATION.library_path.as_path();
    for image in &images {
        let path = lib_path.join(image);
        if path.is_file() {
            std::fs::remove_file(path).or_else(|e|
                Err(Error::FileNotFound(e, image.clone()))
            )?;
            info!("Removed image: '{}'", image);
        } else {
            println!("Attempting to remove image not in the library: '{}'", image);
            warn!("Attempting to remove image not in the library: '{}'", image);
        }
    }
    Ok(())
}