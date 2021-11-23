use std::path::Path;
use pantsu_tags::db::PantsuDB;
use pantsu_tags::{Error, file_handler, PantsuTag, PantsuTagType};
use crate::cli::TagArgs;
use crate::common::AppResult;

pub fn tag(tag_args: TagArgs) -> AppResult<()> {
    match tag_args {
        TagArgs::List { tag_types } => tag_list(tag_types),
        TagArgs::Add { tags, image } => tag_add(tags, image),
        TagArgs::Rm { tags, image } => tag_rm(tags, image)
    }
}

fn tag_list(tag_types: Vec<PantsuTagType>) -> AppResult<()> {
    let db = PantsuDB::new(Path::new("./pantsu_tags.db"))?;
    let tags = if tag_types.len()==0 {
        db.get_all_tags()?
    } else {
        db.get_tags_with_types(&tag_types)?
    };
    for tag in tags {
        println!("{}", tag)
    }
    Ok(())
}

fn tag_add(tags: Vec<PantsuTag>, image: String) -> AppResult<()> {
    let mut db = PantsuDB::new(Path::new("./pantsu_tags.db"))?;
    let image = get_filename(image)?;
    let image =  db.get_file(&image)?
        .ok_or_else(|| Error::ImageNotFoundInDB(image))?;
    db.add_tags_to_file(&image, &tags)?;
    Ok(())
}

fn tag_rm(tags: Vec<String>, image: String) -> AppResult<()> {
    let mut db = PantsuDB::new(Path::new("./pantsu_tags.db"))?;
    let image = get_filename(image)?;
    let image =  db.get_file(&image)?
        .ok_or_else(|| Error::ImageNotFoundInDB(image))?;
    db.remove_tags(&image, &tags)?;
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