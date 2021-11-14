#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Image not found: {1}")]
    ImageNotFound(#[source] std::io::Error, String),

    #[error("Failed to send image source request: {0}")]
    FailedRequest(#[from] reqwest::Error),

    #[error("Unable to retrieve image source, bad response: {0}")]
    BadResponse(String),

    // pantsu tag database errors
    #[error("Failed to add tag '{2}' for file '{1}': {0}")]
    TagInsertionError(#[source] rusqlite::Error, String /* file */, String /* tag */),

    #[error("Failed to remove tag '{2}' from file '{1}': {0}")]
    TagRemovalError(#[source] rusqlite::Error, String /* file */, String /* tag */),

    #[error("Failed underlying SQLite call: {0}")]
    SQLError(#[from] rusqlite::Error)
}