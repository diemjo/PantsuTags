use std::path::{PathBuf};

use colored::Colorize;

use pantsu_tags::{Error, PantsuTagType};
use pantsu_tags::db::PantsuDB;

use crate::common::{AppResult, valid_filename_from_path};
use crate::CONFIGURATION;

pub fn list_tags(images: Vec<PathBuf>, tag_types: Vec<PantsuTagType>, do_print_tagnames: bool) -> AppResult<()> {
    if images.len() == 0 {
        list_all_tags(tag_types, do_print_tagnames)?;
    } else {
        list_tags_for_images(images, tag_types, do_print_tagnames)?;
    }
    Ok(())
}

fn list_tags_for_images(images: Vec<PathBuf>, tag_types: Vec<PantsuTagType>, do_print_tagnames: bool) -> AppResult<()> {
    let db = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    let len = images.len();
    for (i, path) in images.into_iter().enumerate() {
        let image = valid_filename_from_path(&path)?;
        let image = db.get_image_transaction(&image)
            .execute()?
            .ok_or_else(|| Error::ImageNotFoundInDB(image))?;
        let tags = db.get_tags_transaction()
            .for_image(image.get_filename())
            .with_types(&tag_types)
            .execute()?;

        if len>1 {
            println!("{}:", image.get_filename().green());
        }
        for tag in tags {
            println!("{}", if do_print_tagnames { tag.tag_name } else { tag.to_string() } );
        }
        if i<len-1 {
            println!();
        }
    }
    Ok(())
}

fn list_all_tags(tag_types: Vec<PantsuTagType>, do_print_tagnames: bool) -> AppResult<()> {
    let db = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    let tags = db.get_tags_transaction()
        .with_types(&tag_types)
        .execute()?;
    for tag in tags {
        println!("{}", if do_print_tagnames { tag.tag_name } else { tag.to_string() } )
    }
    Ok(())
}

