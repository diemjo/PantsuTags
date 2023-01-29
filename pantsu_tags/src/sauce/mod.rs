use std::cmp::Ordering;
use std::fmt::Display;
use std::fmt::Formatter;
use crate::error::{Error, Result};

mod tag_finder;
mod sauce_finder;
mod image_preparer;
mod net;

use reqwest::Url;
pub use sauce_finder::find_sauce;
pub use sauce_finder::get_thumbnails;
pub use tag_finder::find_tags_gelbooru;

pub fn url_from_str(url: &str) -> Result<Url> {
    Url::parse(url).or_else(|_| Err(Error::InvalidSauce(url.to_string())))
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Sauce {
    Match(Url),
    NotExisting,
    NotChecked
}

impl Sauce {
    pub fn get_type(&self) -> &str {
        match self {
            Sauce::Match(_) => EXISTING_FLAG,
            Sauce::NotChecked => NOT_CHECKED_FLAG,
            Sauce::NotExisting => NOT_EXISTING_FLAG,
        }
    }

    pub fn get_value(&self) -> Option<&str> {
        match self {
            Sauce::Match(url) => Some(url.as_str()),
            Sauce::NotChecked => None,
            Sauce::NotExisting => None,
        }
    }
}

impl Display for Sauce {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Sauce::Match(v) => v.as_str(),
            Sauce::NotChecked => NOT_CHECKED_FLAG,
            Sauce::NotExisting => NOT_EXISTING_FLAG
        })
    }
}

pub const EXISTING_FLAG: &str =
    "EXISTING";
pub const NOT_EXISTING_FLAG: &str =
    "NOT_EXISTING";

pub const NOT_CHECKED_FLAG: &str =
    "NOT_CHECKED";

#[derive(Debug, Eq)]
pub struct SauceMatch {
    pub link: String,           // link can be invalid if image was deleted on gelbooru
    pub similarity: i32,
    pub resolution: (i32, i32),
}

impl Ord for SauceMatch {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.similarity.cmp(&other.similarity) {
            Ordering::Equal => (self.resolution.0 * self.resolution.1).cmp(&(other.resolution.0 * other.resolution.1)),
            other => other,
        }
    }
}

