pub mod hash;
pub mod import;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::file_handler::hash::calculate_filename;
    use crate::file_handler::import::import_file;

    #[test]
    fn test_hash() {
        let file = "test.jpg";
        let hash_name = calculate_filename(file).unwrap();
        println!("{} -> {}", PathBuf::from(file).file_name().unwrap().to_str().unwrap(), hash_name);
    }

    #[test]
    fn test_hard_link() {
        let filename = "test.jpg";
        let new_filename = calculate_filename(filename).unwrap();
        let lib_dir = "./";
        import_file(lib_dir, filename, &new_filename).unwrap();
        let mut new_path = PathBuf::from(lib_dir);
        new_path.push(new_filename);
        assert!(new_path.exists());
    }
}