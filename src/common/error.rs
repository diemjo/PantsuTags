#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to perform operation on file: {0}")]
    FileError(#[from] std::io::Error),

    #[error("Failed to send image source request: {0}")]
    FailedRequest(#[from] reqwest::Error),

    #[error("Unable to retrieve image source, bad response: {0}")]
    BadResponse(String)
}