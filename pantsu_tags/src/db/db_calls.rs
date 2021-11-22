use rusqlite::{Connection, ffi, params, Transaction};
use crate::common::error::Error;
use crate::common::error::Error::{SQLError, SQLPrimaryKeyError};
use crate::common::image_file::ImageFile;
use crate::common::pantsu_tag::{PantsuTag, PantsuTagType};
use crate::db::sqlite_statements;

// INSERT
pub fn add_tags_to_tag_list(transaction: &Transaction, tags: &Vec<PantsuTag>) -> Result<(), Error> {
    let mut add_tag_list_stmt = transaction.prepare(sqlite_statements::INSERT_TAG_INTO_TAG_LIST)?;
    for tag in tags {
        add_tag_list_stmt.execute([&tag.tag_name, &tag.tag_type.to_string()])?;
    }
    Ok(())
}

pub fn add_file_to_file_list(transaction: &Transaction, file: &ImageFile) -> Result<(), Error> {
    let mut add_file_list_stmt = transaction.prepare(sqlite_statements::INSERT_FILE_INTO_FILE_LIST)?;
    let res = add_file_list_stmt.execute(params![&file.filename, &file.file_source]);
    // check for primary key constraint
    return if let Err(rusqlite::Error::SqliteFailure(ffi::Error { code: _, extended_code: 1555 }, ..)) = res {
        Err(SQLPrimaryKeyError(res.unwrap_err()))
    } else if let Err(e) = res {
        Err(SQLError(e))
    } else {
        Ok(())
    }
}

pub fn add_tags_to_file_tags(transaction: &Transaction, file: &ImageFile, tags: &Vec<PantsuTag>) -> Result<(), Error> {
    let mut add_tag_stmt = transaction.prepare(sqlite_statements::INSERT_TAG_FOR_FILE)?;
    for tag in tags {
        add_tag_stmt.execute([&file.filename, &tag.tag_name])?;
    }
    Ok(())
}

// UPDATE
pub fn update_file_source(transaction: &Transaction, file: &ImageFile) -> Result<(), Error> {
    let mut update_file_statement = transaction.prepare(sqlite_statements::UPDATE_FILE)?;
    update_file_statement.execute(params![&file.file_source, &file.filename])?;
    Ok(())
}

// DELETE
pub fn remove_unused_tags(transaction: &Transaction) -> Result<(), Error> {
    transaction.execute(sqlite_statements::DELETE_UNUSED_TAGS, [])?;
    Ok(())
}

pub fn remove_file_from_file_list(transaction: &Transaction, file: &ImageFile) -> Result<(), Error> {
    let mut remove_file_list_stmt = transaction.prepare(sqlite_statements::DELETE_FILE_FROM_FILE_LIST)?;
    remove_file_list_stmt.execute([&file.filename])?;
    Ok(())
}

pub fn remove_tags_from_file(transaction: &Transaction, file: &ImageFile, tags: &Vec<PantsuTag>) -> Result<(), Error> {
    let mut remove_tag_stmt = transaction.prepare(sqlite_statements::DELETE_TAG_FROM_FILE)?;
    for tag in tags {
        remove_tag_stmt.execute([&file.filename, &tag.tag_name])?;
    }
    Ok(())
}

pub fn remove_all_tags_from_file(transaction: &Transaction, file: &ImageFile) -> Result<(), Error> {
    let mut remove_tag_stmt = transaction.prepare(sqlite_statements::DELETE_ALL_TAGS_FROM_FILE)?;
    remove_tag_stmt.execute([&file.filename])?;
    Ok(())
}

pub fn clear_all_file_tags(transaction: &Transaction) -> Result<(), Error> {
    transaction.execute(sqlite_statements::CLEAR_FILE_TAGS, [])?;
    Ok(())
}

pub fn clear_all_files(transaction: &Transaction) -> Result<(), Error> {
    transaction.execute(sqlite_statements::CLEAR_FILE_LIST, [])?;
    Ok(())
}

pub fn clear_all_tags(transaction: &Transaction) -> Result<(), Error> {
    transaction.execute(sqlite_statements::CLEAR_TAG_LIST, [])?;
    Ok(())
}

// SELECT
pub fn get_file(connection: &Connection, filename: &str) -> Result<Option<ImageFile>, Error> {
    let mut stmt = connection.prepare(sqlite_statements::SELECT_FILE)?;
    query_helpers::query_row_as_file(&mut stmt, [filename])
}

pub fn get_all_files(connection: &Connection) -> Result<Vec<ImageFile>, Error> {
    let mut stmt = connection.prepare(sqlite_statements::SELECT_ALL_FILES)?;
    query_helpers::query_rows_as_files(&mut stmt, [])
}

