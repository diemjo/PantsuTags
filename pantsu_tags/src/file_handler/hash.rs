use std::path::Path;
use std::str::FromStr;
use blockhash::{Blockhash144, Image};
use image::{DynamicImage, GenericImageView, ImageError};
use lz_fnv::{Fnv1a, FnvHasher};
use crate::common::error;
use crate::common::error::Error;
use crate::ImageHandle;

struct AdapterImage {
    pub image: DynamicImage,
}

impl Image for AdapterImage {
    fn dimensions(&self) -> (u32, u32) {
        self.image.dimensions()
    }

    fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        self.image.get_pixel(x, y).0
    }
}

pub fn calculate_filename(path: &Path) -> Result<String, Error>{
    let file_content = std::fs::read(&path).or_else(|_|
        Err(Error::ImageLoadError(error::get_path(&path)))
    )?;
    let file_extension = get_file_extension(&path)?;

    let fnv1a_hash = get_fnv1a_hash(&file_content);
    let perceptual_hash = get_perceptual_hash(&file_content).or_else(|_|
        Err(Error::ImageLoadError(error::get_path(&path)))
    )?;

    Ok(format!("{}-{}.{}", fnv1a_hash, perceptual_hash, file_extension))
}

pub fn get_similarity_distances(filename: &String, files: Vec<ImageHandle>, min_dist: u32) -> Vec<String> {
    let file_hash = extract_hash(filename);
    files.into_iter()
        .filter(|file|{
            let p_hash = extract_hash(file.get_filename());
            let dist = file_hash.distance(&p_hash);
            dist < min_dist
        }).map(|file|{
            String::from(file.get_filename())
        }).filter(|file|{
            file!=filename
        }).collect::<Vec<String>>()
}

fn extract_hash(filename: &str) -> Blockhash144 {
    // 0-15=fnv_hash, 16='-', 17-52=p_hash, 53='.', 54+=extension
    let p_hash = &filename[17..53];
    Blockhash144::from_str(p_hash).unwrap()
}

fn get_fnv1a_hash(bytes: &Vec<u8>) -> String {
    let mut fnv = Fnv1a::<u64>::new();
    fnv.write(bytes);
    format!("{:016x}", fnv.finish())
}

fn get_perceptual_hash(bytes: &[u8]) -> Result<String, ImageError> {
    let image = image::load_from_memory(bytes)?;
    let hash = blockhash::blockhash144(&AdapterImage { image });
    Ok(hash.to_string())
}

fn get_file_extension(path: &Path) -> Result<String, Error> {
    let file_extension = path.extension().ok_or_else(||
        Error::ImageLoadError(error::get_path(&path))
    )?;
    let file_extension = file_extension.to_str().ok_or_else(||
        Error::ImageLoadError(error::get_path(&path))
    )?;
    Ok(String::from(file_extension))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use blockhash::Blockhash144;
    use crate::file_handler::hash::extract_hash;

    #[test]
    fn test_hash_extract() {
        let name = String::from("9cc02982301095ef-7c7703613f313831e31e34e25cd7cd7e05c0.png");
        let hash = extract_hash(&name);
        println!("{:?}", hash);
        assert_eq!(hash, Blockhash144::from_str("7c7703613f313831e31e34e25cd7cd7e05c0").unwrap())
    }
}