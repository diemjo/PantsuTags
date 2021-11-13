#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Image not found: {1}")]
    ImageNotFound(#[source] std::io::Error, String),

    #[error("Failed to send image source request: {0}")]
    FailedRequest(#[from] reqwest::Error),

    #[error("Unable to retrieve image source, bad response: {0}")]
    BadResponse(String)
}