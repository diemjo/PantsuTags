use reqwest::blocking;
use select::document::Document;
use select::predicate::Attr;
use enum_iterator::IntoEnumIterator;
use select::node::Node;
use crate::common::error::Error;
use crate::common::pantsu_tag::{PantsuTag, PantsuTagType};

pub fn find_tags_gelbooru(url: &str) -> Result<Vec<PantsuTag>, Error> {
    let response = blocking::get(url)?;
    if !response.status().is_success() {
        return Err(Error::BadResponseStatus(response.status()));
    }
    let html = Document::from(response.text()?.as_str());
    let tags = extract_tags(&html)?;

    if tags.is_empty() {
        return Err(Error::NoTagsFound(String::from(url)));
    }
    Ok(tags)
}

fn extract_tags(html: &Document) -> Result<Vec<PantsuTag>, Error> {
    let mut tags: Vec<PantsuTag> = Vec::new();
    let tag_list = match html.find(Attr("id", "tag-list")).next() {
        Some(tag_list) => tag_list,
        None => return Ok(tags),
    };
    extract_rating(&tag_list, &mut tags)?;
    for tag_type in PantsuTagType::into_enum_iter() {
        extract_tags_of_type(&tag_list, tag_type, &mut tags);
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
    Err(Error::ErrorGettingTags)
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