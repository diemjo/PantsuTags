pub use rusqlite::{Connection};
use rusqlite::{params_from_iter, Transaction};
use crate::common::error::Error;
use crate::common::PantsuTag;

pub struct PantsuDB {
    conn: Connection
}

impl PantsuDB {
    pub fn new(db_path: &str) -> Result<PantsuDB, Error> {
        let conn = Connection::open(db_path)?;
        PantsuDB::create_tables(&conn)?;
        Ok(PantsuDB { conn })
    }

    // file
    pub fn add_file(&mut self, filename: &str) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        PantsuDB::add_filename_to_filename_list(&transaction, filename)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn remove_file(&mut self, filename: &str) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        PantsuDB::remove_all_tags_from_filename(&transaction, filename)?;
        PantsuDB::remove_filename_from_filename_list(&transaction, filename)?;
        PantsuDB::remove_unused_tags(&transaction)?;

        transaction.commit()?;
        Ok(())
    }

    pub fn get_files(&self) -> Result<Vec<String>, Error> {
        let mut stmt = self.conn.prepare("SELECT filename FROM files")?;
        let mut rows = stmt.query([]).unwrap();

        let mut files: Vec<String> = Vec::new();
        while let Some(row) = rows.next()? {
            files.push(row.get(0)?);
        }
        Ok(files)

        /*let rows : Vec<Result<String, rusqlite::Error>>  = stmt.query_map([], |row| {
            Ok(row.get::<usize, String>(0).unwrap())
        }).unwrap().collect();
        let rows : Result<Vec<String>, rusqlite::Error> = rows.into_iter().collect();
        Ok(rows?)*/
    }

    pub fn get_files_with_tags(&self, tags: &Vec<PantsuTag>) -> Result<Vec<String>, Error> {
        if tags.len()==0 {
            return self.get_files();
        }

        let mut stmt = self.conn.prepare(
            &format!("{}{}{}",
                     "SELECT filename FROM file_tags
                      WHERE tag IN (", repeat_vars(tags.len()), ")"
            )
        )?;
        let mut rows = stmt.query(
            params_from_iter(tags.iter().map(|t|&t.tag_name).into_iter())
        ).unwrap();

        let mut files: Vec<String> = Vec::new();
        while let Some(row) = rows.next()? {
            files.push(row.get(0)?);
        }
        Ok(files)
    }

    // file->tag
    pub fn add_tags(&mut self, filename: &str, tags: &Vec<PantsuTag>) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        PantsuDB::add_filename_to_filename_list(&transaction, filename)?;
        PantsuDB::add_tags_to_tag_list(&transaction, tags)?;
        PantsuDB::add_tags_to_filename(&transaction, filename, tags)?;

        transaction.commit()?;
        Ok(())
    }

    // file->tag
    pub fn remove_tags(&mut self, filename: &str, tags: &Vec<PantsuTag>) -> Result<(), Error> {
        let transaction = self.conn.transaction()?;

        PantsuDB::remove_tags_from_filename(&transaction, filename, tags)?;
        PantsuDB::remove_unused_tags(&transaction)?;

        transaction.commit()?;
        Ok(())
    }
}

impl PantsuDB {
    fn create_tables(conn: &Connection) -> Result<(), Error> {
        conn.execute("PRAGMA foreign_keys=ON", [])?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
            filename TEXT PRIMARY KEY
        )",
            []
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
            tag TEXT PRIMARY KEY,
            tag_type TEXT NOT NULL
        )",
            []
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS file_tags (
            filename TEXT NOT NULL,
            tag TEXT NOT NULL,
            PRIMARY KEY(filename, tag),
            FOREIGN KEY(filename) REFERENCES files(filename),
            FOREIGN KEY(tag) REFERENCES tags(tag)
        )",
            []
        )?;
        Ok(())
    }

    fn add_tags_to_tag_list(transaction: &Transaction, tags: &Vec<PantsuTag>) -> Result<(), Error> {
        let mut add_tag_list_stmt = transaction.prepare(
            "INSERT OR IGNORE INTO tags (tag, tag_type) VALUES (?, ?)"
        )?;
        for tag in tags {
            add_tag_list_stmt.execute([&tag.tag_name, &tag.tag_type.to_string()])?;
        }
        Ok(())
    }

    fn remove_unused_tags(transaction: &Transaction) -> Result<(), Error> {
        transaction.execute(
            "DELETE FROM tags
            WHERE tag IN
                   (SELECT tags.tag FROM tags LEFT JOIN file_tags ON tags.tag=file_tags.tag WHERE file_tags.tag IS NULL)",
            []
        )?;
        Ok(())
    }

    fn add_filename_to_filename_list(transaction: &Transaction, filename: &str) -> Result<(), Error> {
        let mut add_file_list_stmt = transaction.prepare(
            "INSERT OR IGNORE INTO files (filename) VALUES (?)"
        )?;
        add_file_list_stmt.execute([filename])?;
        Ok(())
    }

    fn remove_filename_from_filename_list(transaction: &Transaction, filename: &str) -> Result<(), Error> {
        let mut remove_file_list_stmt = transaction.prepare(
            "DELETE FROM files WHERE filename=(?)"
        )?;
        remove_file_list_stmt.execute([filename])?;
        Ok(())
    }

    fn add_tags_to_filename(transaction: &Transaction, filename: &str, tags: &Vec<PantsuTag>) -> Result<(), Error> {
        let mut add_tag_stmt = transaction.prepare(
            "INSERT OR IGNORE INTO file_tags (filename, tag) VALUES (?, ?)"
        )?;
        for tag in tags {
            add_tag_stmt.execute([filename, &tag.tag_name])?;
        }
        Ok(())
    }

    fn remove_tags_from_filename(transaciton: &Transaction, filename: &str, tags: &Vec<PantsuTag>) -> Result<(), Error> {
        let mut remove_tag_stmt = transaciton.prepare(
            "DELETE FROM file_tags WHERE filename=(?) AND tag=(?)"
        )?;
        for tag in tags {
            remove_tag_stmt.execute([filename, &tag.tag_name])?;
        }
        Ok(())
    }

    fn remove_all_tags_from_filename(transaciton: &Transaction, filename: &str) -> Result<(), Error> {
        let mut remove_tag_stmt = transaciton.prepare(
            "DELETE FROM file_tags WHERE filename=(?)"
        )?;
        remove_tag_stmt.execute([filename])?;
        Ok(())
    }
}

fn repeat_vars(count: usize) -> String {
    assert_ne!(count, 0);
    let mut s = "?,".repeat(count);
    // Remove trailing comma
    s.pop();
    s
}