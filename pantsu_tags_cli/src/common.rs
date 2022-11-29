use std::{path::Path, str::FromStr};
use pantsu_tags::{Error, ImageHandle, db::sort::{SortOrder, ImageSortOption, TagSortOption}};
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

pub fn parse_image_sort_order(options: Vec<String>) -> AppResult<Option<SortOrder<ImageSortOption>>> {
    let options = options.iter()
        .map(|o| ImageSortOption::from_str(o).or_else(|e| Err(AppError::LibError(e))))
        .collect::<AppResult<Vec<ImageSortOption>>>()?;
    match SortOrder::<ImageSortOption>::new(options) {
        Ok(s) => Ok(Some(s)),
        Err(Error::NoSortingOptionSpecified) => Ok(None),
        Err(e) => Err(e)?
    }
}

pub fn parse_tag_sort_order(options: Vec<String>) -> AppResult<Option<SortOrder<TagSortOption>>> {
    let options = options.iter()
        .map(|o| TagSortOption::from_str(o).or_else(|e| Err(AppError::LibError(e))))
        .collect::<AppResult<Vec<TagSortOption>>>()?;
    match SortOrder::<TagSortOption>::new(options) {
        Ok(s) => Ok(Some(s)),
        Err(Error::NoSortingOptionSpecified) => Ok(None),
        Err(e) => Err(e)?
    }
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