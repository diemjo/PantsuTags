pub mod tag_finder;
pub mod sauce_finder;

pub struct SauceMatch {
    pub link: String,
    pub similarity: f32,
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::common::pantsu_tag::{PantsuTag, PantsuTagType};
    use crate::sauce::{sauce_finder, tag_finder};

    #[test]
    fn find_sauce() {
        let path = Path::new("test.png");
        let sauces = sauce_finder::find_sauce(&path).unwrap();
        assert_eq!(sauces[0].link, "http://gelbooru.com/index.php?page=post&s=list&md5=4f76b8d52983af1d28b1bf8d830d684e");
        assert_eq!(sauces[0].similarity, 96.0);
    }

    #[test]
    fn find_tag() {
        let url = "http://gelbooru.com/index.php?page=post&s=list&md5=4f76b8d52983af1d28b1bf8d830d684e";
        let tags = tag_finder::find_tags_gelbooru(url).unwrap();
        assert!(tags.iter().any(|tag| tag.tag_name.eq("loli") && matches!(tag.tag_type, PantsuTagType::Generic)));
        assert!(tags.iter().any(|tag| tag.tag_name.eq("stuffed dinosaur") && matches!(tag.tag_type, PantsuTagType::Generic)));
        assert!(tags.iter().any(|tag| tag.tag_name.eq("ichihaya") && matches!(tag.tag_type, PantsuTagType::Artist)));
        assert!(tags.iter().any(|tag| tag.tag_name.eq("awano iroha") && matches!(tag.tag_type, PantsuTagType::Character)));
        assert!(tags.iter().any(|tag| tag.tag_name.eq("original") && matches!(tag.tag_type, PantsuTagType::Source)));
        assert!(!tags.iter().any(|tag| tag.tag_name.eq("large breasts") && matches!(tag.tag_type, PantsuTagType::Source)));
    }

    #[test]
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
}