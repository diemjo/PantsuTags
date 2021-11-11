#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to perform operation on file: {0}")]
    FileError(#[from] std::io::Error),

    #[error("Failed to send request: {0}")]
    FailedRequest(#[from] reqwest::Error),
}