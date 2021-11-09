pub use rusqlite::{Connection, Result};
use rusqlite::Transaction;
use crate::common::{PantsuTag, PantsuTagType};

pub struct PantsuDB {
    conn: Connection
}

impl PantsuDB {
    pub fn add_tags(&mut self, filename: &String, tags: &Vec<PantsuTag>) -> Result<()> {
        let transaction = self.conn.transaction()?;

        add_tags_to_tag_list(&transaction, tags)?;

        add_filename_to_filename_list(&transaction, filename)?;

        add_tags_to_filename(&transaction, filename, tags)?;

        transaction.commit()
    }
}

fn add_tags_to_tag_list(transaction: &Transaction, tags: &Vec<PantsuTag>) -> Result<()> {
    let mut tag_list_stmt = transaction.prepare("INSERT OR IGNORE INTO tags (tag, tag_type) VALUES (?, ?)")?;
    for tag in tags {
        match tag.tag_type {
            PantsuTagType::Artist => {
                tag_list_stmt.execute([&tag.tag_name, &tag.tag_type.to_string()])?;
            },
            PantsuTagType::Source => {
                tag_list_stmt.execute([&tag.tag_name, &tag.tag_type.to_string()])?;
            },
            PantsuTagType::Character => {
                tag_list_stmt.execute([&tag.tag_name, &tag.tag_type.to_string()])?;
            },
            PantsuTagType::Generic => {
                tag_list_stmt.execute([&tag.tag_name, &tag.tag_type.to_string()])?;
            }
        }
    }
    Ok(())
}

fn add_filename_to_filename_list(transaction: &Transaction, filename: &String) -> Result<()> {
    let mut file_list_stmt = transaction.prepare("INSERT OR IGNORE INTO files (filename) VALUES (?)")?;
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

pub fn init_db() -> Result<PantsuDB> {
    let conn = open_db()?;
    create_tables(&conn)?;
    Ok(PantsuDB { conn })
}

fn open_db() -> Result<Connection> {
    Connection::open("pantsu_tags.db")
}

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