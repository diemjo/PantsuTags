use std::path::{Path, PathBuf};
use rusqlite::{Connection};
use crate::common::error;
use crate::common::error::{Error, Result};
use crate::{file_handler};
use crate::db::transactions::PantsuDBTransaction;

mod db_calls;
mod sqlite_statements;
mod db_init;
pub mod transactions;

pub struct PantsuDB {
    conn: Connection
}

impl PantsuDB {

    pub fn default() -> Result<PantsuDB> {
        let mut data_dir = file_handler::get_data_dir();
        data_dir.push("pantsu_tags.db");
        PantsuDB::new(&data_dir)
    }

    pub fn new(db_path: &Path) -> Result<PantsuDB> {
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
    pub fn clear(&mut self) -> Result<()> {
        let transaction = self.conn.transaction()?;

        db_calls::clear_all_file_tags(&transaction)?;
        db_calls::clear_all_files(&transaction)?;
        db_calls::clear_all_tags(&transaction)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn execute<R, T: PantsuDBTransaction<R>>(&mut self, pantsu_transaction: T) -> Result<R> {
        let mut transaction = self.conn.transaction()?;
        let res = pantsu_transaction.execute(&mut transaction)?;
        transaction.commit()?;
        Ok(res)
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
        pdb.add_file_with_source(&get_test_image()).unwrap();
        assert!(matches!(pdb.add_file_with_source(&get_test_image()).unwrap_err(), Error::SQLPrimaryKeyError{..}));
    }

    #[test]
    #[serial]
    fn db_update_file_source() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        let img = &get_test_image();
        pdb.add_file_with_source(img).unwrap();
        pdb.update_file_source(get_test_image(), Sauce::Match(String::from("https://fake.url"))).unwrap();
        assert_eq!(pdb.get_file(img.get_filename()).unwrap().unwrap().get_sauce(), &Sauce::Match(String::from("https://fake.url")));
    }

    #[test]
    #[serial]
    fn db_add_tags_to_file() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        pdb.add_file_with_source(&get_test_image()).unwrap();
        pdb.add_tags_to_file(
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
        pdb.add_file_with_source(&get_test_image()).unwrap();
        pdb.add_tags_to_file(
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
        pdb.add_file_with_source(&get_test_image()).unwrap();
        pdb.add_tags_to_file(
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
        pdb.add_file_with_source(&get_test_image()).unwrap();
        pdb.add_file_with_source(&get_test_image2()).unwrap();
        pdb.add_tags_to_file(
            &get_test_image(),
            &vec![
                "general:Haha".parse().unwrap(),
                "artist:Hehe".parse().unwrap(),
                "character:Hoho".parse().unwrap(),
            ]).unwrap();
        pdb.add_tags_to_file(
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
        pdb.add_file_with_source(&get_test_image()).unwrap();
        pdb.add_tags_to_file(&get_test_image(), &tags_to_add).unwrap();
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
        pdb.add_file_with_source(&get_test_image()).unwrap();
        pdb.add_tags_to_file(&get_test_image(), &tags_to_add).unwrap();
        pdb.add_file_with_source(&get_test_image2()).unwrap();
        pdb.add_tags_to_file(&get_test_image2(), &tags_to_add).unwrap();
        pdb.add_tags_to_file(&get_test_image2(), &vec!["general:Huhu".parse().unwrap()]).unwrap();
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
        pdb.add_file_with_source(&get_test_image()).unwrap();
        pdb.add_tags_to_file(&get_test_image(), &tags_to_add).unwrap();
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
        pdb.add_file_with_source(&get_test_image()).unwrap();
        pdb.add_tags_to_file(&get_test_image(), &tags_to_add).unwrap();
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
        ImageHandle::new(String::from("test_image_1db8ab6c94e95f36a9dd5bde347f6dd1.png"), Sauce::NotChecked, (0, 0))
    }

    fn get_test_image2() -> ImageHandle {
        ImageHandle::new(String::from("test_image_4f76b8d52983af1d28b1bf8d830d684e.png"), Match(String::from("http://real.url")), (0, 0))
    }
}