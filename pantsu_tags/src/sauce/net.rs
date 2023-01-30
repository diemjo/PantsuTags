use std::path::Path;

use reqwest::{multipart::Part, StatusCode, Url};
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

pub fn https_url(url: &str) -> Result<Url> {
    let mut https_url = Url::parse(url)
        .or_else(|_| Err(Error::BadUrl(String::from(url))))?;
    https_url.set_scheme("https")
        .or_else(|_| Err(Error::BadUrl(String::from(url))))?;
    Ok(https_url)
}

pub fn gelbooru_https_url(url: &str) -> Result<Url> {
    let https_url = https_url(url)?;
    let domain = https_url.domain()
        .ok_or_else(|| Error::BadUrl(String::from(url)))?;

    if !domain.ends_with("gelbooru.com") {
        return Err(Error::BadUrl(String::from(url)))
    }
    Ok(https_url)
}