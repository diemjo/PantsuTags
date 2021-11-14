pub mod sauce;
pub mod common;
pub mod db;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::Path;

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
}

#[cfg(test)]
mod db_tests {
    use std::path::{Path, PathBuf};
    use crate::common::{PantsuTag, PantsuTagType};
    use crate::common::error::Error;
    use crate::db::PantsuDB;

    #[test]
    fn db_add_tags_to_file() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.add_tags(
            "file001.png",
            &vec![
                "generic:Haha".parse().unwrap(),
                "artist:Hehe".parse().unwrap(),
                "character:Hihi".parse().unwrap(),
                "source:Hoho".parse().unwrap(),
                "generic:Huhu".parse().unwrap()
        ]).unwrap();
    }

    #[test]
    fn db_add_and_remove_file() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        pdb.add_tags(
            "file001.png",
            &vec![
                "generic:Haha".parse().unwrap(),
                "artist:Hehe".parse().unwrap(),
                "character:Hihi".parse().unwrap(),
        ]).unwrap();
        let files1 = pdb.get_files().unwrap();
        pdb.remove_file("file001.png").unwrap();
        let files2 = pdb.get_files().unwrap();
        assert_eq!(1, files1.len());
        assert_eq!(0, files2.len());
        println!("{:?}\n{:?}", files1, files2);
    }

    #[test]
    fn db_get_tags() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        assert_eq!(pdb.get_all_tags().unwrap().len(), 0);
        let tags_to_add = vec![
            "generic:Haha".parse().unwrap(),
            "artist:Hehe".parse().unwrap(),
            "character:Hihi".parse().unwrap(),
        ];
        pdb.add_tags("file001.png", &tags_to_add).unwrap();
        let all_tags = pdb.get_all_tags().unwrap();
        assert_eq!(all_tags, tags_to_add);
    }

    #[test]
    fn db_get_generic_tags() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        assert_eq!(pdb.get_all_tags().unwrap().len(), 0);
        let tags_to_add: Vec<PantsuTag> = vec![
            "generic:Haha".parse().unwrap(),
            "artist:Hehe".parse().unwrap(),
            "character:Hihi".parse().unwrap(),
            "generic:Huhu".parse().unwrap()
        ];
        pdb.add_tags("file001.png", &tags_to_add).unwrap();
        let all_tags = pdb.get_tags_with_types(&vec![PantsuTagType::Generic]).unwrap();
        assert_eq!(all_tags, vec![
            "generic:Haha".parse().unwrap(),
            "generic:Huhu".parse().unwrap()
        ]);
    }

    #[test]
    fn db_get_generic_and_character_tags() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        assert_eq!(pdb.get_all_tags().unwrap().len(), 0);
        let tags_to_add = vec![
            "generic:Haha".parse().unwrap(),
            "artist:Hehe".parse().unwrap(),
            "character:Hihi".parse().unwrap(),
            "generic:Hoho".parse().unwrap()
        ];
        pdb.add_tags("file001.png", &tags_to_add).unwrap();
        let all_tags = pdb.get_tags_with_types(&vec![PantsuTagType::Generic, PantsuTagType::Character]).unwrap();
        assert_eq!(all_tags, vec![
            "generic:Haha".parse().unwrap(),
            "character:Hihi".parse().unwrap(),
            "generic:Hoho".parse().unwrap()
        ]);
    }

    fn get_pantsu_db(path: Option<&Path>) -> Result<PantsuDB, Error> {
        let mut db_path : PathBuf = match path {
            Some(path) => PathBuf::from(path),
            None => get_or_create_data_dir().unwrap()
        };
        db_path.push("pantsu_tags.db");
        Ok(PantsuDB::new(db_path.as_path().to_str().unwrap())?)
    }

    fn get_or_create_data_dir() -> Result<PathBuf, Error> {
        match directories::ProjectDirs::from("moe", "karpador", "PantsuTags") {
            Some(project_dir) => {
                let mut path = PathBuf::new();
                path.push(project_dir.data_dir());
                std::fs::create_dir_all(&path).or_else(|e|
                    Err(Error::FilesystemError(e, path.as_path().to_str().unwrap().to_string()))
                )?;
                Ok(path)
            },
            None => panic!("No valid home dir found")
        }
    }
}