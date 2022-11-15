use std::path::{Path, PathBuf};
use crate::common::error;
use crate::db::{PantsuDB};
use crate::file_handler::import;

pub use crate::common::error::Error;
pub use crate::common::error::Result;
pub use crate::common::image_handle::ImageHandle;
pub use crate::common::image_handle::Sauce;
pub use crate::common::pantsu_tag::{PantsuTag, PantsuTagType};
pub use crate::common::tmp_dir::TmpFile;
use crate::image_similarity::ImageToImport;
pub use crate::sauce::SauceMatch;
pub use crate::sauce::get_thumbnails;

mod sauce;
mod common;
pub mod image_similarity;
pub mod db;
pub mod file_handler;

// This check can fail with Error::ImageLoadError or Error:ImageAlreadyExists
pub fn check_image(pantsu_db: &mut PantsuDB, image_path: &Path) -> Result<ImageToImport> {
    let image_info = file_handler::hash::calculate_fileinfo(image_path)?;
    if pantsu_db.get_image_transaction(image_info.filename.as_str()).execute()?.is_some() {
        return Err(Error::ImageAlreadyExists(common::get_path(image_path)));
    }
    let image_handle = ImageHandle::new(image_info.filename, Sauce::NotChecked, image_info.file_res); 
    Ok(ImageToImport {
        current_path: PathBuf::from(image_path),
        image_handle,
    } )
}

pub fn import_image(pantsu_db: &mut PantsuDB, lib: &Path, image: &ImageToImport, always_copy: bool) -> Result<()> { // todo: could consume imageToImport
    import::import_file(lib, &image.current_path, image.image_handle.get_filename(), always_copy)?;
    pantsu_db.add_images_transaction().add_image(&image.image_handle).execute()
}


pub fn get_image_sauces(lib: &Path, image: &ImageHandle) -> Result<Vec<SauceMatch>> {
    let mut image_path = PathBuf::from(lib);
    image_path.push(image.get_filename());
    let mut sauce_matches = sauce::find_sauce(image_path)?;
    sauce_matches.sort();
    sauce_matches.reverse();
    Ok(sauce_matches)
}

pub fn get_sauce_tags(sauce: &SauceMatch) -> Result<Vec<PantsuTag>> {
    sauce::find_tags_gelbooru(&sauce.link)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::path::{Path, PathBuf};
    use crate::{PantsuDB, Sauce};
    use serial_test::serial;
    use crate::image_similarity::NamedImage;

    #[test]
    #[serial]
    fn test_add_image() {
        let mut db_path = std::env::current_dir().unwrap();
        db_path.push("pantsu_tags.db");
        let mut pdb = PantsuDB::new(&db_path).unwrap();
        let image_path = prepare_image("https://img1.gelbooru.com/images/4f/76/4f76b8d52983af1d28b1bf8d830d684e.png");

        let new_image = crate::check_image(&mut pdb, &image_path).unwrap();
        crate::import_image(&mut pdb, Path::new("./test_image_lib"), &new_image, true).unwrap();
        let sauces = crate::get_image_sauces(Path::new("./test_image_lib"), &new_image.image_handle).unwrap();
        let best_match = &sauces[0];
        // in general, you would want to check the similarity here
        let tags = crate::get_sauce_tags(&best_match).unwrap();
        pdb.update_images_transaction().for_image(new_image.get_name()).update_sauce(&Sauce::Match(best_match.link.clone())).add_tags(&tags).execute().unwrap();
    }

    // todo: this test does not really make sense anymore, since import_image() will always succeed even if images are similar. Move to image_similarity module and make into proper test
    #[test]
    #[serial]
    fn test_similar_images() {
        let image_path = prepare_image("https://img1.gelbooru.com/images/4f/76/4f76b8d52983af1d28b1bf8d830d684e.png");
        let similar_image_path = prepare_image("https://img1.gelbooru.com/images/22/3a/223a6444a6e79ecb273896cfccee9850.png");
        let not_similar_image_path = prepare_image("https://img3.gelbooru.com/images/1d/b8/1db8ab6c94e95f36a9dd5bde347f6dd1.png");
        let mut db_path = std::env::current_dir().unwrap();
        db_path.push("pantsu_tags.db");
        let mut pdb = PantsuDB::new(&db_path).unwrap();
        pdb.clear().unwrap();

        let image = crate::check_image(&mut pdb, &image_path).unwrap();
        crate::import_image(&mut pdb, Path::new("./test_image_lib"), &image, false).unwrap();
        let not_similar_image = crate::check_image(&mut pdb, &not_similar_image_path).unwrap();
        crate::import_image(&mut pdb, Path::new("./test_image_lib"), &not_similar_image, false).unwrap();
        let similar_image = crate::check_image(&mut pdb, &similar_image_path).unwrap();
        crate::import_image(&mut pdb, Path::new("./test_image_lib"), &similar_image, false).unwrap();
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