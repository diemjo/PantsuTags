pub mod tag_finder;
pub mod sauce_finder;

pub struct SauceMatch {
    pub link: String,
    pub similarity: f32,
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::sauce;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    //#[ignore]
    fn find_sauce() {
        let path = Path::new("test.png");
        /*let file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", path.display(), why),
            Ok(file) => file,
        };
        */
        let sauces = sauce::sauce_finder::find_sauce(&path).unwrap();
        println!("found sauces:");
        for s in sauces {
            println!("link: {}, similarity: {}", s.link, s.similarity);
        }
        /*let res = sauce::sauce_finder::find_sauce(&path);
        if let Err(err) = res {
            println!("error:\n{}", err.to_string());
        }*/
    }

    #[test]
    fn find_tag() {
        let url = "https://gelbooru.com/index.php?page=post&s=list&md5=b3b2aa651df45f6cd74f9c45fb715c79";
        let tags = sauce::tag_finder::find_tags_gelbooru(url).unwrap();
        println!("Found {} tags: ", tags.len());
        for tag in tags {
            println!("{}, Category: {}", tag.tag_name, tag.tag_type.to_string());
        }
    }
}