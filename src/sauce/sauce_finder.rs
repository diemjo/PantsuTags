use std::path::Path;
use reqwest::blocking::Client;
use reqwest::blocking::multipart;

const IQDB_ADDRESS: &str = "http://gelbooru.iqdb.org/";

pub fn find_sauce(image_path: &Path) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let form = multipart::Form::new()
        .file("file", image_path).unwrap();
    let response = client.post(IQDB_ADDRESS)
        .multipart(form)
        .send()?;
    let response = response.text()?;

    println!("got response:\n{}", response);

    Ok(String::from("hello"))
}