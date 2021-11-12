use std::path::Path;
use reqwest::blocking::{Client, Response, multipart};
use select::document::Document;
use select::predicate::Attr;
use crate::common::error::Error;

const IQDB_ADDRESS: &str = "https://gelbooru.iqdb.org/";

pub fn find_sauce(image_path: &Path) -> Result<String, Error> {
    let client = Client::new();
    let form = multipart::Form::new()
        .file("file", image_path)?;
    let response = client.post(IQDB_ADDRESS)
        .multipart(form)
        .send()?;
    if !response.status().is_success() {
        return Err(Error::BadResponse(format!("status code {}", String::from(response.status().as_str()))));
    }

    let response = response.text()?;
    //println!("--- got response ---\n{}", response);
    let html = Document::from(response.as_str());
    check_response( &html)?;

    println!("success uwu!");
    Ok(String::from("hello"))
}

fn check_response(response_html: &Document) -> Result<(), Error> {
    let mut status = response_html.find(Attr("id", "urlstat"));
    if let Some(status) = status.next() {
        if !status.text().trim().starts_with("OK, ") {
            return Err(Error::BadResponse(status.text()));
        }
    }
    else {
        let mut err = response_html.find(Attr("class", "err"));
        if let Some(err) = err.next() {
            return Err(Error::BadResponse(err.text()));
        }
        else {
            return Err(Error::BadResponse(String::from("Unexpected response")));
        }
    }

    Ok(())
}