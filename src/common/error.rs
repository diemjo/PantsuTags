use std::path::Path;
use reqwest::StatusCode;

pub fn get_path(path: &Path) -> String {
    String::from(path.to_str().unwrap_or("cannot display path"))
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // sauce errors
    #[error("Failed to send image source request: {0}")]
    FailedRequest(#[from] reqwest::Error),

    #[error("Received response with bad http status: {0}")]
    BadResponseStatus(StatusCode),

    #[error("Failed to parse html, maybe the website layout changed?")]
    HtmlParseError,

    // pantsu tag database errors
    #[error("Primary key constraint error: {0}")]
    SQLPrimaryKeyError(#[source] rusqlite::Error),

    #[error("Failed underlying SQLite call: {0}")]
    SQLError(#[from] rusqlite::Error),

    #[error("Cannot convert invalid tag type '{0}' to enum variant of PantsuTagType")]
    InvalidTagType(String),

    #[error("Cannot convert tag string '{0}' to PantsuTag")]
    InvalidTagFormat(String),

    #[error("Similar images to '{0}' already exist in database: '{1:?}'")]
    SimilarImagesExist(String, Vec<String>),

    #[error("Failed to add image {0}: Image already exists")]
    ImageAlreadyExists(String),

    // file system
    #[error("File not found: {1}")]
    FileNotFound(#[source] std::io::Error, String),

    #[error("Cannot hard link file '{1}' into image library: {0}")]
    HardLinkError(#[source] std::io::Error, String),

    #[error("Error creating dir {1}: {0}")]
    DirectoryCreateError(#[source] std::io::Error, String),

    #[error("File '{0}' is not an image or cannot be loaded as an image")]
    ImageLoadError(String),

    #[error("'{0}' is not a file")]
    InvalidDatabasePath(String),
}
