use std::path::Path;
use rusqlite::{Connection, OpenFlags};
use crate::common::error::{Error};
use log::{debug};
use crate::db::{db_calls, sqlite_statements};

pub fn open(db_path: &Path) -> Result<Connection, Error> {
    let pantsu_db_updates: Vec<&dyn Fn(&mut Connection) -> Result<(), Error>> = vec![
        //eg: &db_update_1_2,
    ];
    let pantsu_db_newest_version = pantsu_db_updates.len() + 1;

    let mut conn = match Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_WRITE) {
        Ok(conn) => conn,
        Err(_) => {
            let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE)?;
            conn.pragma_update(None, "foreign_key", "ON")?;
            conn.pragma_update(None, "user_version", 0)?;
            conn
        }
    };
    let current_db_version = db_calls::db_version(&conn)?;
    if pantsu_db_newest_version < current_db_version {
        return Err(Error::ProgramOutdated(format!("Expected database version <={} but found version {}", pantsu_db_newest_version, current_db_version)));
    } else if pantsu_db_newest_version > current_db_version {
        if current_db_version == 0 {
            db_init_new(&mut conn)?;
            conn.pragma_update(None, "user_version", pantsu_db_newest_version)?;
        } else {
            for i in current_db_version..pantsu_db_newest_version {
                pantsu_db_updates[i-1](&mut conn)?;
                conn.pragma_update(None, "user_version", i + 1)?;
            }
        }
    } else {
        debug!("opened database with version {}", current_db_version);
    }
    Ok(conn)
}

fn db_init_new(connection: &mut Connection) -> Result<(), Error> {
    debug!("Initializing database");
    connection.execute_batch(sqlite_statements::DB_INIT_TABLES)?;
    Ok(())
}