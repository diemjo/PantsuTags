use std::path::Path;
use pantsu_tags::db::PantsuDB;
use pantsu_tags::{Error, PantsuTag, PantsuTagType};
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

fn tag_add(_tags: Vec<PantsuTag>, image: String) -> AppResult<()> {
    let db = PantsuDB::new(Path::new("./pantsu_tags.db"))?;
    if db.get_file(&image)?.is_none() {
        Err(Error::ImageNotFoundInDB(image))?
    }
    Ok(())
}

fn tag_rm(_tags: Vec<String>, _image: String) -> AppResult<()> {
    let db = PantsuDB::new(Path::new("./pantsu_tags.db"))?;
    Ok(())
}