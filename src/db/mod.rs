pub use rusqlite::{Connection, Result};
use rusqlite::Transaction;
use crate::common::PantsuTag;

pub struct PantsuDB {
    conn: Connection
}

impl PantsuDB {
    pub fn new(db_path: &str) -> Result<PantsuDB> {
        let conn = Connection::open(db_path)?;
        PantsuDB::create_tables(&conn)?;
        Ok(PantsuDB { conn })
    }

    pub fn add_tags(&mut self, filename: &String, tags: &Vec<PantsuTag>) -> Result<()> {
        let transaction = self.conn.transaction()?;

        PantsuDB::add_tags_to_tag_list(&transaction, tags)?;

        PantsuDB::add_filename_to_filename_list(&transaction, filename)?;

        PantsuDB::add_tags_to_filename(&transaction, filename, tags)?;

        transaction.commit()
    }
}

impl PantsuDB {
    fn create_tables(conn: &Connection) -> Result<()> {
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

    fn add_tags_to_tag_list(transaction: &Transaction, tags: &Vec<PantsuTag>) -> Result<()> {
        let mut tag_list_stmt = transaction.prepare("INSERT OR IGNORE INTO tags (tag, tag_type) VALUES (?, ?)")?;
        for tag in tags {
            tag_list_stmt.execute([&tag.tag_name, &tag.tag_type.to_string()])?;
        }
        Ok(())
    }

    fn add_filename_to_filename_list(transaction: &Transaction, filename: &String) -> Result<()> {
        let mut file_list_stmt = transaction.prepare("INSERT OR IGNORE INTO files (filename) VALUES (?)")?;
        file_list_stmt.execute([filename])?;
        Ok(())
    }

    fn remove_filename_from_filename_list(transaction: &Transaction, filename: &String) -> Result<()> {
        let mut file_list_stmt = transaction.prepare("DELETE FROM files WHERE filename=(?)")?;
        file_list_stmt.execute([filename])?;
        Ok(())
    }

    fn add_tags_to_filename(transaction: &Transaction, filename: &String, tags: &Vec<PantsuTag>) -> Result<()> {
        let mut tag_stmt = transaction.prepare("INSERT OR IGNORE INTO file_tags (filename, tag) VALUES (?, ?)")?;
        for tag in tags {
            tag_stmt.execute([filename, &tag.tag_name])?;
        }
        Ok(())
    }

    fn remove_tags_from_tag_list(transaciton: &Transaction, filename: &String, tags: &Vec<PantsuTag>) -> Result<()> {
        let mut tag_remove_stmt = transaciton.prepare("DELETE FROM file_tags WHERE filename=(?) AND tag=(?)")?;
        for tag in tags {
            tag_remove_stmt.execute([filename, &tag.tag_name])?;
        }
        Ok(())
    }
}
