use std::cmp::Ordering;

pub mod tag_finder;
pub mod sauce_finder;

#[derive(Eq)]
pub struct SauceMatch {
    pub link: String,
    pub similarity: i32,
    pub resolution: (i32, i32),
}

impl Ord for SauceMatch {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.similarity.cmp(&other.similarity) {
            Ordering::Equal => self.resolution.cmp(&other.resolution),
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
    use std::path::Path;
    use crate::common::pantsu_tag::{PantsuTag, PantsuTagType};
    use crate::sauce::{sauce_finder, tag_finder};
    use crate::SauceMatch;

    #[test]
    fn find_sauce() {
        let path = Path::new("test.png");
        let sauces = sauce_finder::find_sauce(&path).unwrap();
        assert_eq!(sauces[0].link, "http://gelbooru.com/index.php?page=post&s=list&md5=4f76b8d52983af1d28b1bf8d830d684e");
        assert_eq!(sauces[0].similarity, 96);
        assert_eq!(sauces[0].resolution, (533, 745));
    }

    #[test]
    fn find_tag() {
        let url = "http://gelbooru.com/index.php?page=post&s=list&md5=4f76b8d52983af1d28b1bf8d830d684e";
        let tags = tag_finder::find_tags_gelbooru(url).unwrap();
        assert!(tags.iter().any(|tag| tag.tag_name.eq("loli") && matches!(tag.tag_type, PantsuTagType::General)));
        assert!(tags.iter().any(|tag| tag.tag_name.eq("stuffed dinosaur") && matches!(tag.tag_type, PantsuTagType::General)));
        assert!(tags.iter().any(|tag| tag.tag_name.eq("ichihaya") && matches!(tag.tag_type, PantsuTagType::Artist)));
        assert!(tags.iter().any(|tag| tag.tag_name.eq("awano iroha") && matches!(tag.tag_type, PantsuTagType::Character)));
        assert!(tags.iter().any(|tag| tag.tag_name.eq("original") && matches!(tag.tag_type, PantsuTagType::Source)));
        assert!(tags.iter().any(|tag| tag.tag_name.eq("Questionable") && matches!(tag.tag_type, PantsuTagType::Rating)));
        assert!(!tags.iter().any(|tag| tag.tag_name.eq("large breasts") && matches!(tag.tag_type, PantsuTagType::Source)));
    }

    #[test]
    #[ignore]
    fn find_multiple_tags() {
        let links: Vec<&str> = vec![
            "https://gelbooru.com/index.php?page=post&s=list&md5=b3b2aa651df45f6cd74f9c45fb715c79",
            "https://gelbooru.com/index.php?page=post&s=view&id=6261499",
            "https://gelbooru.com/index.php?page=post&s=view&id=3160369",
            "https://gelbooru.com/index.php?page=post&s=view&id=5276383",
        ];
        let mut tags: Vec<Vec<PantsuTag>> = Vec::new();
        for link in links {
            tags.push(tag_finder::find_tags_gelbooru(link).unwrap());
        }
        for tag in tags {
            assert!(tag.iter().any(|t| t.tag_name.eq("original") && matches!(t.tag_type, PantsuTagType::Source)))
        }
    }

    #[test]
    fn find_tags_rating() {
        let links: Vec<(&str, &str)> = vec![
            ("https://gelbooru.com/index.php?page=post&s=view&id=6250367&tags=rurudo", "Safe"),
            ("https://gelbooru.com/index.php?page=post&s=view&id=5558687&tags=rurudo", "Questionable"),
            ("https://gelbooru.com/index.php?page=post&s=view&id=5591747&tags=rurudo", "Explicit"),
        ];
        let mut tags: Vec<(Vec<PantsuTag>, &str)> = Vec::new();
        for link in links {
            tags.push((tag_finder::find_tags_gelbooru(link.0).unwrap(), link.1));
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