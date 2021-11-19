use std::path::{Path, PathBuf};
use crate::common::error;
use crate::common::image_handle::ImageHandle;
use crate::common::image_file::ImageFile;
use crate::common::pantsu_tag::PantsuTag;
use crate::db::PantsuDB;
use crate::error::Error;
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
impl SauceQuality {
    fn get_quality(similarity: i32) -> Self {
        if similarity >= SIMILARITY_GOOD {
            SauceQuality::Found
        } else if similarity >= SIMILARITY_UNSURE {
            SauceQuality::Unsure
        } else {
            SauceQuality::NotFound
        }
    }
}

pub fn new_image_handle(pantsu_db: &PantsuDB, image_path: &Path, error_on_similar: bool) -> Result<ImageHandle, Error> {
    let image_name = file_handler::hash::calculate_filename(image_path)?;

    if pantsu_db.get_file(&image_name)?.is_some() {
        return Err(Error::ImageAlreadyExists(error::get_path(image_path)));
    }

    if error_on_similar {
        let similar = get_similar_images(&pantsu_db, &image_name, 10)?;
        if similar.len()>0 {
            return Err(Error::SimilarImagesExist(similar))
        }
    }

    import::import_file("./test_image_lib/", image_path, &image_name)?;
    Ok(ImageHandle::new(String::from(image_name)))
}

pub fn get_image_sauces(image: &ImageHandle) -> Result<(SauceQuality, Vec<SauceMatch>), Error> {
    let image_path = PathBuf::from(format!("./test_image_lib/{}", image.get_filename()));
    let mut sauce_matches = sauce_finder::find_sauce(&image_path)?;
    sauce_matches.sort();
    sauce_matches.reverse();
    let best_match = &sauce_matches[0];
    let quality = SauceQuality::get_quality(best_match.similarity);
    Ok((quality, sauce_matches))
}

pub fn get_sauce_tags(sauce: &SauceMatch) -> Result<Vec<PantsuTag>, Error> {
    tag_finder::find_tags_gelbooru(&sauce.link)
}

pub fn store_image_with_tags_from_sauce(pantsu_db: &mut PantsuDB, image: &ImageHandle, sauce: &SauceMatch, tags: &Vec<PantsuTag>) -> Result<(), Error> {
    pantsu_db.add_file_and_tags(
        &ImageFile {
            filename: String::from(image.get_filename()),
            file_source: Some(String::from(&sauce.link)),
        },
        tags
    )
}

pub fn store_image_with_tags(pantsu_db: &mut PantsuDB, image: &ImageHandle, tags: &Vec<PantsuTag>) -> Result<(), Error> {
    pantsu_db.add_file_and_tags(
        &ImageFile {
            filename: String::from(image.get_filename()),
            file_source: None,
        },
        tags
    )
}

fn get_similar_images(pantsu_db: &PantsuDB, file_name: &String, min_dist: u32) -> Result<Vec<String>, Error> {
    let files = pantsu_db.get_all_files()?;
    Ok(file_handler::hash::get_similarity_distances(file_name, files, min_dist))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::path::PathBuf;
    use crate::{get_image_sauces, get_sauce_tags, get_similar_images, ImageFile, new_image_handle, PantsuDB, PantsuTag, SIMILARITY_GOOD, store_image_with_tags_from_sauce};
    use crate::file_handler::hash;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_add_image() {
        let mut db_path = std::env::current_dir().unwrap();
        db_path.push("pantsu_tags.db");
        let mut pdb = PantsuDB::new(&db_path).unwrap();
        let image_path = prepare_image("https://img1.gelbooru.com/images/4f/76/4f76b8d52983af1d28b1bf8d830d684e.png");

        let image_handle = new_image_handle(&pdb, &image_path, false).unwrap();
        let sauces = get_image_sauces(&image_handle).unwrap();
        let best_match = &sauces.1[0];
        assert!(best_match.similarity > SIMILARITY_GOOD);
        let tags = get_sauce_tags(&best_match).unwrap();
        store_image_with_tags_from_sauce(&mut pdb, &image_handle, &best_match, &tags).unwrap();
    }

    #[test]
    #[serial]
    fn test_similar_images() {
        let image_path = prepare_image("https://img1.gelbooru.com/images/4f/76/4f76b8d52983af1d28b1bf8d830d684e.png");
        let image_name = hash::calculate_filename(&image_path).unwrap();
        let similar_image_path = prepare_image("https://img1.gelbooru.com/images/22/3a/223a6444a6e79ecb273896cfccee9850.png");
        let similar_image_name = hash::calculate_filename(&similar_image_path).unwrap();
        let not_similar_image_path = prepare_image("https://img3.gelbooru.com/images/1d/b8/1db8ab6c94e95f36a9dd5bde347f6dd1.png");
        let not_similar_image_name = hash::calculate_filename(&not_similar_image_path).unwrap();
        let mut db_path = std::env::current_dir().unwrap();
        db_path.push("pantsu_tags.db");
        let mut pdb = PantsuDB::new(&db_path).unwrap();
        pdb.clear().unwrap();
        pdb.add_file_and_tags(
            &ImageFile { filename: image_name, file_source: None },
            &vec![PantsuTag{tag_name: String::from("Hehe"), tag_type: "general".parse().unwrap()}]
        ).unwrap();
        pdb.add_file_and_tags(
            &ImageFile { filename: not_similar_image_name, file_source: None },
            &vec![PantsuTag{tag_name: String::from("Hehe"), tag_type: "general".parse().unwrap()}]
        ).unwrap();
        let similar_images = get_similar_images(&pdb, &similar_image_name, 10).unwrap();
        assert_eq!(similar_images.len(), 1);
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