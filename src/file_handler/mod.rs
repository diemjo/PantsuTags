pub mod hash;
pub mod import;

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::path::PathBuf;
    use crate::file_handler::hash::calculate_filename;
    use crate::file_handler::import::import_file;

    #[test]
    fn test_hash() {
        let file = "https://img1.gelbooru.com/images/4f/76/4f76b8d52983af1d28b1bf8d830d684e.png";
        let file_path = prepare_image(file);
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        let hash_name = calculate_filename(file_name).unwrap();
        println!("{} -> {}", file_name, hash_name);
    }

    #[test]
    fn test_hard_link() {
        let file = "https://img1.gelbooru.com/images/4f/76/4f76b8d52983af1d28b1bf8d830d684e.png";
        let file_path = prepare_image(file);
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        let new_filename = calculate_filename(file_name).unwrap();
        let lib_dir = "./";
        import_file(lib_dir, file_name, &new_filename).unwrap();
        let mut new_path = PathBuf::from(lib_dir);
        new_path.push(new_filename);
        assert!(new_path.exists());
        std::fs::remove_file(new_path);
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