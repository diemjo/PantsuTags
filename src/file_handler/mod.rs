mod hash;

pub use hash::calculate_filename;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::file_handler::calculate_filename;

    #[test]
    fn test_hash() {
        let file = PathBuf::from("test.jpg");
        let hash_name = calculate_filename(file.as_path()).unwrap();
        println!("{} -> {}", file.file_name().unwrap().to_str().unwrap(), hash_name);
    }
}