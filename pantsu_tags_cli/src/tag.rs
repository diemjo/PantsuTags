use std::path::{Path};
use colored::Colorize;
use pantsu_tags::db::PantsuDB;
use pantsu_tags::{Error, file_handler, PantsuTag, PantsuTagType};
use crate::common::AppResult;

pub fn tag_list(tag_types: Vec<PantsuTagType>) -> AppResult<()> {
    let db = PantsuDB::new(Path::new("./pantsu_tags.db"))?;
    let tags = db.get_tags_transaction()
            .with_types(&tag_types)
            .execute()?;
    for tag in tags {
        println!("{}", tag)
    }
    Ok(())
}

pub fn tag_add(tags: Vec<PantsuTag>, images: Vec<String>) -> AppResult<()> {
    let mut db = PantsuDB::new(Path::new("./pantsu_tags.db"))?;
    for image in images {
        let image = get_filename(image)?;
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

pub fn tag_rm(tags: Vec<String>, images: Vec<String>) -> AppResult<()> {
    let mut db = PantsuDB::new(Path::new("./pantsu_tags.db"))?;
    for image in images {
        let image = get_filename(image)?;
        let image = db.get_image_transaction(&image)
            .execute()?
            .ok_or_else(|| Error::ImageNotFoundInDB(image))?;
        db.update_images_transaction()
            .for_image(image.get_filename())
            .remove_tags(&tags)
            .execute()?;
    }
    Ok(())
}

pub fn tag_get(images: Vec<String>, tag_types: Vec<PantsuTagType>) -> AppResult<()> {
    let len = images.len();
    if images.len()==0 {
        eprintln!("Warning: No image was provided");
    }
    let db = PantsuDB::new(Path::new("./pantsu_tags.db"))?;
    for (i, name) in images.into_iter().enumerate() {
        let image = get_filename(name)?;
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
            println!("{}", tag.to_string());
        }
        if i<len-1 {
            println!();
        }
    }
    Ok(())
}

fn get_filename(image: String) -> AppResult<String> {
    let filename = Path::new(&image)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(Error::InvalidFilename(image.clone()))?;
    if !file_handler::filename_is_valid(filename) {
        Err(Error::InvalidFilename(image.clone()))?;
    }
    Ok(String::from(filename))
}