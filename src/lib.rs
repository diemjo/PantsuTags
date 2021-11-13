pub mod sauce;
pub mod common;
pub mod db;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::Path;
    use crate::common::{PantsuTag, PantsuTagType};
    use crate::db::PantsuDB;

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
    fn add_tags_to_file() {
        let mut pdb = PantsuDB::new("pantsu_tags.db").unwrap();
        let op = pdb.add_tags(
            &String::from("file001.png"),
            &vec![
                PantsuTag{ tag_name: String::from("Haha"), tag_type: PantsuTagType::Generic },
                PantsuTag{ tag_name: String::from("Hehe"), tag_type: PantsuTagType::Artist },
                PantsuTag{ tag_name: String::from("Hihi"), tag_type: PantsuTagType::Character },
                PantsuTag{ tag_name: String::from("Hoho"), tag_type: PantsuTagType::Source },
                PantsuTag{ tag_name: String::from("Huhu"), tag_type: PantsuTagType::Generic },
            ]);
        match op {
            Ok(_) => { println!("SUCC") }
            Err(e) => { println!("ERR: {}", e.to_string());}
        }
        /*let files = pdb.get_files().unwrap();
        for file in files {
            println!("file: {}", file);
        }*/
    }
}
