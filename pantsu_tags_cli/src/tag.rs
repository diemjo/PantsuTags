use std::path::Path;
use colored::Colorize;
use pantsu_tags::db::PantsuDB;
use pantsu_tags::{Error, file_handler, PantsuTag, PantsuTagType};
use pantsu_tags::db::transactions::{AddTagsTransaction, GetImageTransaction, GetTagsTransaction, RemoveTagsTransaction};
use crate::common::AppResult;

pub fn tag_list(tag_types: Vec<PantsuTagType>) -> AppResult<()> {
    let mut db = PantsuDB::new(Path::new("./pantsu_tags.db"))?;
    let tags = db.execute(GetTagsTransaction::new().with_types(&tag_types))?;
    for tag in tags {
        println!("{}", tag)
    }
    Ok(())
}

pub fn tag_add(tags: Vec<PantsuTag>, image: String) -> AppResult<()> {
    let mut db = PantsuDB::new(Path::new("./pantsu_tags.db"))?;
    let image = get_filename(image)?;
    let image = db.execute(GetImageTransaction::new(&image))?
        .ok_or_else(|| Error::ImageNotFoundInDB(image))?;
    db.execute(
        AddTagsTransaction::new(&image)
            .with_tags(&tags)
    )?;
    Ok(())
}

pub fn tag_rm(tags: Vec<String>, image: String) -> AppResult<()> {
    let mut db = PantsuDB::new(Path::new("./pantsu_tags.db"))?;
    let image = get_filename(image)?;
    let image =  db.execute(GetImageTransaction::new(&image))?
        .ok_or_else(|| Error::ImageNotFoundInDB(image))?;
    db.execute(
        RemoveTagsTransaction::new(&image)
            .with_tags(&tags)
    )?;
    Ok(())
}

pub fn tag_get(images: Vec<String>) -> AppResult<()> {
    if images.len()==0 {
        eprintln!("Warning: No image was provided");
    }
    let len = images.len();
    let mut db = PantsuDB::new(Path::new("./pantsu_tags.db"))?;
    for (i, name) in images.into_iter().enumerate() {
        let image = get_filename(name)?;
        let image = db.execute(GetImageTransaction::new(&image))?
            .ok_or_else(|| Error::ImageNotFoundInDB(image))?;
        let tags = db.execute(GetTagsTransaction::new().only_for_image(&image))?;

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