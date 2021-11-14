use rusqlite::{Connection, OpenFlags};
use crate::common::error::Error;
use crate::db::sqlite_statements;

pub fn open(db_path: &str) -> Result<Connection, Error> {
    let conn = match Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_WRITE) {
        Ok(conn) => conn,
        Err(_) => {
            eprintln!("Database {} does not exist, creating new...", db_path);
            let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE)?;
            conn.execute(sqlite_statements::PRAGMA_FOREIGN_KEY_ENFORCE, [])?;
            create_tables(&conn)?;
            conn
        }
    };
    Ok(conn)
}

fn create_tables(conn: &Connection) -> Result<(), Error> {
    conn.execute(sqlite_statements::CREATE_FILES_TABLE, [])?;
    conn.execute(sqlite_statements::CREATE_TAGS_TABLE, [])?;
    conn.execute(sqlite_statements::CREATE_FILE_TAGS_TABLE, [])?;
    Ok(())
}