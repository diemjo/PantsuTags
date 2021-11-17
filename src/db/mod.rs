use rusqlite::{Connection};
use crate::common::error::Error;
use crate::common::image_file::ImageFile;
use crate::common::pantsu_tag::{PantsuTag, PantsuTagType};

mod db_calls;
mod sqlite_statements;
mod db_init;

pub struct PantsuDB {
    conn: Connection
}

impl PantsuDB {
    pub fn new(db_path: &str) -> Result<PantsuDB, Error> {
        let conn = db_init::open(db_path)?;
        Ok(PantsuDB { conn })
    }

    // WARNING: ALL DATA WILL BE LOST
    pub fn clear(&mut self) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        db_calls::clear_all_file_tags(&transaction)?;
        db_calls::clear_all_files(&transaction)?;
        db_calls::clear_all_tags(&transaction)?;

        transaction.commit()?;
        Ok(())
    }

    // file
    pub fn add_file(&mut self, file: &ImageFile) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        db_calls::add_file_to_file_list(&transaction, file)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn remove_file(&mut self, file: &ImageFile) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        db_calls::remove_all_tags_from_file(&transaction, file)?;
        db_calls::remove_file_from_file_list(&transaction, file)?;
        db_calls::remove_unused_tags(&transaction)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn update_file_source(&mut self, file: &ImageFile) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        db_calls::update_file_source(&transaction, file)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn get_file(&self, filename: &str) -> Result<ImageFile, Error> {
        db_calls::get_file(&self.conn, filename)
    }

    pub fn get_all_files(&self) -> Result<Vec<ImageFile>, Error> {
        db_calls::get_all_files(&self.conn)
    }

    pub fn get_files_with_tags(&self, tags: &Vec<PantsuTag>) -> Result<Vec<ImageFile>, Error> {
        if tags.len() == 0 {
            return self.get_all_files();
        }
        db_calls::get_files_with_tags(&self.conn, tags)
    }

    // file->tag
    pub fn add_tags(&mut self, file: &ImageFile, tags: &Vec<PantsuTag>) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        db_calls::add_tags_to_tag_list(&transaction, tags)?;
        db_calls::add_tags_to_file_tags(&transaction, file, tags)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn remove_tags(&mut self, file: &ImageFile, tags: &Vec<PantsuTag>) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        db_calls::remove_tags_from_file(&transaction, file, tags)?;
        db_calls::remove_unused_tags(&transaction)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn get_tags_for_file(&self, file: &ImageFile) -> Result<Vec<PantsuTag>, Error> {
        db_calls::get_tags_for_file(&self.conn, file)
    }

    // tags
    pub fn get_all_tags(&self) -> Result<Vec<PantsuTag>, Error> {
        db_calls::get_all_tags(&self.conn)
    }

    pub fn get_tags_with_types(&self, types: &Vec<PantsuTagType>) -> Result<Vec<PantsuTag>, Error> {
        db_calls::get_tags_with_types(&self.conn, types)
    }
}




