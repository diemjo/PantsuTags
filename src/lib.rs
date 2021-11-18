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

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::path::PathBuf;
    use crate::{add_image, PantsuDB};

    #[test]
    fn test_add_image() {
        let mut db_path = std::env::current_dir().unwrap();
        db_path.push("pantsu_tags.db");
        let mut pdb = PantsuDB::new(&db_path).unwrap();
        let image_path = prepare_image("https://img1.gelbooru.com/images/4f/76/4f76b8d52983af1d28b1bf8d830d684e.png");
        add_image(&mut pdb, image_path.as_path()).unwrap();
    }

    fn prepare_image(image_link: &str) -> PathBuf {
        let image_name = image_link.rsplit('/').next().unwrap();
        let image_path = PathBuf::from(format!("test_image_{}", image_name));
        if image_path.exists() {
            return image_path;
        }

        let response = reqwest::blocking::get(image_link).unwrap();
        let mut file = std::fs::File::create(&image_path).unwrap();
        let mut content =  Cursor::new(response.bytes().unwrap());
        std::io::copy(&mut content, &mut file).unwrap();
        image_path
    }
}