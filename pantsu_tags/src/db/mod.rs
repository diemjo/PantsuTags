use std::path::{Path, PathBuf};
use rusqlite::{Connection};
use crate::common::error;
use crate::common::error::Error;
use crate::{file_handler};
use crate::db::transactions::{DeleteImagesTransaction, InsertImagesTransaction, SelectImagesTransaction, SelectImageTransaction, SelectTagsTransaction, UpdateImagesTransaction};

mod db_calls;
mod sqlite_statements;
mod db_init;
mod transactions;

pub enum AspectRatio {
    Any,
    Min(f32),
    Max(f32),
    Range(f32, f32)
}

pub(crate) enum SauceType {
    NotChecked,
    NotExisting,
    Existing,
    Any,
}

pub struct PantsuDB {
    conn: Connection
}

impl PantsuDB {

    pub fn default() -> Result<PantsuDB, Error> {
        let mut data_dir = file_handler::default_db_dir();
        data_dir.push("pantsu_tags.db");
        PantsuDB::new(&data_dir)
    }

    pub fn new(db_path: &Path) -> Result<PantsuDB, Error> {
        let mut path_buf = PathBuf::from(db_path);
        if path_buf.exists() && path_buf.is_dir() {
            path_buf.push("pantsu_tags.db");
        }
        std::fs::create_dir_all(path_buf.parent().unwrap()).or_else(|e|
            Err(Error::DirectoryCreateError(e, error::get_path(path_buf.as_path())))
        )?;
        let conn = db_init::open(path_buf.as_path())?;
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

    // select
    pub fn get_image_transaction<'a>(&'a self, filename: &'a str) -> SelectImageTransaction<'a> {
        SelectImageTransaction::new(&self.conn, filename)
    }

    pub fn get_images_transaction<'a>(&'a self) -> SelectImagesTransaction<'a> {
        SelectImagesTransaction::new(&self.conn)
    }

    pub fn get_tags_transaction<'a>(&'a self) -> SelectTagsTransaction<'a> {
        SelectTagsTransaction::new(&self.conn)
    }

    pub fn update_images_transaction<'a>(&'a mut self) -> UpdateImagesTransaction<'a> {
        UpdateImagesTransaction::new(&mut self.conn)
    }

    pub fn add_images_transaction<'a>(&'a mut self) -> InsertImagesTransaction<'a> {
        InsertImagesTransaction::new(&mut self.conn)
    }

    pub fn remove_image_transaction<'a>(&'a mut self) -> DeleteImagesTransaction<'a> {
        DeleteImagesTransaction::new(&mut self.conn)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use std::path::Path;
    use crate::common::error::Error;
    use crate::common::image_handle::ImageHandle;
    use crate::db::PantsuDB;

    use serial_test::serial;
    use crate::{PantsuTag, PantsuTagType, Sauce};
    use crate::Sauce::Match;

    #[test]
    #[serial]
    fn db_add_file_twice() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        add_test_image(&mut pdb).unwrap();
        assert!(
            matches!(pdb.add_images_transaction()
                .add_image(&get_test_image())
                .execute()
                .unwrap_err(),
                Error::SQLPrimaryKeyError{..}
            )
        );
    }

    #[test]
    #[serial]
    fn db_update_file_source() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        let img = &get_test_image();
        add_test_image(&mut pdb).unwrap();
        pdb.update_images_transaction()
            .for_image(get_test_image().get_filename())
            .update_sauce(&Match(String::from("https://fake.url")))
            .execute()
            .unwrap();
        assert_eq!(pdb.get_image_transaction(img.get_filename())
                       .execute()
                       .unwrap()
                       .unwrap()
                       .get_sauce(),
                   &Match(String::from("https://fake.url"))
        );
    }

    #[test]
    #[serial]
    fn db_add_tags_to_file() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        add_test_image(&mut pdb).unwrap();
        pdb.update_images_transaction()
            .for_image(&get_test_image2().get_filename())
            .add_tags(&vec![
                "general:Haha".parse().unwrap(),
                "artist:Hehe".parse().unwrap(),
                "character:Hihi".parse().unwrap(),
                "source:Hoho".parse().unwrap(),
                "general:Huhu".parse().unwrap()]
            )
            .execute()
            .unwrap();
    }

    #[test]
    #[serial]
    fn db_add_and_remove_file() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        add_test_image(&mut pdb).unwrap();
        add_test_image2(&mut pdb).unwrap();
        pdb.update_images_transaction()
            .for_image(&get_test_image().get_filename())
            .add_tags(&vec![
                "general:Haha".parse().unwrap(),
                "artist:Hehe".parse().unwrap(),
                "character:Hihi".parse().unwrap(),
            ])
            .execute()
            .unwrap();
        let files1 = pdb.get_images_transaction().execute().unwrap();
        pdb.remove_image_transaction().remove_image(get_test_image().get_filename()).execute().unwrap();
        let files2 = pdb.get_images_transaction().execute().unwrap();
        assert_eq!(2, files1.len());
        assert_eq!(1, files2.len());
        println!("{:?}\n{:?}", files1, files2);
    }

    #[test]
    #[serial]
    fn db_add_and_remove_tags() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        add_test_image(&mut pdb).unwrap();
        pdb.update_images_transaction()
            .for_image(&get_test_image().get_filename())
            .add_tags(&vec![
                "general:Haha".parse().unwrap(),
                "artist:Hehe".parse().unwrap(),
                "character:Hihi".parse().unwrap(),
            ])
            .execute()
            .unwrap();
        pdb.update_images_transaction()
            .for_image(get_test_image().get_filename())
            .remove_tags(
                &vec!["Haha".to_string()]
            ).execute()
            .unwrap();
        let tags = pdb.get_tags_transaction()
            .for_image(get_test_image().get_filename())
            .execute()
            .unwrap();
        assert_eq!(&tags, &vec!["artist:Hehe".parse().unwrap(), "character:Hihi".parse().unwrap()]);
    }

    #[test]
    #[serial]
    fn db_get_tags_for_file() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        add_test_image(&mut pdb).unwrap();
        add_test_image2(&mut pdb).unwrap();
        pdb.update_images_transaction()
            .for_image(get_test_image().get_filename())
            .add_tags(&vec![
                "general:Haha".parse().unwrap(),
                "artist:Hehe".parse().unwrap(),
                "character:Hoho".parse().unwrap(),
            ])
            .execute()
            .unwrap();
        pdb.update_images_transaction()
            .for_image(get_test_image2().get_filename())
            .add_tags(&vec![
                "general:Haha".parse().unwrap(),
                "artist:Huhu".parse().unwrap(),
                "character:Höhö".parse().unwrap(),
            ])
            .execute()
            .unwrap();
        let tags = pdb.get_tags_transaction()
            .for_image(get_test_image2().get_filename())
            .execute()
            .unwrap();
        assert_eq!(HashSet::<PantsuTag>::from_iter(tags),
                   HashSet::<PantsuTag>::from_iter(vec!["general:Haha".parse().unwrap(), "artist:Huhu".parse().unwrap(), "character:Höhö".parse().unwrap()])
        );
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
        add_test_image(&mut pdb).unwrap();
        pdb.update_images_transaction()
            .for_image(get_test_image().get_filename())
            .add_tags(&tags_to_add)
            .execute()
            .unwrap();
        let all_tags = pdb.get_tags_transaction()
            .execute()
            .unwrap();
        assert_eq!(HashSet::<PantsuTag>::from_iter(all_tags),
                   HashSet::<PantsuTag>::from_iter(tags_to_add));
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
        add_test_image(&mut pdb).unwrap();
        pdb.update_images_transaction()
            .for_image(get_test_image().get_filename())
            .add_tags(&tags_to_add)
            .execute()
            .unwrap();        add_test_image2(&mut pdb).unwrap();
        pdb.update_images_transaction()
            .for_image(get_test_image2().get_filename())
            .add_tags(&tags_to_add)
            .execute()
            .unwrap();
        pdb.update_images_transaction()
            .for_image(get_test_image2().get_filename())
            .add_tags(&vec!["general:Huhu".parse().unwrap()])
            .execute()
            .unwrap();
        let files = pdb.get_images_transaction()
            .including_tag("Haha")
            .execute()
            .unwrap();
        assert_eq!(files, vec![get_test_image(), get_test_image2()]);
        let files = pdb.get_images_transaction()
            .including_tag("Huhu")
            .execute()
            .unwrap();
        assert_eq!(files, vec![get_test_image2()]);
        let files = pdb.get_images_transaction()
            .excluding_tag("Huhu")
            .execute()
            .unwrap();
        assert_eq!(files, vec![get_test_image()]);
        let files = pdb.get_images_transaction()
            .including_tag("Haha")
            .excluding_tag("Huhu")
            .execute()
            .unwrap();
        assert_eq!(files, vec![get_test_image()]);
    }

    #[test]
    #[serial]
    fn db_get_general_tags() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        assert_eq!(pdb.get_tags_transaction().execute().unwrap().len(), 0);
        let tags_to_add: Vec<PantsuTag> = vec![
            "general:Haha".parse().unwrap(),
            "artist:Hehe".parse().unwrap(),
            "character:Hihi".parse().unwrap(),
            "general:Huhu".parse().unwrap()
        ];
        add_test_image(&mut pdb).unwrap();
        pdb.update_images_transaction()
            .for_image(get_test_image().get_filename())
            .add_tags(&tags_to_add)
            .execute()
            .unwrap();
        let all_tags = pdb.get_tags_transaction()
            .with_types(&vec![PantsuTagType::General])
            .execute()
            .unwrap();
        assert_eq!(
            HashSet::<PantsuTag>::from_iter(all_tags),
            HashSet::<PantsuTag>::from_iter(vec![
                "general:Haha".parse().unwrap(),
                "general:Huhu".parse().unwrap()
            ])
        );
    }

    #[test]
    #[serial]
    fn db_get_general_and_character_tags() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        assert_eq!(pdb.get_tags_transaction().execute().unwrap().len(), 0);
        let tags_to_add = vec![
            "general:Haha".parse().unwrap(),
            "artist:Hehe".parse().unwrap(),
            "character:Hihi".parse().unwrap(),
            "general:Hoho".parse().unwrap()
        ];
        add_test_image(&mut pdb).unwrap();
        pdb.update_images_transaction()
            .for_image(&get_test_image().get_filename())
            .add_tags(&tags_to_add)
            .execute()
            .unwrap();
        let all_tags = pdb.get_tags_transaction()
            .with_types(&vec![PantsuTagType::General, PantsuTagType::Character])
            .execute()
            .unwrap();
        assert_eq!(
            HashSet::<PantsuTag>::from_iter(all_tags),
            HashSet::<PantsuTag>::from_iter(vec![
                "general:Haha".parse().unwrap(),
                "character:Hihi".parse().unwrap(),
                "general:Hoho".parse().unwrap()
            ])
        );
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

    fn add_test_image(pdb: &mut PantsuDB) -> Result<(), Error> {
        pdb.add_images_transaction()
            .add_image(&get_test_image())
            .execute()
    }

    fn add_test_image2(pdb: &mut PantsuDB) -> Result<(), Error> {
        pdb.add_images_transaction()
            .add_image(&get_test_image2())
            .execute()
    }

    fn get_test_image() -> ImageHandle {
        ImageHandle::new(String::from("1b64e362cdf968d9-c1fc07e23e05e2fc0be39ce8cc88f8044fcf.jpg"), Sauce::NotChecked, (100, 200))
    }

    fn get_test_image2() -> ImageHandle {
        ImageHandle::new(String::from("c3811874f801fd63-03f07d07b03b05f3370670df0db0ff037037.jpg"), Match(String::from("http://real.url")), (0, 0))
    }
}