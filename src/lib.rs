use std::path::Path;
use crate::common::error::Error;
use crate::common::file_handle::FileHandle;
use crate::common::image_file::ImageFile;
use crate::common::pantsu_tag::PantsuTag;
use crate::db::PantsuDB;
use crate::file_handler::import;
use crate::sauce::{sauce_finder, SauceMatch, tag_finder};

pub mod sauce;
mod common;
pub mod db;
pub mod file_handler;

const SIMILARITY_GOOD: i32 = 80;
const SIMILARITY_UNSURE: i32 = 50;

pub enum SauceQuality {
    Found,
    Unsure,
    NotFound,
}

pub fn get_file_handle() -> FileHandle {

    FileHandle::new(String::from(""))
}

pub fn add_image(pantsu_db: &mut PantsuDB, image_path: &Path) -> Result<(SauceQuality, Vec<SauceMatch>), Error>{
    // opt: check whether file is image

    // file_handler: get file name
    let image_name = file_handler::hash::calculate_filename(image_path)?;

    // file_handler: check whether file already exists
    if pantsu_db.get_file(&image_name)?.is_none() {
        import::import_file("./test_image_lib/", image_path, &image_name)?;
    }

    let mut sauce_matches = sauce_finder::find_sauce(image_path)?;
    sauce_matches.sort();
    let best_match = &sauce_matches[0];
    return if best_match.similarity >= SIMILARITY_GOOD {
        let tags = tag_finder::find_tags_gelbooru(&best_match.link)?;
        // create img in db

        add_tags_to_image(pantsu_db, &image_name, &best_match.link, &tags)?;
        Ok((SauceQuality::Found, sauce_matches))
    } else if best_match.similarity >= SIMILARITY_UNSURE {
        Ok((SauceQuality::Unsure, sauce_matches))
    } else {
        Ok((SauceQuality::NotFound, sauce_matches))
    }
}

pub fn add_tags_to_image(pantsu_db: &mut PantsuDB, image_name: &str, source: &str, tags: &Vec<PantsuTag>) -> Result<(), Error> {
    pantsu_db.add_file_and_tags(&ImageFile { filename: String::from(image_name), file_source: Some(String::from(source))}, tags)?;
    Ok(())
}