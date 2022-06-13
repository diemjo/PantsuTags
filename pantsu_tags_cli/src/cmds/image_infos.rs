use std::path::{Path, PathBuf};
use colored::{Colorize};
use pantsu_tags::db::PantsuDB;
use crate::common::{AppResult, valid_filename_from_path};

pub fn image_infos(images: Vec<PathBuf>) -> AppResult<()> {
    let pdb_path = Path::new("./pantsu_tags.db");
    let pdb = PantsuDB::new(pdb_path)?;
    let images: Vec<String> = images.into_iter()
        .map(|i| valid_filename_from_path(i.as_path()))
        .collect::<AppResult<Vec<String>>>()?;
    if images.is_empty() {
        let images = pdb.get_images_transaction()
            .execute()?;
        for image in images {
            println!("{}", &image);
        }
    } else {
        for image in images {
            let db_image = pdb.get_image_transaction(image.as_str()).execute()?;
            match db_image {
                Some(img) => println!("{}", img),
                None => eprintln!("{}: no such image in database", image.red())
            }
        }
    }
    Ok(())
}