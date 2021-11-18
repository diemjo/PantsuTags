use std::path::Path;
use blockhash::Image;
use image::{DynamicImage, GenericImageView, ImageError};
use lz_fnv::{Fnv1a, FnvHasher};
use crate::common::error;
use crate::common::error::Error;

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