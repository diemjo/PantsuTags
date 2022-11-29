use std::path::{PathBuf};
use colored::{Colorize};
use pantsu_tags::ImageHandle;
use pantsu_tags::db::PantsuDB;
use crate::common::{AppResult, self, parse_image_sort_order};
use crate::CONFIGURATION;

pub fn image_infos(images: Vec<PathBuf>, sort_order: Vec<String>) -> AppResult<()> {
    let pdb = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    let sort_order = parse_image_sort_order(sort_order)?;
    let images = images.into_iter()
        .map(|i| common::image_handle_from_path(i.as_path()))
        .collect::<AppResult<Vec<ImageHandle>>>()?;
    if images.is_empty() {
        let images_transaction = pdb.get_images_transaction();
        let images = match sort_order {
            Some(order) => images_transaction.sort_by(&order).execute()?,
            None => images_transaction.execute()?
        };
        for image in images {
            println!("{}", &image);
        }
    } else {
        for image in images {
            let db_image = pdb.get_image_transaction(&image).execute()?;
            match db_image {
                Some(img) => println!("{}", &img),
                None => eprintln!("{}: no such image in database", image.get_filename().red())
            }
        }
    }
    Ok(())
}