pub fn get_files_with_tags(connection: &Connection, included_tags: &Vec<String>, excluded_tags: &Vec<String>) -> Result<Vec<ImageFile>, Error> {
    let formatted_stmt =
        if included_tags.len()!=0 && excluded_tags.len()!=0 {
            sqlite_statements::SELECT_FILES_FOR_INCLUDING_AND_EXCLUDING_TAGS
                .replace(sqlite_statements::SELECT_FILES_FOR_INCLUDING_TAGS_PLACEHOLDER, &query_helpers::repeat_vars(included_tags.len()))
                .replace(sqlite_statements::SELECT_FILES_FOR_EXCLUDING_TAGS_PLACEHOLDER, &query_helpers::repeat_vars(excluded_tags.len()))
                .replace(sqlite_statements::SELECT_FILES_FOR_TAGS_TAG_COUNT, &included_tags.len().to_string())
        } else if included_tags.len()==0 {
            sqlite_statements::SELECT_FILES_FOR_EXCLUDING_TAGS
                .replace(sqlite_statements::SELECT_FILES_FOR_EXCLUDING_TAGS_PLACEHOLDER, &query_helpers::repeat_vars(excluded_tags.len()))
        } else {
            sqlite_statements::SELECT_FILES_FOR_INCLUDING_TAGS
                .replace(sqlite_statements::SELECT_FILES_FOR_INCLUDING_TAGS_PLACEHOLDER, &query_helpers::repeat_vars(included_tags.len()))
                .replace(sqlite_statements::SELECT_FILES_FOR_TAGS_TAG_COUNT, &included_tags.len().to_string())
        };
    let mut stmt = connection.prepare(&formatted_stmt)?;

    if included_tags.len()!=0 && excluded_tags.len()!=0 {
        let mut vec = Vec::<&String>::new();
        vec.extend(included_tags);
        vec.extend(excluded_tags);
        let params = rusqlite::params_from_iter(vec.iter());
        query_helpers::query_rows_as_files(&mut stmt, params)
    } else if included_tags.len()==0 {
        let params = rusqlite::params_from_iter(excluded_tags.iter());
        query_helpers::query_rows_as_files(&mut stmt, params)
    } else {
        let params = rusqlite::params_from_iter(included_tags.iter());
        query_helpers::query_rows_as_files(&mut stmt, params)
    }
}

pub fn get_tags_for_file(connection: &Connection, file: &ImageFile) -> Result<Vec<PantsuTag>, Error> {
    let mut stmt = connection.prepare(sqlite_statements::SELECT_TAGS_FOR_FILE)?;
    query_helpers::query_rows_as_tags(&mut stmt, [&file.filename])
}

pub fn get_all_tags(connection: &Connection) -> Result<Vec<PantsuTag>, Error> {
    let mut stmt = connection.prepare(sqlite_statements::SELECT_ALL_TAGS)?;
    query_helpers::query_rows_as_tags(&mut stmt, [])
}

pub fn get_tags_with_types(connection: &Connection, types: &Vec<PantsuTagType>) -> Result<Vec<PantsuTag>, Error> {
    let formatted_stmt = sqlite_statements::SELECT_TAGS_WITH_TYPE
        .replace(sqlite_statements::SELECT_TAGS_WITH_TYPE_PLACEHOLDER, &query_helpers::repeat_vars(types.len()));
    let mut stmt = connection.prepare(&formatted_stmt)?;

    let params = rusqlite::params_from_iter(types.iter().map(|t| t.to_string()).into_iter());
    query_helpers::query_rows_as_tags(&mut stmt, params)
}



mod query_helpers {
    use rusqlite::{Params, Statement};
    use crate::common::error::Error;
    use crate::common::image_file::ImageFile;
    use crate::common::pantsu_tag::PantsuTag;

    pub fn query_row_as_file<P: Params>(stmt: &mut Statement, params: P) -> Result<Option<ImageFile>, Error> {
        let file = stmt.query_row(params, |row| {
            Ok(
                ImageFile {
                    filename: row.get(0).unwrap(),
                    file_source: row.get(1).unwrap()
                }
            )
        });
         match file {
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Ok(file) => Ok(Some(file)),
            Err(e) => Err(e)?
        }
    }

    pub fn query_rows_as_files<P: Params>(stmt: &mut Statement, params: P) -> Result<Vec<ImageFile>, Error> {
        let rows: Vec<Result<ImageFile, rusqlite::Error>> = stmt.query_map(params, |row| {
            Ok(
                ImageFile {
                    filename: row.get(0).unwrap(),
                    file_source: row.get(1).unwrap()
                }
            )
        }).unwrap().collect();
        let rows: Result<Vec<ImageFile>, rusqlite::Error> = rows.into_iter().collect();
        Ok(rows?)

        /*let mut rows = stmt.query([]).unwrap();
    let mut files: Vec<String> = Vec::new();
    while let Some(row) = rows.next()? {
        files.push(row.get(0)?);
    }
    Ok(files)*/
    }

    pub fn query_rows_as_tags<P: Params>(stmt: &mut Statement, params: P) -> Result<Vec<PantsuTag>, Error> {
        let rows: Vec<Result<PantsuTag, rusqlite::Error>> = stmt.query_map(params, |row| {
            Ok(PantsuTag {
                tag_name: row.get(0).unwrap(),
                tag_type: row.get::<usize, String>(1).unwrap().parse().unwrap()
            })
        }).unwrap().collect();
        let rows: Result<Vec<PantsuTag>, rusqlite::Error> = rows.into_iter().collect();
        Ok(rows?)
    }

    pub fn repeat_vars(count: usize) -> String {
        assert_ne!(count, 0);
        let mut s = "?,".repeat(count);
        // Remove trailing comma
        s.pop();
        s
    }
}