#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use crate::common::pantsu_tag::{PantsuTag, PantsuTagType};
    use crate::common::error::Error;
    use crate::common::image_file::ImageFile;
    use crate::db::PantsuDB;

    use serial_test::serial;

    #[test]
    #[serial]
    fn db_add_file_twice() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        pdb.add_file(&get_test_image()).unwrap();
        assert!(matches!(pdb.add_file(&get_test_image()).unwrap_err(), Error::SQLPrimaryKeyError{..}));
    }

    #[test]
    #[serial]
    fn db_update_file_source() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        let img = &get_test_image();
        pdb.add_file(img).unwrap();
        pdb.update_file_source(&ImageFile{ filename: img.filename.clone(), file_source: Some(String::from("https://fake.url")) }).unwrap();
        assert_eq!(pdb.get_file(&img.filename).unwrap().file_source, Some(String::from("https://fake.url")));
    }

    #[test]
    #[serial]
    fn db_add_tags_to_file() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        pdb.add_file(&get_test_image()).unwrap();
        pdb.add_tags(
            &get_test_image(),
            &vec![
                "general:Haha".parse().unwrap(),
                "artist:Hehe".parse().unwrap(),
                "character:Hihi".parse().unwrap(),
                "source:Hoho".parse().unwrap(),
                "general:Huhu".parse().unwrap()
            ]).unwrap();
    }

    #[test]
    #[serial]
    fn db_add_and_remove_file() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        pdb.add_file(&get_test_image()).unwrap();
        pdb.add_tags(
            &get_test_image(),
            &vec![
                "general:Haha".parse().unwrap(),
                "artist:Hehe".parse().unwrap(),
                "character:Hihi".parse().unwrap(),
            ]).unwrap();
        let files1 = pdb.get_all_files().unwrap();
        pdb.remove_file(&get_test_image()).unwrap();
        let files2 = pdb.get_all_files().unwrap();
        assert_eq!(1, files1.len());
        assert_eq!(0, files2.len());
        println!("{:?}\n{:?}", files1, files2);
    }

    #[test]
    #[serial]
    fn db_add_and_remove_tags() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        pdb.add_file(&get_test_image()).unwrap();
        pdb.add_tags(
            &get_test_image(),
            &vec![
                "general:Haha".parse().unwrap(),
                "artist:Hehe".parse().unwrap(),
                "character:Hihi".parse().unwrap(),
            ]).unwrap();
        pdb.remove_tags(&get_test_image(), &vec!["general:Haha".parse().unwrap()]).unwrap();
        let tags = pdb.get_tags_for_file(&get_test_image()).unwrap();
        assert_eq!(&tags, &vec!["artist:Hehe".parse().unwrap(), "character:Hihi".parse().unwrap()]);
    }

    #[test]
    #[serial]
    fn db_get_tags_for_file() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        pdb.add_file(&get_test_image()).unwrap();
        pdb.add_file(&get_test_image2()).unwrap();
        pdb.add_tags(
            &get_test_image(),
            &vec![
                "general:Haha".parse().unwrap(),
                "artist:Hehe".parse().unwrap(),
                "character:Hoho".parse().unwrap(),
            ]).unwrap();
        pdb.add_tags(
            &get_test_image2(),
            &vec![
                "general:Haha".parse().unwrap(),
                "artist:Huhu".parse().unwrap(),
                "character:Höhö".parse().unwrap(),
            ]).unwrap();
        let tags = pdb.get_tags_for_file(&get_test_image2()).unwrap();
        assert_eq!(&tags, &vec!["general:Haha".parse().unwrap(), "artist:Huhu".parse().unwrap(), "character:Höhö".parse().unwrap()]);
    }

    #[test]
    #[serial]
    fn db_get_tags() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        assert_eq!(pdb.get_all_tags().unwrap().len(), 0);
        let tags_to_add = vec![
            "general:Haha".parse().unwrap(),
            "artist:Hehe".parse().unwrap(),
            "character:Hihi".parse().unwrap(),
        ];
        pdb.add_file(&get_test_image()).unwrap();
        pdb.add_tags(&get_test_image(), &tags_to_add).unwrap();
        let all_tags = pdb.get_all_tags().unwrap();
        assert_eq!(all_tags, tags_to_add);
    }

    #[test]
    #[serial]
    fn db_get_general_tags() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        assert_eq!(pdb.get_all_tags().unwrap().len(), 0);
        let tags_to_add: Vec<PantsuTag> = vec![
            "general:Haha".parse().unwrap(),
            "artist:Hehe".parse().unwrap(),
            "character:Hihi".parse().unwrap(),
            "general:Huhu".parse().unwrap()
        ];
        pdb.add_file(&get_test_image()).unwrap();
        pdb.add_tags(&get_test_image(), &tags_to_add).unwrap();
        let all_tags = pdb.get_tags_with_types(&vec![PantsuTagType::General]).unwrap();
        assert_eq!(all_tags, vec![
            "general:Haha".parse().unwrap(),
            "general:Huhu".parse().unwrap()
        ]);
    }

    #[test]
    #[serial]
    fn db_get_general_and_character_tags() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        assert_eq!(pdb.get_all_tags().unwrap().len(), 0);
        let tags_to_add = vec![
            "general:Haha".parse().unwrap(),
            "artist:Hehe".parse().unwrap(),
            "character:Hihi".parse().unwrap(),
            "general:Hoho".parse().unwrap()
        ];
        pdb.add_file(&get_test_image()).unwrap();
        pdb.add_tags(&get_test_image(), &tags_to_add).unwrap();
        let all_tags = pdb.get_tags_with_types(&vec![PantsuTagType::General, PantsuTagType::Character]).unwrap();
        assert_eq!(all_tags, vec![
            "general:Haha".parse().unwrap(),
            "character:Hihi".parse().unwrap(),
            "general:Hoho".parse().unwrap()
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
                    Err(Error::DirectoryCreateError(e, path.as_path().to_str().unwrap().to_string()))
                )?;
                Ok(path)
            },
            None => panic!("No valid home dir found")
        }
    }

    fn get_test_image() -> ImageFile {
        ImageFile { filename: String::from("file001.png"), file_source: None }
    }

    fn get_test_image2() -> ImageFile {
        ImageFile { filename: String::from("file002.png"), file_source: Some(String::from("http://real.url")) }
    }
}