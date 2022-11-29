use std::path::{Path, PathBuf};
use rusqlite::{Connection};

use crate::common::error::Result;
use crate::{common, Error, ImageHandle};
use crate::db::transactions::{DeleteImagesTransaction, InsertImagesTransaction, SelectImagesTransaction, SelectImageTransaction, SelectTagsTransaction, SelectImageTagsTransaction, UpdateImagesTransaction};

mod db_calls;
mod sqlite_statements;
mod db_init;
mod transactions;
mod db_import_export;
pub mod sort;

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

    /*pub fn default() -> Result<PantsuDB> {
        let mut data_dir = file_handler::default_db_dir();
        data_dir.push("pantsu_tags.db");
        PantsuDB::new(&data_dir)
    }*/

    pub fn new(db_path: &Path) -> Result<PantsuDB> {
        let mut path_buf = PathBuf::from(db_path);
        if path_buf.exists() && path_buf.is_dir() {
            path_buf.push("pantsu_tags.db");
        }
        std::fs::create_dir_all(path_buf.parent().unwrap()).or_else(|e|
            Err(Error::DirectoryCreateError(e, common::get_path(path_buf.as_path())))
        )?;
        let conn = db_init::open(path_buf.as_path())?;
        Ok(PantsuDB { conn })
    }

    pub fn get_db_version(&self) -> Result<usize> {
        db_calls::db_version(&self.conn)
    }

    // WARNING: ALL DATA WILL BE LOST
    pub fn clear(&mut self) -> Result<()> {
        let transaction = self.conn.transaction()?;

        db_calls::clear_all_image_tags(&transaction)?;
        db_calls::clear_all_images(&transaction)?;
        db_calls::clear_all_tags(&transaction)?;

        transaction.commit()?;
        Ok(())
    }

    // select
    pub fn get_image_transaction<'a>(&'a self, image: &'a ImageHandle) -> SelectImageTransaction<'a> {
        SelectImageTransaction::new(&self.conn, image)
    }

    pub fn get_images_transaction<'a>(&'a self) -> SelectImagesTransaction<'a> {
        SelectImagesTransaction::new(&self.conn)
    }

    pub fn get_tags_transaction<'a>(&'a self) -> SelectTagsTransaction<'a> {
        SelectTagsTransaction::new(&self.conn)
    }

    pub fn get_image_tags_transaction<'a>(&'a self, image: &'a ImageHandle) -> SelectImageTagsTransaction<'a> {
        SelectImageTagsTransaction::new(&self.conn, image)
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

    pub fn import_tags(&mut self, import_file_path: &Path) -> Result<()> {
        db_import_export::import_tags(self, import_file_path)
    }

    pub fn export_tags(&mut self, export_file_path: &Path) -> Result<()> {
        db_import_export::export_tags(self, export_file_path)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use std::path::{Path, PathBuf};
    use crate::common::error::Error;
    use crate::common::image_handle::ImageHandle;
    use crate::db::PantsuDB;

    use serial_test::serial;
    use crate::{PantsuTag, PantsuTagType, Sauce, sauce};

    #[test]
    #[serial]
    fn db_add_file_twice() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        add_test_image(&mut pdb).unwrap();
        let res = add_test_image(&mut pdb).unwrap_err();
        assert!( matches!(res, Error::SQLPrimaryKeyError{..}) );
    }

    #[test]
    #[serial]
    fn db_update_file_source() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        let img = &get_test_image();
        add_test_image(&mut pdb).unwrap();
        pdb.update_images_transaction()
            .for_image(&img)
            .update_sauce(&Sauce::Match(sauce::url_from_str("https://fake.url").unwrap()))
            .execute()
            .unwrap();
        assert_eq!(pdb.get_image_transaction(img)
                       .execute()
                       .unwrap()
                       .unwrap()
                       .get_sauce(),
                   &Sauce::Match(sauce::url_from_str("https://fake.url").unwrap())
        );
    }

    #[test]
    #[serial]
    fn db_add_tags_to_file() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        add_test_image(&mut pdb).unwrap();
        let img2 = get_test_image2();
        pdb.update_images_transaction()
            .for_image(&img2)
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
        let img = get_test_image();
        pdb.update_images_transaction()
            .for_image(&img)
            .add_tags(&vec![
                "general:Haha".parse().unwrap(),
                "artist:Hehe".parse().unwrap(),
                "character:Hihi".parse().unwrap(),
            ])
            .execute()
            .unwrap();
        let files1 = pdb.get_images_transaction().execute().unwrap();
        pdb.remove_image_transaction().remove_image(&img).execute().unwrap();
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
        let img = get_test_image();
        pdb.update_images_transaction()
            .for_image(&img)
            .add_tags(&vec![
                "general:Haha".parse().unwrap(),
                "artist:Hehe".parse().unwrap(),
                "character:Hihi".parse().unwrap(),
            ])
            .execute()
            .unwrap();
        pdb.update_images_transaction()
            .for_image(&img)
            .remove_tags(
                &vec!["general:Haha".parse().unwrap()]
            ).execute()
            .unwrap();
        let tags = pdb.get_image_tags_transaction(&img)
            .execute()
            .unwrap()
            .into_iter()
            .map(|t| t.tag)
            .collect::<Vec<_>>();
        assert_eq!(HashSet::<&PantsuTag>::from_iter(&tags),
                   HashSet::<&PantsuTag>::from_iter(&vec!["artist:Hehe".parse().unwrap(), "character:Hihi".parse().unwrap()])
                );
    }

    #[test]
    #[serial]
    fn db_get_tags_for_file() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();
        add_test_image(&mut pdb).unwrap();
        add_test_image2(&mut pdb).unwrap();
        let img = get_test_image();
        let img2 = get_test_image2();
        pdb.update_images_transaction()
            .for_image(&img)
            .add_tags(&vec![
                "general:Haha".parse().unwrap(),
                "artist:Hehe".parse().unwrap(),
                "character:Hoho".parse().unwrap(),
            ])
            .execute()
            .unwrap();
        pdb.update_images_transaction()
            .for_image(&img2)
            .add_tags(&vec![
                "general:Haha".parse().unwrap(),
                "artist:Huhu".parse().unwrap(),
                "character:Höhö".parse().unwrap(),
            ])
            .execute()
            .unwrap();
        let tags = pdb.get_image_tags_transaction(&img2)
            .execute()
            .unwrap()
            .into_iter()
            .map(|t| t.tag)
            .collect::<Vec<_>>();
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
        let img = get_test_image();
        pdb.update_images_transaction()
            .for_image(&img)
            .add_tags(&tags_to_add)
            .execute()
            .unwrap();
        let tags_to_add2 = vec![
            "general:Haha".parse().unwrap(),
            "artist:Hehe".parse().unwrap(),
            "rating:Huhu".parse().unwrap(),
        ];
        add_test_image2(&mut pdb).unwrap();
        let img2 = get_test_image2();
        pdb.update_images_transaction()
            .for_image(&img2)
            .add_tags(&tags_to_add2)
            .execute()
            .unwrap();
        let all_tags = pdb.get_tags_transaction()
            .execute()
            .unwrap();
        assert_eq!(HashSet::<PantsuTag>::from_iter(all_tags),
                   HashSet::<PantsuTag>::from_iter(vec![
                    "general:Haha".parse().unwrap(),
                    "artist:Hehe".parse().unwrap(),
                    "character:Hihi".parse().unwrap(),
                    "rating:Huhu".parse().unwrap(),
                    ])
        );
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
        let img = get_test_image();
        pdb.update_images_transaction()
            .for_image(&img)
            .add_tags(&tags_to_add)
            .execute()
            .unwrap();
        add_test_image2(&mut pdb).unwrap();
        let img2 = get_test_image2();
        pdb.update_images_transaction()
            .for_image(&img2)
            .add_tags(&tags_to_add)
            .execute()
            .unwrap();
        pdb.update_images_transaction()
            .for_image(&img2)
            .add_tags(&vec!["general:Huhu".parse().unwrap()])
            .execute()
            .unwrap();
        let files = pdb.get_images_transaction()
            .including_tag(&"general:Haha".parse().unwrap())
            .execute()
            .unwrap();
        assert_eq!(files.iter().map(|i| i.get_image()).collect::<Vec<&ImageHandle>>(), vec![&img, &img2]);
        let files = pdb.get_images_transaction()
            .including_tag(&"general:Huhu".parse().unwrap())
            .execute()
            .unwrap();
        assert_eq!(files.iter().map(|i| i.get_image()).collect::<Vec<&ImageHandle>>(), vec![&img2]);
        let files = pdb.get_images_transaction()
            .excluding_tag(&"general:Huhu".parse().unwrap())
            .execute()
            .unwrap();
        assert_eq!(files.iter().map(|i| i.get_image()).collect::<Vec<&ImageHandle>>(), vec![&img]);
        let files = pdb.get_images_transaction()
            .including_tag(&"general:Haha".parse().unwrap())
            .excluding_tag(&"general:Huhu".parse().unwrap())
            .execute()
            .unwrap();
        assert_eq!(files.iter().map(|i| i.get_image()).collect::<Vec<&ImageHandle>>(), vec![&img]);
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
        let img = get_test_image();
        pdb.update_images_transaction()
            .for_image(&img)
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
        let img = get_test_image();
        pdb.update_images_transaction()
            .for_image(&img)
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

    #[test]
    #[serial]
    fn db_import_file() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();

        let tags_to_add = vec![
            "general:Haha".parse().unwrap(),
            "artist:Hehe".parse().unwrap(),
            "character:Hihi".parse().unwrap(),
            "general:Hoho".parse().unwrap()
        ];
        add_test_image(&mut pdb).unwrap();
        let img = get_test_image();
        pdb.update_images_transaction()
            .for_image(&img)
            .add_tags(&tags_to_add)
            .execute()
            .unwrap();

        add_test_image2(&mut pdb).unwrap();
        let img2 = get_test_image2();

        let file = PathBuf::from("./test/test_db_import.txt");

        pdb.import_tags(file.as_path()).unwrap();

        let imgi1 = pdb.get_image_transaction(&img).execute().unwrap().unwrap();
        let sauce1 = imgi1.get_sauce();
        assert_eq!(sauce1, &Sauce::Match(sauce::url_from_str("http://domain.found.hehe/cool/tags?cool=yes").unwrap()));

        let imgi2 = pdb.get_image_transaction(&img2).execute().unwrap().unwrap();
        let sauce2 = imgi2.get_sauce();
        assert_eq!(sauce2, &Sauce::Match(sauce::url_from_str("http://real.url").unwrap()));

        let image = pdb.get_image_transaction(&ImageHandle::new("00001874f801fd63-03f07d07b03b05f3370670df0db0ff031111.jpg".to_string()).unwrap()).execute().unwrap();
        assert_eq!(image, None);
    }

    #[test]
    #[serial]
    fn db_import_file_error() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();

        let file = PathBuf::from("./test/test_db_import_fail.txt");
        assert!(match pdb.import_tags(file.as_path()).unwrap_err() {
            e @Error::DatabaseVersionMismatch(_, _, _) => { println!("{:?}", e); true },
            _ => false
        });
    }

    #[test]
    #[serial]
    fn db_import_file_error2() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();

        let file = PathBuf::from("./test/test_db_import_fail2.txt");
        assert!(match pdb.import_tags(file.as_path()).unwrap_err() {
            e @Error::InvalidImportFileFormat(_, _) => { println!("{:?}", e); true },
            _ => false
        });
    }

    #[test]
    #[serial]
    fn db_export_file() {
        let mut pdb = get_pantsu_db(Some(std::env::current_dir().unwrap().as_path())).unwrap();
        pdb.clear().unwrap();

        let tags_to_add1 = vec![
            "general:Haha".parse().unwrap(),
            "artist:Hehe".parse().unwrap(),
            "character:Hihi".parse().unwrap(),
            "general:Hoho".parse().unwrap()
        ];
        let sauce1_update = Sauce::Match(sauce::url_from_str("https://export.url.com/final").unwrap());
        add_test_image(&mut pdb).unwrap();
        let img = get_test_image();
        pdb.update_images_transaction()
            .for_image(&img)
            .add_tags(&tags_to_add1)
            .update_sauce(&sauce1_update)
            .execute()
            .unwrap();

        let tags_to_add2 = vec![
            "general:Haha2".parse().unwrap(),
            "artist:Hehe2".parse().unwrap(),
            "character:Hihi2".parse().unwrap(),
        ];
        add_test_image2(&mut pdb).unwrap();
        let img2 = get_test_image2();
        pdb.update_images_transaction()
            .for_image(&img2)
            .add_tags(&tags_to_add2)
            .execute()
            .unwrap();
        add_test_image3(&mut pdb).unwrap();

        let file = PathBuf::from("./test/test_db_export.txt");

        pdb.export_tags(file.as_path()).unwrap();
        
        pdb.clear().unwrap();
        add_test_image(&mut pdb).unwrap();
        add_test_image2(&mut pdb).unwrap();
        pdb.import_tags(file.as_path()).unwrap();

        let imgi1 = pdb.get_image_transaction(&img).execute().unwrap().unwrap();
        let sauce1 = imgi1.get_sauce();
        assert_eq!(sauce1, &sauce1_update);

        let imgi2 = pdb.get_image_transaction(&img2).execute().unwrap().unwrap();
        let sauce2 = imgi2.get_sauce();
        assert_eq!(sauce2, &Sauce::Match(sauce::url_from_str("http://real.url").unwrap()));

        let img3 = get_test_image3();
        let imgi3 = pdb.get_image_transaction(&img3).execute().unwrap();
        assert_eq!(imgi3, None);
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
        let img = get_test_image();
        pdb.add_images_transaction()
            .add_image(&img, (100, 200))
            .execute()?;
        pdb.update_images_transaction()
            .for_image(&img)
            .update_sauce(&Sauce::NotChecked)
            .execute()?;
        Ok(())
    }

    fn add_test_image2(pdb: &mut PantsuDB) -> Result<(), Error> {
        let img = get_test_image2();
        pdb.add_images_transaction()
            .add_image(&img, (0, 0))
            .execute()?;
        pdb.update_images_transaction()
            .for_image(&img)
            .update_sauce(&Sauce::Match(sauce::url_from_str("http://real.url").unwrap()))
            .execute()?;
        Ok(())
    }

    fn add_test_image3(pdb: &mut PantsuDB) -> Result<(), Error> {
        let img = get_test_image3();
        pdb.add_images_transaction()
            .add_image(&img, (4269, 1337))
            .execute()?;
        pdb.update_images_transaction()
            .for_image(&img)
            .update_sauce(&Sauce::NotExisting)
            .execute()?;
        Ok(())
    }

    fn get_test_image() -> ImageHandle {
        ImageHandle::new(String::from("1b64e362cdf968d9-c1fc07e23e05e2fc0be39ce8cc88f8044fcf.jpg")).unwrap()
    }

    fn get_test_image2() -> ImageHandle {
        ImageHandle::new(String::from("c3811874f801fd63-03f07d07b03b05f3370670df0db0ff037037.jpg")).unwrap()
    }

    fn get_test_image3() -> ImageHandle {
        ImageHandle::new(String::from("000011152151fec3-03f07d07b03b05f3370670df0db0ff000000.jpg")).unwrap()
    }
}