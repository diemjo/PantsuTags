use std::path::Path;

use reqwest::{multipart::Part, StatusCode};
use tokio::io::AsyncReadExt;

use crate::{common, Error, Result};

pub async fn create_image_part(image: &Path) -> Result<Part> {
    let mut file = tokio::fs::File::open(image).await
        .or_else(|err| Err(Error::FileNotFound(err, common::get_path(image))))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).await
        .or(Err(Error::FailedRequestCreation))?;
    let file_name = String::from(image.file_name().and_then(|name| name.to_str())
        .ok_or(Error::FailedRequestCreation)?);
    let mime = mime_guess::from_path(image).first_or_octet_stream();

    Part::bytes(bytes)
        .file_name(file_name)
        .mime_str(mime.essence_str())
        .or(Err(Error::FailedRequestCreation))
}

pub fn check_status(status: StatusCode) -> Result<()> {
    if !status.is_success() {
        return Err(Error::BadResponseStatus(status));
    }
    Ok(())
}