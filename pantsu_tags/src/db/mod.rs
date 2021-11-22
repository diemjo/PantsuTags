use std::path::{Path, PathBuf};
use rusqlite::{Connection};
use crate::common::error;
use crate::common::error::Error;
use crate::common::image_handle::ImageHandle;
use crate::common::pantsu_tag::{PantsuTag, PantsuTagType};
use crate::file_handler;

mod db_calls;
mod sqlite_statements;
mod db_init;

pub struct PantsuDB {
    conn: Connection
}

impl PantsuDB {

    pub fn default() -> Result<PantsuDB, Error> {
        let mut data_dir = file_handler::get_data_dir();
        data_dir.push("pantsu_tags.db");
        PantsuDB::new(&data_dir)
    }

    pub fn new(db_path: &Path) -> Result<PantsuDB, Error> {
        if db_path.eq(Path::new("/")) {
            return Err(Error::InvalidDatabasePath(error::get_path(db_path)));
        }
        let path_buf = PathBuf::from(db_path);
        std::fs::create_dir_all(path_buf.parent().unwrap()).or_else(|e|
            Err(Error::DirectoryCreateError(e, error::get_path(db_path)))
        )?;
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
    pub fn add_file(&mut self, file: &ImageHandle) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        db_calls::add_file_to_file_list(&transaction, file)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn remove_file(&mut self, file: &ImageHandle) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        db_calls::remove_all_tags_from_file(&transaction, file)?;
        db_calls::remove_file_from_file_list(&transaction, file)?;
        db_calls::remove_unused_tags(&transaction)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn update_file_source(&mut self, file: &ImageHandle) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        db_calls::update_file_source(&transaction, file)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn get_file(&self, filename: &str) -> Result<Option<ImageHandle>, Error> {
        db_calls::get_file(&self.conn, filename)
    }

    pub fn get_all_files(&self) -> Result<Vec<ImageHandle>, Error> {
        db_calls::get_all_files(&self.conn)
    }

    pub fn get_files_with_tags(&self, included_tags: &Vec<String>) -> Result<Vec<ImageHandle>, Error> {
        if included_tags.len() == 0 {
            return self.get_all_files();
        }
        db_calls::get_files_with_tags(&self.conn, included_tags, &Vec::<String>::new())
    }

    pub fn get_files_with_tags_but(&self, included_tags: &Vec<String>, excluded_tags: &Vec<String>) -> Result<Vec<ImageHandle>, Error> {
        if included_tags.len() == 0 && excluded_tags.len() == 0 {
            return self.get_all_files();
        }
        db_calls::get_files_with_tags(&self.conn, included_tags, excluded_tags)
    }

    // file->tag
    pub fn add_tags(&mut self, file: &ImageHandle, tags: &Vec<PantsuTag>) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        db_calls::add_tags_to_tag_list(&transaction, tags)?;
        db_calls::add_tags_to_file_tags(&transaction, file, tags)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn add_file_and_tags(&mut self, file: &ImageHandle, tags: &Vec<PantsuTag>) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        db_calls::add_file_to_file_list(&transaction, file)?;
        db_calls::add_tags_to_tag_list(&transaction, tags)?;
        db_calls::add_tags_to_file_tags(&transaction, file, tags)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn remove_tags(&mut self, file: &ImageHandle, tags: &Vec<PantsuTag>) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        db_calls::remove_tags_from_file(&transaction, file, tags)?;
        db_calls::remove_unused_tags(&transaction)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn get_tags_for_file(&self, file: &ImageHandle) -> Result<Vec<PantsuTag>, Error> {
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
    use std::path::Path;
    use crate::common::pantsu_tag::{PantsuTag, PantsuTagType};
    use crate::common::error::Error;
    use crate::common::image_handle::ImageHandle;
    use crate::db::PantsuDB;

    use serial_test::serial;
    use crate::Sauce;
    use crate::Sauce::Match;

    #[test]
    #[serial]
    #[ignore]
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
        pdb.update_file_source(&ImageHandle::new(String::from(img.get_filename()), Sauce::Match(String::from("https://fake.url")))).unwrap();
        assert_eq!(pdb.get_file(img.get_filename()).unwrap().unwrap().get_sauce(), &Sauce::Match(String::from("https://fake.url")));
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
    fn db_get_files_with_tags() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        let tags_to_add = vec![
            "general:Haha".parse().unwrap(),
            "artist:Hehe".parse().unwrap(),
            "character:Hihi".parse().unwrap(),
        ];
        pdb.add_file_and_tags(&get_test_image(), &tags_to_add).unwrap();
        pdb.add_file_and_tags(&get_test_image2(), &tags_to_add).unwrap();
        pdb.add_tags(&get_test_image2(), &vec!["general:Huhu".parse().unwrap()]).unwrap();
        let files = pdb.get_files_with_tags(&vec![String::from("Haha")]).unwrap();
        assert_eq!(files, vec![get_test_image(), get_test_image2()]);
        let files = pdb.get_files_with_tags(&vec![String::from("Huhu")]).unwrap();
        assert_eq!(files, vec![get_test_image2()]);
        let files = pdb.get_files_with_tags_but(&Vec::new(), &vec![String::from("Huhu")]).unwrap();
        assert_eq!(files, vec![get_test_image()]);
        let files = pdb.get_files_with_tags_but(&vec![String::from("Haha")], &vec![String::from("Huhu")]).unwrap();
        assert_eq!(files, vec![get_test_image()]);
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
        match path {
            None => PantsuDB::new(&std::env::current_dir().unwrap().as_path().join("pantsu_tags.db")),
            Some(path) => {
                if path.is_dir() {
                    PantsuDB::new(&path.join("pantsu_tags.db"))
                } else {
                    PantsuDB::new(path)
                }
            }
        }
    }

    fn get_test_image() -> ImageHandle {
        ImageHandle::new(String::from("test_image_1db8ab6c94e95f36a9dd5bde347f6dd1.png"), Sauce::NotChecked)
    }

    fn get_test_image2() -> ImageHandle {
        ImageHandle::new(String::from("test_image_4f76b8d52983af1d28b1bf8d830d684e.png"), Match(String::from("http://real.url")) )
    }
}