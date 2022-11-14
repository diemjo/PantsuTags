use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use image::GenericImageView;
use image_compressor::Factor;
use image_compressor::compressor::Compressor;
use log::warn;
use crate::{Error, Result, common};

const TMP_DIR_NAME: &str = "pantsu-tags-compressed-images";
const IQDB_MAX_SIZE: u64 = 1<<23;
const IQDB_MAX_DIMENSIONS: (u32,u32) = (7500,7500);

const IMAGE_SCALE_EPSILON: f32 = 0.1;
const QUALITY_IMAGE_HIGH: f32 = 80.0;
const QUALITY_IMAGE_LOW: f32 = 60.0;

pub struct ImagePrepared {
    pub path: PathBuf,
    to_remove: bool,
}

struct ImageMetadata {
    dimensions: (u32,u32),
    file_size: u64,
}

impl ImagePrepared {
    fn new(image_path: PathBuf) -> ImagePrepared {
        ImagePrepared { path: image_path, to_remove: false }
    }

    fn new_tmp(image_path: PathBuf) -> ImagePrepared {
        ImagePrepared { path: image_path, to_remove: true }
    }
}

impl Drop for ImagePrepared {
    fn drop(&mut self) {
        if self.to_remove {
            assert!(self.path.starts_with(std::env::temp_dir()));
            if let Err(_) = fs::remove_file(&mut self.path) {
                warn!("warning: failed to remove temporary image '{}'", common::get_path(&self.path));
            }
        }
    }
}

pub fn prepare_image(image_path: PathBuf) -> Result<ImagePrepared> {
    let metadata = get_image_metadata(&image_path)
        .or_else(|_| Err(Error::ImageLoadError(common::get_path(&image_path))))?;
    if metadata.file_size <= IQDB_MAX_SIZE && tuple::lte(metadata.dimensions, IQDB_MAX_DIMENSIONS) {
        return Ok(ImagePrepared::new(image_path));
    }

    let res_dir = get_tmp_dir()?;

    // try compress to QUALITY_IMAGE_HIGH
    let res = compress_image(&image_path, &res_dir, |w, h, _| {
        return Factor::new(QUALITY_IMAGE_HIGH, get_scale((w, h)))
    });
    if let  Ok(res_image) = res {
        return Ok(res_image);
    }

    // try compress to QUALITY_IMAGE_LOW
    compress_image(&image_path, &res_dir, |w, h, _| {
        return Factor::new(QUALITY_IMAGE_LOW, get_scale((w, h)))
    })
}

fn compress_image(image_path: &Path, res_dir: &Path, factor_func: fn(u32, u32, u64) -> Factor) -> Result<ImagePrepared> {
    print!("  (Compressing image ");   // hide the filename that is printed by compress_to_jpg()
    let comp = Compressor::new(image_path, res_dir, factor_func);
    let res_image = comp.compress_to_jpg()
        .or_else(|_| Err(Error::ImageTooBig(common::get_path(image_path))))?;
    println!(")");
    let res_image = ImagePrepared::new_tmp(res_image);
    let res_metadata = get_image_metadata(&res_image.path)
        .or_else(|_| Err(Error::ImageLoadError(common::get_path(image_path))))?;
    if res_metadata.file_size <= IQDB_MAX_SIZE {
        return Ok(res_image)
    }
    Err(Error::ImageTooBig(common::get_path(image_path)))
}

fn get_image_metadata(image_path: &Path) -> io::Result<ImageMetadata> {
    let mut file = File::open(image_path)?;
    let mut image_content = Vec::new();

    file.read_to_end(&mut image_content)?;
    let image = image::load_from_memory(&image_content)
        .or_else(|_| Err(io::Error::from(io::ErrorKind::NotFound)))?;
    let size = file.metadata()?.len();

    Ok(ImageMetadata {
        dimensions: image.dimensions(),
        file_size: size,
    })
}

fn get_tmp_dir() -> Result<PathBuf>{
    let mut tmp_dir = std::env::temp_dir();
    tmp_dir.push(TMP_DIR_NAME);
    fs::create_dir_all(&tmp_dir)
        .or_else(|err| Err(Error::DirectoryCreateError(err, common::get_path(&tmp_dir))))?;
    Ok(tmp_dir)
}

fn get_scale(dimensions: (u32,u32)) -> f32 {
    if tuple::lte(dimensions, IQDB_MAX_DIMENSIONS) {
        return 1.0;
    }

    let scale = tuple::op(IQDB_MAX_DIMENSIONS, dimensions, |a, b| (a as f32)/(b as f32));
    return tuple::min(scale) - IMAGE_SCALE_EPSILON; // Make sure that image will always be less than IQDB_MAX_DIMENSIONS
}


mod tuple {
    pub fn op<T, R>((a,b): (T,T), (x,y): (T,T), op: fn(T, T)->R) -> (R,R) {
        (op(a, x), op(b, y))
    }
    
    pub fn lte<T: std::cmp::PartialOrd>(tuple1: (T,T), tuple2: (T,T)) -> bool {
        tuple1.0 <= tuple2.0 && tuple1.1 <= tuple2.1
    }
    
    pub fn min<T: std::cmp::PartialOrd>(tuple: (T,T)) -> T {
        if tuple.0 < tuple.1 {
            tuple.0
        }
        else {
            tuple.1
        }
    }
}