impl PartialOrd for SauceMatch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SauceMatch {
    fn eq(&self, other: &Self) -> bool {
        self.similarity == other.similarity && self.resolution == other.resolution
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::path::PathBuf;
    use crate::common::pantsu_tag::{PantsuTag, PantsuTagType};
    use crate::file_handler::hash::{self};
    use crate::sauce::{sauce_finder, tag_finder};
    use crate::{SauceMatch};

    fn prepare_image(image_link: &str) -> PathBuf {
        let image_name = image_link.rsplit('/').next().unwrap();
        let image_path = PathBuf::from(format!("test_image_{}", image_name));
        if image_path.exists() {
            return image_path;
        }

        let response = reqwest::blocking::get(image_link).unwrap();
        let mut file = std::fs::File::create(&image_path).unwrap();
        let mut content =  Cursor::new(response.bytes().unwrap());
        std::io::copy(&mut content, &mut file).unwrap();
        image_path
    }

    #[tokio::test]
    async fn find_sauce() {
        let sauce_link = "http://gelbooru.com/index.php?page=post&s=list&md5=4f76b8d52983af1d28b1bf8d830d684e";
        let image_link = "https://img1.gelbooru.com/images/4f/76/4f76b8d52983af1d28b1bf8d830d684e.png";
        let path = prepare_image(image_link);
        let image = hash::calculate_fileinfo(&path).unwrap().0;
        std::fs::copy(path, image.get_filename()).unwrap();
        //let image = ImageHandle::new(path.file_name().unwrap().to_str().unwrap().to_string(), Sauce::NotChecked, (0, 0));

        let sauces = sauce_finder::find_sauce(&image, &PathBuf::from(".")).await.unwrap();
        std::fs::remove_file(image.get_filename()).unwrap();
        assert_eq!(sauces[0].link, sauce_link);
        assert_eq!(sauces[0].similarity, 95);
        assert_eq!(sauces[0].resolution, (533, 745));
    }

    #[tokio::test]
    async fn find_tag() {
        let url = "http://gelbooru.com/index.php?page=post&s=list&md5=4f76b8d52983af1d28b1bf8d830d684e";
        let tags = tag_finder::find_tags_gelbooru(url).await.unwrap();
        assert!(tags.iter().any(|tag| tag.tag_name.eq("loli") && matches!(tag.tag_type, PantsuTagType::General)));
        assert!(tags.iter().any(|tag| tag.tag_name.eq("stuffed dinosaur") && matches!(tag.tag_type, PantsuTagType::General)));
        assert!(tags.iter().any(|tag| tag.tag_name.eq("ichihaya") && matches!(tag.tag_type, PantsuTagType::Artist)));
        assert!(tags.iter().any(|tag| tag.tag_name.eq("awano iroha") && matches!(tag.tag_type, PantsuTagType::Character)));
        assert!(tags.iter().any(|tag| tag.tag_name.eq("original") && matches!(tag.tag_type, PantsuTagType::Source)));
        assert!(tags.iter().any(|tag| tag.tag_name.eq("Questionable") && matches!(tag.tag_type, PantsuTagType::Rating)));
        assert!(!tags.iter().any(|tag| tag.tag_name.eq("large breasts") && matches!(tag.tag_type, PantsuTagType::Source)));
    }

    #[tokio::test]
    #[ignore]
    async fn find_multiple_tags() {
        let links: Vec<&str> = vec![
            "https://gelbooru.com/index.php?page=post&s=list&md5=b3b2aa651df45f6cd74f9c45fb715c79",
            "https://gelbooru.com/index.php?page=post&s=view&id=6261499",
            "https://gelbooru.com/index.php?page=post&s=view&id=3160369",
            "https://gelbooru.com/index.php?page=post&s=view&id=5276383",
        ];
        let mut tags: Vec<Vec<PantsuTag>> = Vec::new();
        for link in links {
            tags.push(tag_finder::find_tags_gelbooru(link).await.unwrap());
        }
        for tag in tags {
            assert!(tag.iter().any(|t| t.tag_name.eq("original") && matches!(t.tag_type, PantsuTagType::Source)))
        }
    }

    #[tokio::test]
    async fn find_tags_rating() {
        let links: Vec<(&str, &str)> = vec![
            ("https://gelbooru.com/index.php?page=post&s=view&id=6250367&tags=rurudo", "Sensitive"),
            ("https://gelbooru.com/index.php?page=post&s=view&id=5558687&tags=rurudo", "Questionable"),
            ("https://gelbooru.com/index.php?page=post&s=view&id=5591747&tags=rurudo", "Explicit"),
        ];
        let mut tags: Vec<(Vec<PantsuTag>, &str)> = Vec::new();
        for link in links {
            tags.push((tag_finder::find_tags_gelbooru(link.0).await.unwrap(), link.1));
        }
        for tag in tags {
            assert!(tag.0.iter().any(|t| t.tag_name.eq(tag.1) && matches!(t.tag_type, PantsuTagType::Rating)))
        }
    }

    #[test]
    fn sort_sauce_matches() {
        let mut matches_list: Vec<SauceMatch> = vec![
            SauceMatch {
                link: String::from("a"),
                similarity: 50,
                resolution: (10,10),
            },
            SauceMatch {
                link: String::from("b"),
                similarity: 60,
                resolution: (5,5),
            },
            SauceMatch {
                link: String::from("c"),
                similarity: 51,
                resolution: (9,0),
            },
            SauceMatch {
                link: String::from("d"),
                similarity: 49,
                resolution: (20,20),
            },
            SauceMatch {
                link: String::from("e"),
                similarity: 50,
                resolution: (12,12),
            },
            SauceMatch {
                link: String::from("f"),
                similarity: 50,
                resolution: (12,0),
            },
            SauceMatch {
                link: String::from("g"),
                similarity: 50,
                resolution: (0,12),
            },

        ];
        matches_list.sort();
        let mut iter = matches_list.iter();
        assert_eq!(iter.next().unwrap().link, "d");
        assert_eq!(iter.next().unwrap().link, "g");
        assert_eq!(iter.next().unwrap().link, "a");
        assert_eq!(iter.next().unwrap().link, "f");
        assert_eq!(iter.next().unwrap().link, "e");
        assert_eq!(iter.next().unwrap().link, "c");
        assert_eq!(iter.next().unwrap().link, "b");

        assert!(matches_list[0].eq(&matches_list[0]));

        //for (idx, m) in matches_list.iter().enumerate() {
        //    println!("list {}: {}, {}, ({},{})", idx, m.link, m.similarity, m.resolution.0, m.resolution.1);
        //}
    }
}