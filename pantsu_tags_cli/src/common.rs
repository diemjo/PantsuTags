use std::path::Path;
use pantsu_tags::{Error, ImageHandle};
use tokio::task::JoinError;

pub type AppResult<T> = std::result::Result<T, AppError>;

pub fn get_path(path: &Path) -> String {
    String::from(path.to_str().unwrap_or("cannot display path"))
}

pub fn try_get_path(path: &Path) -> AppResult<String> {
    let path_str = path.to_str().ok_or(AppError::PathConversionError)?;
    Ok(String::from(path_str))
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

pub fn image_handle_from_path(path: &Path) -> AppResult<ImageHandle> {
    let filename = get_filename(path)?;
    let image_handle = ImageHandle::new(filename)?;
    Ok(image_handle)
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Image not found in PantsuTags: {0}")]
    ImageNotFound(String),

    #[error("Failed to read from stdin")]
    StdinReadError(#[source]std::io::Error),

    #[error(transparent)]
    LibError(#[from] Error),

    #[error("Failed to load config")]
    ConfigError(#[from] figment::Error),

    #[error("Invalid path: unable to convert path to string")]
    PathConversionError,

    #[error("Faild to join with task")]
    TaskJoinError(#[from] JoinError),

    #[error("Failed to communicate with task")]
    TaskCommunicationError,
}