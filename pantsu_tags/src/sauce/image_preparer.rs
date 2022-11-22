use std::fs::File;
use std::io::{self, ErrorKind};
use std::io::Read;
use std::path::{Path, PathBuf};
use image::GenericImageView;
use image_compressor::Factor;
use image_compressor::compressor::Compressor;
use log::{warn, info};
use tokio::task;
use crate::common::tmp_dir::TmpFile;
use crate::common::tmp_dir_async;
use crate::{Error, Result, common, ImageHandle};

const COMPRESSED_TMP_SUBDIR: &str = "compressed-images";
const IQDB_MAX_SIZE: u64 = 1<<23;
const IQDB_MAX_DIMENSIONS: (u32,u32) = (7500,7500);

const IMAGE_SCALE_EPSILON: f32 = 0.1;
const QUALITY_IMAGE_HIGH: f32 = 80.0;
const QUALITY_IMAGE_LOW: f32 = 60.0;

enum PathKind {
    File(PathBuf),
    TmpFile(TmpFile),
}

pub struct ImagePrepared {
    path: PathKind,
}

struct ImageMetadata {
    dimensions: (u32,u32),
    file_size: u64,
}

impl ImagePrepared {
    fn new(image_path: PathBuf) -> ImagePrepared {
        ImagePrepared { path: PathKind::File(image_path) }
    }

    fn new_tmp(image_path: PathBuf) -> ImagePrepared {
        ImagePrepared { path: PathKind::TmpFile(TmpFile::new(image_path)) }
    }

    pub fn get_path(&self) -> &Path {
        match &self.path {
            PathKind::File(f) => &f,
            PathKind::TmpFile(f) => f.get_path(),
        }
    }
}

pub async fn prepare_image(image_handle: &ImageHandle, lib: &Path) -> Result<ImagePrepared> {
    let image_path = image_handle.get_path(lib);
    let metadata = get_image_metadata(&image_path)
        .or_else(|_| Err(Error::ImageLoadError(common::get_path(&image_path))))?;
    if metadata.file_size <= IQDB_MAX_SIZE && tuple::lte(metadata.dimensions, IQDB_MAX_DIMENSIONS) {
        return Ok(ImagePrepared::new(image_path));
    }

    info!("Image too large, will compress it: {}", image_handle.get_filename());
    let res_dir = tmp_dir_async::get_tmp_dir(COMPRESSED_TMP_SUBDIR).await?;

    task::spawn_blocking(move || {
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
    }).await
        .or_else(|e| Err(Error::CompressImageError(Some(Box::new(Error::TokioBlockingTask(e))))))?
}

fn compress_image(image_path: &Path, res_dir: &Path, factor_func: fn(u32, u32, u64) -> Factor) -> Result<ImagePrepared> {
    let comp = Compressor::new(image_path, res_dir, factor_func);
    let res_image = comp.compress_to_jpg()
        .or_else(|e| {
            compress_image_try_fix_error(image_path, res_dir, e)?;
            comp.compress_to_jpg() // try again
                .or(Err(Error::CompressImageError(None)))
        })?;
    let res_image = ImagePrepared::new_tmp(res_image);
    let res_metadata = get_image_metadata(res_image.get_path())
        .or_else(|_| Err(Error::ImageLoadError(common::get_path(image_path))))?;
    if res_metadata.file_size <= IQDB_MAX_SIZE {
        return Ok(res_image)
    }
    Err(Error::ImageTooBig(common::get_path(image_path)))
}

fn compress_image_try_fix_error(image_path: &Path, res_dir: &Path, error: Box<dyn std::error::Error>) -> Result<()> {
    let io_err = error.downcast_ref::<std::io::Error>()
        .ok_or(Error::CompressImageError(None))?;
    if let ErrorKind::AlreadyExists = io_err.kind() { // remove old file to retry compression
        let file_name = image_path.file_name().and_then(|n| n.to_str())
            .ok_or(Error::CompressImageError(None))?;
        let target_path = Path::new(res_dir).join(file_name);
        std::fs::remove_file(&target_path)
            .or(Err(Error::CompressImageError(None)))?;
        warn!("Removed file {} to compress image to that location", common::get_path(&target_path));
        return Ok(());
    }
    Err(Error::CompressImageError(None))
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
