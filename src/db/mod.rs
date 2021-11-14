pub use rusqlite::{Connection};
use rusqlite::{Params, Statement, Transaction};
use crate::common::error::Error;
use crate::common::{PantsuTag, PantsuTagType};

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

    // file
    pub fn add_file(&mut self, filename: &str) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        PantsuDB::add_file_to_file_list(&transaction, filename)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn remove_file(&mut self, filename: &str) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        PantsuDB::remove_all_tags_from_file(&transaction, filename)?;
        PantsuDB::remove_file_from_file_list(&transaction, filename)?;
        PantsuDB::remove_unused_tags(&transaction)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn get_files(&self) -> Result<Vec<String>, Error> {
        let mut stmt = self.conn.prepare(sqlite_statements::SELECT_ALL_FILES)?;
        query_rows_as_files(&mut stmt, [])
    }

    pub fn get_files_with_tags(&self, tags: &Vec<PantsuTag>) -> Result<Vec<String>, Error> {
        if tags.len()==0 {
            return self.get_files();
        }

        let formatted_stmt = sqlite_statements::SELECT_FILES_FOR_TAGS
            .replace(sqlite_statements::SELECT_FILES_FOR_TAGS_PLACEHOLDER, &repeat_vars(tags.len()));
        let mut stmt = self.conn.prepare(&formatted_stmt)?;

        let params = rusqlite::params_from_iter(tags.iter().map(|t|&t.tag_name).into_iter());
        query_rows_as_files(&mut stmt, params)
    }

    // file->tag
    pub fn add_tags(&mut self, filename: &str, tags: &Vec<PantsuTag>) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        PantsuDB::add_file_to_file_list(&transaction, filename)?;
        PantsuDB::add_tags_to_tag_list(&transaction, tags)?;
        PantsuDB::add_tags_to_file(&transaction, filename, tags)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn remove_tags(&mut self, filename: &str, tags: &Vec<PantsuTag>) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        PantsuDB::remove_tags_from_file(&transaction, filename, tags)?;
        PantsuDB::remove_unused_tags(&transaction)?;

        transaction.commit()?;
        Ok(())
    }

    // tags
    pub fn get_all_tags(&self) -> Result<Vec<PantsuTag>, Error> {
        let mut stmt = self.conn.prepare(sqlite_statements::SELECT_ALL_TAGS)?;
        query_rows_as_tags(&mut stmt, [])
    }

    pub fn get_tags_with_types(&self, types: &Vec<PantsuTagType>) -> Result<Vec<PantsuTag>, Error> {
        let formatted_stmt = sqlite_statements::SELECT_TAGS_WITH_TYPE
            .replace(sqlite_statements::SELECT_TAGS_WITH_TYPE_PLACEHOLDER, &repeat_vars(types.len()));
        let mut stmt = self.conn.prepare(&formatted_stmt)?;

        let params = rusqlite::params_from_iter(types.iter().map(|t|t.to_string()).into_iter());
        query_rows_as_tags(&mut stmt, params)
    }
}

impl PantsuDB {

    fn add_tags_to_tag_list(transaction: &Transaction, tags: &Vec<PantsuTag>) -> Result<(), Error> {
        let mut add_tag_list_stmt = transaction.prepare(sqlite_statements::INSERT_TAG_INTO_TAG_LIST)?;
        for tag in tags {
            add_tag_list_stmt.execute([&tag.tag_name, &tag.tag_type.to_string()])?;
        }
        Ok(())
    }

    fn remove_unused_tags(transaction: &Transaction) -> Result<(), Error> {
        transaction.execute(sqlite_statements::DELETE_UNUSED_TAGS, [])?;
        Ok(())
    }

    fn add_file_to_file_list(transaction: &Transaction, filename: &str) -> Result<(), Error> {
        let mut add_file_list_stmt = transaction.prepare(sqlite_statements::INSERT_FILE_INTO_FILE_LIST)?;
        add_file_list_stmt.execute([filename])?;
        Ok(())
    }

    fn remove_file_from_file_list(transaction: &Transaction, filename: &str) -> Result<(), Error> {
        let mut remove_file_list_stmt = transaction.prepare(sqlite_statements::DELETE_FILE_FROM_FILE_LIST)?;
        remove_file_list_stmt.execute([filename])?;
        Ok(())
    }

    fn add_tags_to_file(transaction: &Transaction, filename: &str, tags: &Vec<PantsuTag>) -> Result<(), Error> {
        let mut add_tag_stmt = transaction.prepare(sqlite_statements::INSERT_TAG_FOR_FILE)?;
        for tag in tags {
            add_tag_stmt.execute([filename, &tag.tag_name])?;
        }
        Ok(())
    }

    fn remove_tags_from_file(transaciton: &Transaction, filename: &str, tags: &Vec<PantsuTag>) -> Result<(), Error> {
        let mut remove_tag_stmt = transaciton.prepare(sqlite_statements::DELETE_TAG_FROM_FILE)?;
        for tag in tags {
            remove_tag_stmt.execute([filename, &tag.tag_name])?;
        }
        Ok(())
    }

    fn remove_all_tags_from_file(transaciton: &Transaction, filename: &str) -> Result<(), Error> {
        let mut remove_tag_stmt = transaciton.prepare(sqlite_statements::DELETE_ALL_TAGS_FROM_FILE)?;
        remove_tag_stmt.execute([filename])?;
        Ok(())
    }
}

fn query_rows_as_files<P: Params>(stmt: &mut Statement, params: P) -> Result<Vec<String>, Error> {
    let rows : Vec<Result<String, rusqlite::Error>>  = stmt.query_map(params, |row| {
        Ok(row.get::<usize, String>(0).unwrap())
    }).unwrap().collect();
    let rows : Result<Vec<String>, rusqlite::Error> = rows.into_iter().collect();
    Ok(rows?)

    /*let mut rows = stmt.query([]).unwrap();
    let mut files: Vec<String> = Vec::new();
    while let Some(row) = rows.next()? {
        files.push(row.get(0)?);
    }
    Ok(files)*/
}

fn query_rows_as_tags<P: Params>(stmt: &mut Statement, params: P) -> Result<Vec<PantsuTag>, Error> {
    let rows : Vec<Result<PantsuTag, rusqlite::Error>>  = stmt.query_map(params, |row| {
        Ok(PantsuTag {
            tag_name: row.get(0).unwrap(),
            tag_type: row.get::<usize, String>(1).unwrap().parse().unwrap()
        })
    }).unwrap().collect();
    let rows : Result<Vec<PantsuTag>, rusqlite::Error> = rows.into_iter().collect();
    Ok(rows?)
}

fn repeat_vars(count: usize) -> String {
    assert_ne!(count, 0);
    let mut s = "?,".repeat(count);
    // Remove trailing comma
    s.pop();
    s
}