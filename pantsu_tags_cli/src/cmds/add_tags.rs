use std::path::{Path, PathBuf};
use pantsu_tags::db::PantsuDB;
use pantsu_tags::{Error, PantsuTag};
use crate::common::{AppResult, valid_filename_from_path};

pub fn add_tags(tags: Vec<PantsuTag>, images: Vec<PathBuf>) -> AppResult<()> {
    let mut db = PantsuDB::new(Path::new("./pantsu_tags.db"))?;
    for image in images {
        let image = valid_filename_from_path(image.as_path())?;
        let image = db.get_image_transaction(&image)
            .execute()?
            .ok_or_else(|| Error::ImageNotFoundInDB(image))?;
        db.update_images_transaction()
            .for_image(image.get_filename())
            .add_tags(&tags)
            .execute()?;
    }
    Ok(())
}