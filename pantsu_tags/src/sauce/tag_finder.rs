use reqwest::Client;
use select::document::Document;
use select::predicate::Attr;
use enum_iterator::IntoEnumIterator;
use select::node::Node;
use crate::common::error::Error;
use crate::common::pantsu_tag::{PantsuTag, PantsuTagType};

use super::net;

// If image was deleted on gelbooru, throws an HtmlParseError
pub async fn find_tags_gelbooru(url: &str) -> Result<Vec<PantsuTag>, Error> {
    let client = Client::new();
    let resp = client.get(net::gelbooru_https_url(url)?).send().await?;
    net::check_status(resp.status())?;
    let text = resp.text().await?;
    let html = Document::from(text.as_str());
    extract_tags(&html)
}

fn extract_tags(html: &Document) -> Result<Vec<PantsuTag>, Error> {
    let mut tags: Vec<PantsuTag> = Vec::new();
    let tag_list = html.find(Attr("id", "tag-list")).next().ok_or(Error::HtmlParseError)?; // html should always contain the tag-list html element
    extract_rating(&tag_list, &mut tags)?;
    for tag_type in PantsuTagType::into_enum_iter() {
        extract_tags_of_type(&tag_list, tag_type, &mut tags);
    }

    if tags.is_empty() {
        return Err(Error::HtmlParseError); // every gelbooru page has tags
    }
    Ok(tags)
}

fn extract_tags_of_type(tag_list: &Node, tag_type: PantsuTagType, result: &mut Vec<PantsuTag>) {
    if let Some(gelbooru_attr) = tag_type.get_gelbooru_attr() {
        let tags = tag_list.find(Attr("class", gelbooru_attr));
        for tag in tags {
            for node in tag.children() {
                if node.is(Attr("href", ())) {
                    result.push(PantsuTag {
                        tag_name: node.text(),
                        tag_type,
                    });
                }
            }
        }
    }
}

fn extract_rating(tag_list: &Node, result: &mut Vec<PantsuTag>) -> Result<(), Error>{
    for tag in tag_list.children() {
        match tag.text().strip_prefix("Rating: ") {
            Some(rating) => {
                result.push(PantsuTag {
                    tag_name: String::from(rating.trim()),
                    tag_type: PantsuTagType::Rating,
                });
                return Ok(());
            }
            None => {}
        }
    }
    Err(Error::HtmlParseError) // every gelbooru page has a rating
}

impl PantsuTagType {
    fn get_gelbooru_attr(&self) -> Option<&str> {
        match self {
            PantsuTagType::Artist => Some("tag-type-artist"),
            PantsuTagType::Source => Some("tag-type-copyright"),
            PantsuTagType::Character => Some("tag-type-character"),
            PantsuTagType::General => Some("tag-type-general"),
            PantsuTagType::Rating => None,
            PantsuTagType::Custom => None,
        }
    }
}