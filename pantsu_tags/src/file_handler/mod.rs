use std::path::PathBuf;
use directories::{ProjectDirs};
use regex::Regex;

pub mod hash;
pub mod import;

pub(crate) struct ImageInfo {
    pub filename: String,
    pub file_res: (u32, u32)
}

pub fn default_db_dir() -> PathBuf {
    match ProjectDirs::from("moe", "karpador", "PantsuTags") {
        Some(project_dir) => {
            let mut path = PathBuf::new();
            path.push(project_dir.data_dir());
            path
        },
        None => panic!("No valid home dir found")
    }
}

pub fn filename_is_valid(name: &str) -> bool {
    let regex = Regex::new(r"^[[:xdigit:]]{16}-[[:xdigit:]]{36}\.[^.]+$").unwrap();
    regex.is_match(name.trim())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::path::{Path, PathBuf};
    use crate::file_handler::hash;
    use crate::file_handler::{filename_is_valid, import};

    #[test]
    fn test_regex() {
        let name1 = "217c401223d83fae-e0ff0f00de0fcc6cc2ff1f40f00e60e70e74.png";
        let name2 = "0217c401223d83fae-e0ff0f00de0fcc6cc2ff1f40f00e60e70e74.png";
        let name2_5 = "217c401223d83fae--e0ff0f00de0fcc6cc2ff1f40f00e60e70e74.png";
        let name3 = "/home/217c401223d83fae-e0ff0f00de0fcc6cc2ff1f40f00e60e70e74.png";
        let name4 = "217c401223d83fae-e0ff0f00de0fcc6cc2ff1f40f00e60e70e74.png ";
        let name5 = " 217c401223d83fae-e0ff0f00de0fcc6cc2ff1f40f00e60e70e74.pngdd";
        assert!(filename_is_valid(name1));
        assert!(!filename_is_valid(name2));
        assert!(!filename_is_valid(name2_5));
        assert!(!filename_is_valid(name3));
        assert!(filename_is_valid(name4));
        assert!(filename_is_valid(name5));
    }

    #[test]
    fn test_hash() {
        let file = "https://img1.gelbooru.com/images/4f/76/4f76b8d52983af1d28b1bf8d830d684e.png";
        let file_path = prepare_image(file);
        let hash_name = hash::calculate_fileinfo(&file_path).unwrap().filename;
        println!("{} -> {}", file_path.file_name().unwrap().to_str().unwrap(), hash_name);
    }

    #[test]
    fn test_hard_link() {
        let file = "https://img1.gelbooru.com/images/4f/76/4f76b8d52983af1d28b1bf8d830d684e.png";
        let file_path = prepare_image(file);
        let new_filename = hash::calculate_fileinfo(&file_path).unwrap().filename;
        let lib_dir = Path::new("./");
        import::import_file(lib_dir, &file_path, &new_filename, false).unwrap();
        let mut new_path = PathBuf::from(lib_dir);
        new_path.push(new_filename);
        assert!(new_path.exists());
        std::fs::remove_file(new_path).unwrap();
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