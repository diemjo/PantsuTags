use pantsu_tags::{Error, SauceMatch};

pub type AppResult<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Couldn't find relevant sauces")]
    NoRelevantSauces,

    #[error("Not sure whether sauce is correct or not")]
    SauceUnsure(Vec<SauceMatch>),

    #[error("Failed to read from stdin")]
    StdinReadError(#[source]std::io::Error),

    #[error(transparent)]
    LibError(#[from] Error),
}