use reqwest::blocking;
use select::document::Document;
use select::predicate::Attr;
use enum_iterator::IntoEnumIterator;
use crate::common::error::Error;
use crate::common::{PantsuTag, PantsuTagType};

pub fn find_tags_gelbooru(url: &str) -> Result<Vec<PantsuTag>, Error> {
    let response = blocking::get(url)?;
    if !response.status().is_success() {
        return Err(Error::BadResponse(format!("status code {}", String::from(response.status().as_str()))));
    }
    let html = Document::from(response.text()?.as_str());
    let tags = extract_tags(&html);
    Ok(tags)
}

fn extract_tags(html: &Document) -> Vec<PantsuTag> {
    let mut tags: Vec<PantsuTag> = Vec::new();
    for tag_type in PantsuTagType::into_enum_iter() {
        extract_tags_of_type(html, tag_type, &mut tags);
    }
    tags
}

fn extract_tags_of_type(html: &Document, tag_type: PantsuTagType, result: &mut Vec<PantsuTag>) {
    let tags = html.find(Attr("class", tag_type.get_gelbooru_attr()));
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

impl PantsuTagType {
    fn get_gelbooru_attr(&self) -> &str {
        match self {
            PantsuTagType::Artist => "tag-type-artist",
            PantsuTagType::Source => "tag-type-copyright",
            PantsuTagType::Character => "tag-type-character",
            PantsuTagType::Generic => "tag-type-general",
        }
    }
}