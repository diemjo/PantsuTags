use std::path::Path;
use reqwest::blocking::Client;
use reqwest::blocking::multipart;
use crate::common::error::Error;

const IQDB_ADDRESS: &str = "https://gelbooru.iqdb.org/";

pub fn find_sauce(image_path: &Path) -> Result<String, Error> {
    let client = Client::new();
    let form = multipart::Form::new()
        .file("file", image_path)?;
    let response = client.post(IQDB_ADDRESS)
        .multipart(form)
        .send()?;
    let response = response.text()?;

    println!("got response:\n{}", response);

    Ok(String::from("hello"))
}