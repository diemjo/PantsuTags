use std::path::Path;
use reqwest::blocking::{Client, multipart};
use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Name};
use crate::common::error;
use crate::common::error::Error;
use super::SauceMatch;

const IQDB_ADDRESS: &str = "https://gelbooru.iqdb.org/";

// image path has to point to an image, otherwise returns an Error::HtmlParseError
pub fn find_sauce(image_path: &Path) -> Result<Vec<SauceMatch>, Error> {
    let client = Client::new();
    let form = multipart::Form::new()
        .file("file", image_path).or_else(
        |err| Err(Error::FileNotFound(err, error::get_path(image_path))))?;
    let response = client.post(IQDB_ADDRESS)
        .multipart(form)
        .send()?;
    if !response.status().is_success() {
        return Err(Error::BadResponseStatus(response.status()));
    }

    let response = response.text()?;
    let html = Document::from(response.as_str());
    extract_sauce(&html)
}

fn extract_sauce(html: &Document) -> Result<Vec<SauceMatch>, Error> {
    let mut pages = html.find(Attr("id", "pages"));
    let pages = pages.next().ok_or(Error::HtmlParseError)?; // html element "pages" should always exist, even if there are no relevant matches. Maybe file wasn't an image?
    let mut res: Vec<SauceMatch> = Vec::new();

    for (idx, img_match) in pages.children().enumerate() {
        if idx == 0 {
            continue; // skip uploaded image
        }

        let mut sauce_link: Option<String> = None;
        let mut sauce_similarity: Option<i32> = None;
        let mut sauce_resolution: Option<(i32, i32)> = None;
        for (idx, node) in img_match.find(Name("tr")).enumerate() {
            match idx {
                0 => continue,
                1 => {
                    sauce_link = extract_sauce_link(node);
                    if let None = sauce_link {
                        continue;
                    }
                },
                2 => {
                    sauce_resolution = extract_sauce_resolution(node);
                },
                3 => {
                    sauce_similarity = extract_sauce_similarity(node);
                }
                _ => break,
            }
        }
        if let (Some(link), Some(similarity), Some(resolution)) = (sauce_link, sauce_similarity, sauce_resolution) {
            let sauce_match = SauceMatch {
                link,
                similarity,
                resolution,
            };
            res.push(sauce_match);
        }
    }

    if res.is_empty() {
        return Err(Error::HtmlParseError); // iqdb always returns matches
    }

    Ok(res)
}

fn extract_sauce_link(sauce_match_tr_element: Node) -> Option<String> {
    let td_or_th = sauce_match_tr_element.first_child();
    if td_or_th.is_none() {
        return None;
    }
    let td_or_th = td_or_th.unwrap();

    if td_or_th.is(Name("th")) {
        return None;
    }

    let link = td_or_th.first_child();
    if link.is_none() {
        return None;
    }
    let href = link.unwrap().attr("href");
    if href.is_none() {
        return None
    }
    let href = href.unwrap();
    if href.starts_with("//") {
        Some("https:".to_string() + href)
    } else {
        Some(href.to_string())
    }
}

fn extract_sauce_resolution(sauce_match_tr_element: Node) -> Option<(i32, i32)> {
    let td = sauce_match_tr_element.first_child();
    if td.is_none() {
        return None;
    }

    let td = td.unwrap();
    let text = td.text();
    let resolution = text.split_whitespace().next();
    if let Some(resolution) = resolution {
        let mut resolution = resolution.split('×');
        let resolution = (resolution.next(), resolution.next());
        if let (Some(resol1), Some(resol2)) = resolution {
            if let (Ok(resol1), Ok(resol2)) = (resol1.parse::<i32>(), resol2.parse::<i32>()) {
                return Some((resol1, resol2));
            }
        }
    }

    None
}

fn extract_sauce_similarity(sauce_match_tr_element: Node) -> Option<i32> {
    let td = sauce_match_tr_element.first_child();
    if td.is_none() {
        return None;
    }
    let td = td.unwrap();
    let text = td.text();
    let similarity = text.split('%').collect::<Vec<&str>>()[0];
    similarity.parse::<i32>().ok()
}