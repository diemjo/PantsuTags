use std::path::{Path, PathBuf};
use crate::common::error;
use crate::common::image_handle::ImageHandle;
use crate::db::PantsuDB;
use crate::file_handler::import;

pub use crate::common::error::Error;
pub use crate::common::error::Result;
pub use crate::common::image_file::ImageFile;
pub use crate::common::image_file::Sauce;
pub use crate::common::pantsu_tag::{PantsuTag, PantsuTagType};
pub use crate::sauce::SauceMatch;
pub use crate::sauce::get_thumbnail_link;

mod sauce;
mod common;
pub mod db;
pub mod file_handler;

pub fn new_image_handle(pantsu_db: &PantsuDB, image_path: &Path, error_on_similar: bool) -> Result<ImageHandle> {
    let image_name = file_handler::hash::calculate_filename(image_path)?;

    if pantsu_db.get_file(&image_name)?.is_some() {
        return Err(Error::ImageAlreadyExists(error::get_path(image_path)));
    }

    if error_on_similar {
        let similar = get_similar_images(&pantsu_db, &image_name, 10)?;
        if similar.len()>0 {
            return Err(Error::SimilarImagesExist(image_name, similar))
        }
    }

    import::import_file("./test_image_lib/", image_path, &image_name)?;
    Ok(ImageHandle::new(String::from(image_name)))
}

pub fn get_image_sauces(image: &ImageHandle) -> Result<Vec<SauceMatch>> {
    let image_path = PathBuf::from(format!("./test_image_lib/{}", image.get_filename()));
    let mut sauce_matches = sauce::find_sauce(&image_path)?;
    sauce_matches.sort();
    sauce_matches.reverse();
    Ok(sauce_matches)
}

pub fn get_sauce_tags(sauce: &SauceMatch) -> Result<Vec<PantsuTag>> {
    sauce::find_tags_gelbooru(&sauce.link)
}

pub fn store_image_with_tags(pantsu_db: &mut PantsuDB, image: &ImageHandle, sauce: Sauce, tags: &Vec<PantsuTag>) -> Result<()> {
    pantsu_db.add_file_and_tags(
        &ImageFile {
            filename: String::from(image.get_filename()),
            file_source: sauce
        },
        tags
    )
}

fn get_similar_images(pantsu_db: &PantsuDB, file_name: &String, min_dist: u32) -> Result<Vec<String>> {
    let files = pantsu_db.get_all_files()?;
    Ok(file_handler::hash::get_similarity_distances(file_name, files, min_dist))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::path::PathBuf;
    use crate::{ImageFile, PantsuDB, PantsuTag, Sauce};
    use crate::file_handler::hash;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_add_image() {
        let mut db_path = std::env::current_dir().unwrap();
        db_path.push("pantsu_tags.db");
        let mut pdb = PantsuDB::new(&db_path).unwrap();
        let image_path = prepare_image("https://img1.gelbooru.com/images/4f/76/4f76b8d52983af1d28b1bf8d830d684e.png");

        let image_handle = crate::new_image_handle(&pdb, &image_path, false).unwrap();
        let sauces = crate::get_image_sauces(&image_handle).unwrap();
        let best_match = &sauces[0];
        // in general, you would want to check the similarity here
        let tags = crate::get_sauce_tags(&best_match).unwrap();
        crate::store_image_with_tags(&mut pdb, &image_handle, Sauce::Match(best_match.link.clone()), &tags).unwrap();
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
            &ImageFile { filename: image_name, file_source: Sauce::NotChecked },
            &vec![PantsuTag{tag_name: String::from("Hehe"), tag_type: "general".parse().unwrap()}]
        ).unwrap();
        pdb.add_file_and_tags(
            &ImageFile { filename: not_similar_image_name, file_source: Sauce::NotChecked },
            &vec![PantsuTag{tag_name: String::from("Hehe"), tag_type: "general".parse().unwrap()}]
        ).unwrap();
        let similar_images = crate::get_similar_images(&pdb, &similar_image_name, 10).unwrap();
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