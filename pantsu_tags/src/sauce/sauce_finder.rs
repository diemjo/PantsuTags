use std::path::PathBuf;
use futures::{stream, StreamExt};
use reqwest::{blocking, StatusCode};
use reqwest::blocking::{Client, multipart};
use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Name};
use tokio::io;
use crate::common;
use crate::common::tmp_dir::TmpFile;
use crate::common::tmp_dir_async;
use crate::common::error::Error;
use crate::common::error::Result;
use super::SauceMatch;
use super::image_preparer;

const IQDB_ADDRESS: &str = "https://gelbooru.iqdb.org/";
const MAX_CONCURRENT_REQUESTS: usize = 16;
const THUMBNAIL_TMP_SUBDIR: &str = "thumbnails";

// image path has to point to an image, otherwise returns an Error::HtmlParseError
pub fn find_sauce(image_path: PathBuf) -> Result<Vec<SauceMatch>> {
    let image = image_preparer::prepare_image(image_path)?;
    let client = Client::new();
    let form = multipart::Form::new()
        .file("file", image.get_path()).or_else(
        |err| Err(Error::FileNotFound(err, common::get_path(image.get_path()))))?;
    let response = client.post(IQDB_ADDRESS)
        .multipart(form)
        .send()?;
    check_status(response.status())?;

    let response = response.text()?;
    let html = Document::from(response.as_str());
    extract_sauce(&html)
}

// todo: remove?
fn get_thumbnail_link(sauce: &SauceMatch) -> Result<String> {
    let response = blocking::get(&sauce.link)?;
    check_status(response.status())?;
    let html = Document::from(response.text()?.as_str());
    let image = html.find(Attr("id", "image")).next().ok_or(Error::HtmlParseError)?; // thumbnail html element should always exist
    let link = image.attr("src").ok_or(Error::HtmlParseError)?;
    Ok(link.to_owned())
}

pub fn get_thumbnails(sauces: &Vec<SauceMatch>) -> Result<Vec<TmpFile>> {
    let rt = tokio::runtime::Runtime::new()
        .or_else(|e| Err(Error::TokioInitError(e)))?;
    let links = rt.block_on(get_thumbnails_async(sauces))?;

    // make sure that the vec of links has the same order as the vec of Sauces
    Ok(links.into_iter().enumerate()
        .map(|(idx,link)| {
            assert!(*link.1 == sauces[idx]);
            link.0
        }
    ).collect())
}

// explanation: https://stackoverflow.com/a/51047786
async fn get_thumbnails_async(sauces: &Vec<SauceMatch>) -> Result<Vec<(TmpFile,&SauceMatch)>> {
    let client = reqwest::Client::new();
    let links = stream::iter(sauces)
        .map(|sauce| {
            let client = &client;
            async move {
                let resp = client.get(&sauce.link).send().await?;
                check_status(resp.status())?;
                let text = resp.text().await
                    .map_err(|_| Error::FailedThumbnail)?;
                let link = extract_thumbnail_link(&text)?;

                let resp = client.get(&link).send().await?;
                check_status(resp.status())?;
                let data = resp.bytes().await?;
                let path = store_thumbnail(&link, data.as_ref()).await?;
                Ok((path,sauce))
            }
        })
        .buffered(MAX_CONCURRENT_REQUESTS)
        .collect::<Vec<Result<(TmpFile,&SauceMatch)>>>().await;

    links.into_iter().collect::<Result<Vec<(TmpFile,&SauceMatch)>>>()
}

// Extract thumbnail from Gelbooru page
fn extract_thumbnail_link(html_text: &str) -> Result<String> {
    let html = Document::from(html_text);
    let image = html.find(Attr("id", "image")).next().ok_or(Error::HtmlParseError)?; // thumbnail html element should always exist
    image.attr("src").ok_or(Error::HtmlParseError).map(|link| link.to_owned())
}

async fn store_thumbnail(link: &str, data: &[u8]) -> Result<TmpFile> {
    let file_name = link.rsplit_once('/').map(|(_,name)| name ).unwrap_or(link);
    let (path,mut file) = tmp_dir_async::create_tmp_file(THUMBNAIL_TMP_SUBDIR, file_name).await?;
    io::copy(&mut data.as_ref(), &mut file).await
        .or(Err(Error::FailedThumbnail))?;
    Ok(path)
}

fn extract_sauce(html: &Document) -> Result<Vec<SauceMatch>> {
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

fn check_status(status: StatusCode) -> Result<()> {
    if !status.is_success() {
        return Err(Error::BadResponseStatus(status));
    }
    Ok(())
}