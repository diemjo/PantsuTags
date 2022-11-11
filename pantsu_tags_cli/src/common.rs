use std::path::Path;
use pantsu_tags::{Error, file_handler, SauceMatch};
use pantsu_tags::ImageHandle;

pub type AppResult<T> = std::result::Result<T, AppError>;

pub fn get_path(path: &Path) -> String {
    String::from(path.to_str().unwrap_or("cannot display path"))
}

pub fn get_filename(path: &Path) -> AppResult<String> {
    match path.file_name() {
        Some(name) => match name.to_str() {
            Some(name) => Ok(String::from(name)),
            None => Err(Error::InvalidFilename(get_path(path)))?,
        },
        None => Err(Error::InvalidFilename(get_path(path)))?
    }
}

pub fn valid_filename_from_path(path: &Path) -> AppResult<String> {
    let filename = get_filename(path)?;
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

    #[error("Failed to load config")]
    ConfigError(#[from] figment::Error),
}