use std::path::{PathBuf};

use colored::Colorize;

use pantsu_tags::{Error, PantsuTagType};
use pantsu_tags::db::PantsuDB;

use crate::common::{AppResult, self};
use crate::CONFIGURATION;

pub fn list_tags(images: Vec<PathBuf>, tag_types: Vec<PantsuTagType>, sort_order: Vec<String>, do_print_tagnames: bool) -> AppResult<()> {
    if images.len() == 0 {
        list_all_tags(tag_types, do_print_tagnames)?;
    } else {
        list_tags_for_images(images, tag_types, sort_order, do_print_tagnames)?;
    }
    Ok(())
}

fn list_tags_for_images(images: Vec<PathBuf>, tag_types: Vec<PantsuTagType>, sort_order: Vec<String>, do_print_tagnames: bool) -> AppResult<()> {
    let db = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    let len = images.len();
    let sort_order = common::parse_tag_sort_order(sort_order)?;

    for (i, path) in images.into_iter().enumerate() {
        let image = common::image_handle_from_path(&path)?;
        let _ = db.get_image_transaction(&image)
            .execute()?
            .ok_or_else(|| Error::ImageNotFoundInDB(image.get_filename().to_string()))?;
        let transaction = db.get_image_tags_transaction(&image)
            .with_types(&tag_types);
        let tags = match &sort_order {
            Some(order) => transaction.sort_by(order).execute()?,
            None => transaction.execute()?
        };

        if len>1 {
            println!("{}:", image.get_filename().green());
        }
        for tag in tags {
            println!("{}", if do_print_tagnames { tag.tag.tag_name } else { tag.tag.to_string() } );
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

