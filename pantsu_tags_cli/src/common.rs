use std::path::Path;
use pantsu_tags::{Error, file_handler, SauceMatch};
use pantsu_tags::ImageHandle;

pub type AppResult<T> = std::result::Result<T, AppError>;

pub fn valid_filename_from_path(path: &Path) -> AppResult<String> {
    let filename = path
        .file_name()
        .ok_or(Error::InvalidFilename(path.to_string_lossy().to_string()))?
        .to_string_lossy()
        .to_string();
    if !file_handler::filename_is_valid(filename.as_str()) {
        Err(Error::InvalidFilename(filename))?
    } else {
        Ok(filename)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Couldn't find relevant sauces")]
    NoRelevantSauces,

    #[error("Not sure whether sauce is correct or not")]
    SauceUnsure(ImageHandle, Vec<SauceMatch>),

    #[error("Image not found in PantsuTags: {0}")]
    ImageNotFound(String),

    #[error("Failed to read from stdin")]
    StdinReadError(#[source]std::io::Error),

    #[error(transparent)]
    LibError(#[from] Error),
}