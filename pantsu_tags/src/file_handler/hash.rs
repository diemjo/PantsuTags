use std::path::Path;
use std::str::FromStr;
use blockhash::{Blockhash144, Image};
use image::{DynamicImage, GenericImageView};
use lz_fnv::{Fnv1a, FnvHasher};
use crate::common::error;
use crate::common::error::Result;
use crate::common::error::Error;
use crate::file_handler::ImageInfo;
use crate::ImageHandle;

struct AdapterImage<'a> {
    pub image: &'a DynamicImage,
}

impl<'a> Image for AdapterImage<'a> {
    fn dimensions(&self) -> (u32, u32) {
        self.image.dimensions()
    }

    fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        self.image.get_pixel(x, y).0
    }
}

pub(crate) fn calculate_fileinfo(path: &Path) -> Result<ImageInfo> {
    let file_content = std::fs::read(&path).or_else(|_|
        Err(Error::ImageLoadError(error::get_path(&path)))
    )?;
    let file_extension = get_file_extension(&path)?;

    let image = image::load_from_memory(&file_content).or_else(|_|
        Err(Error::ImageLoadError(error::get_path(&path)))
    )?;
    let fnv1a_hash = get_fnv1a_hash(&file_content);
    let perceptual_hash = get_perceptual_hash(&image);

    Ok(ImageInfo {
        filename: format!("{}-{}.{}", fnv1a_hash, perceptual_hash, file_extension),
        file_res: image.dimensions()
    })
}

pub fn get_similarity_distances(filename: &str, files: Vec<ImageHandle>, min_dist: u32) -> Result<Vec<ImageHandle>> {
    let file_hash = extract_hash(filename)?;
    Ok(files.into_iter()
        .filter(|file|{
            let p_hash = extract_hash(file.get_filename()).unwrap();
            let dist = file_hash.distance(&p_hash);
            dist < min_dist && file.get_filename()!=filename
        }).collect::<Vec<ImageHandle>>())
}

fn extract_hash(filename: &str) -> Result<Blockhash144> {
    let filename = filename.trim();
    if !super::filename_is_valid(filename) {
        return Err(Error::InvalidFilename(String::from(filename)))
    }
    // 0-15=fnv_hash, 16='-', 17-52=p_hash, 53='.', 54+=extension
    let p_hash = &filename[17..53];
    Ok(Blockhash144::from_str(p_hash).unwrap())
}

fn get_fnv1a_hash(bytes: &Vec<u8>) -> String {
    let mut fnv = Fnv1a::<u64>::new();
    fnv.write(bytes);
    format!("{:016x}", fnv.finish())
}

fn get_perceptual_hash(image: &DynamicImage) -> String {
    let hash = blockhash::blockhash144(&AdapterImage { image });
    hash.to_string()
}

fn get_file_extension(path: &Path) -> Result<String> {
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
        let hash = extract_hash(&name).unwrap();
        println!("{:?}", hash);
        assert_eq!(hash, Blockhash144::from_str("7c7703613f313831e31e34e25cd7cd7e05c0").unwrap())
    }
}