use rusqlite::{Connection, ffi, params, Transaction};
use crate::common::error::Result;
use crate::common::image_handle::ImageHandle;
use crate::common::pantsu_tag::{PantsuTag, PantsuTagType};
use crate::db::{SauceType, sqlite_statements};
use crate::{Error, Sauce};

// INSERT
pub(crate) fn add_tags_to_tag_list(transaction: &Transaction, tags: &Vec<&PantsuTag>) -> Result<()> {
    let mut add_tag_list_stmt = transaction.prepare(sqlite_statements::INSERT_TAG_INTO_TAG_LIST)?;
    for &tag in tags {
        add_tag_list_stmt.execute([&tag.tag_name, &tag.tag_type.to_string()])?;
    }
    Ok(())
}

pub(crate) fn add_file_to_file_list(transaction: &Transaction, file: &ImageHandle) -> Result<()> {
    let mut add_file_list_stmt = transaction.prepare(sqlite_statements::INSERT_FILE_INTO_FILE_LIST)?;
    let res = add_file_list_stmt.execute(params![file.get_filename(), file.get_sauce().get_type(), file.get_sauce().get_value(), file.get_res().0, file.get_res().1]);
    // check for primary key constraint
    return if let Err(rusqlite::Error::SqliteFailure(ffi::Error { code: _, extended_code: 1555 }, ..)) = res {
        Err(Error::SQLPrimaryKeyError(res.unwrap_err()))
    } else if let Err(e) = res {
        Err(Error::SQLError(e))
    } else {
        Ok(())
    }
}

pub(crate) fn add_tags_to_file(transaction: &Transaction, filename: &str, tags: &Vec<&PantsuTag>) -> Result<()> {
    let mut add_tag_stmt = transaction.prepare(sqlite_statements::INSERT_TAG_FOR_FILE)?;
    for &tag in tags {
        add_tag_stmt.execute([filename, tag.tag_name.as_str()])?;
    }
    Ok(())
}

// UPDATE
pub(crate) fn update_file_source(transaction: &Transaction, filename: &str, sauce: &Sauce) -> Result<()> {
    let mut update_file_statement = transaction.prepare(sqlite_statements::UPDATE_IMAGE_SOURCE)?;
    update_file_statement.execute(params![sauce.get_type(), sauce.get_value(), filename])?;
    Ok(())
}

// DELETE
pub(crate) fn remove_unused_tags(transaction: &Transaction) -> Result<()> {
    transaction.execute(sqlite_statements::DELETE_UNUSED_TAGS, [])?;
    Ok(())
}

pub(crate) fn remove_file_from_file_list(transaction: &Transaction, filename: &str) -> Result<()> {
    let mut remove_file_list_stmt = transaction.prepare(sqlite_statements::DELETE_FILE_FROM_FILE_LIST)?;
    remove_file_list_stmt.execute([filename])?;
    Ok(())
}

pub(crate) fn remove_tags_from_file(transaction: &Transaction, filename: &str, tags: &Vec<&str>) -> Result<()> {
    let mut remove_tag_stmt = transaction.prepare(sqlite_statements::DELETE_TAG_FROM_FILE)?;
    for tag in tags {
        remove_tag_stmt.execute([filename, tag])?;
    }
    Ok(())
}

pub(crate) fn remove_all_tags_from_file(transaction: &Transaction, filename: &str) -> Result<()> {
    let mut remove_tags_stmt = transaction.prepare(sqlite_statements::DELETE_ALL_TAGS_FROM_FILE)?;
    remove_tags_stmt.execute([filename])?;
    Ok(())
}

pub(crate) fn clear_all_file_tags(transaction: &Transaction) -> Result<()> {
    transaction.execute(sqlite_statements::CLEAR_FILE_TAGS, [])?;
    Ok(())
}

pub(crate) fn clear_all_files(transaction: &Transaction) -> Result<()> {
    transaction.execute(sqlite_statements::CLEAR_FILE_LIST, [])?;
    Ok(())
}

pub(crate) fn clear_all_tags(transaction: &Transaction) -> Result<()> {
    transaction.execute(sqlite_statements::CLEAR_TAG_LIST, [])?;
    Ok(())
}

// SELECT
pub(crate) fn get_file(connection: &Connection, filename: &str) -> Result<Option<ImageHandle>> {
    let mut stmt = connection.prepare(sqlite_statements::SELECT_FILE)?;
    query_helpers::query_row_as_file(&mut stmt, [filename])
}
/*
pub fn get_all_files(connection: &Connection) -> Result<Vec<ImageHandle>> {
    let mut stmt = connection.prepare(sqlite_statements::SELECT_ALL_FILES)?;
    query_helpers::query_rows_as_files(&mut stmt, [])
}
*/
pub(crate) fn get_files(connection: &Connection, included_tags: &Vec<&str>, excluded_tags: &Vec<&str>, sauce_type: SauceType) -> Result<Vec<ImageHandle>> {
    let formatted_stmt =
        if included_tags.len()!=0 && excluded_tags.len()!=0 {
            sqlite_statements::SELECT_FILES_FOR_INCLUDING_AND_EXCLUDING_TAGS
                .replace(sqlite_statements::SELECT_FILES_FOR_INCLUDING_TAGS_PLACEHOLDER, &query_helpers::repeat_vars(included_tags.len()))
                .replace(sqlite_statements::SELECT_FILES_FOR_EXCLUDING_TAGS_PLACEHOLDER, &query_helpers::repeat_vars(excluded_tags.len()))
                .replace(sqlite_statements::SELECT_FILES_FOR_TAGS_TAG_COUNT, &included_tags.len().to_string())
        } else if excluded_tags.len()!=0 {
            sqlite_statements::SELECT_FILES_FOR_EXCLUDING_TAGS
                .replace(sqlite_statements::SELECT_FILES_FOR_EXCLUDING_TAGS_PLACEHOLDER, &query_helpers::repeat_vars(excluded_tags.len()))
        } else if included_tags.len()!=0 {
            sqlite_statements::SELECT_FILES_FOR_INCLUDING_TAGS
                .replace(sqlite_statements::SELECT_FILES_FOR_INCLUDING_TAGS_PLACEHOLDER, &query_helpers::repeat_vars(included_tags.len()))
                .replace(sqlite_statements::SELECT_FILES_FOR_TAGS_TAG_COUNT, &included_tags.len().to_string())
        } else {
            sqlite_statements::SELECT_ALL_FILES.to_string()
        };
    let formatted_stmt = formatted_stmt.replace(sqlite_statements::SAUCE_TYPE_PLACEHOLDER, match sauce_type {
        SauceType::Existing => crate::common::image_handle::EXISTING_FLAG,
        SauceType::NotExisting => crate::common::image_handle::NOT_EXISTING_FLAG,
        SauceType::NotChecked => crate::common::image_handle::NOT_CHECKED_FLAG,
        SauceType::Any => "%",
    });
    let mut stmt = connection.prepare(&formatted_stmt)?;

    if included_tags.len()!=0 && excluded_tags.len()!=0 {
        let mut vec = Vec::<&str>::new();
        vec.extend(included_tags);
        vec.extend(excluded_tags);
        let params = rusqlite::params_from_iter(vec.iter());
        query_helpers::query_rows_as_files(&mut stmt, params)
    } else if excluded_tags.len()!=0 {
        let params = rusqlite::params_from_iter(excluded_tags.iter());
        query_helpers::query_rows_as_files(&mut stmt, params)
    } else if included_tags.len()!=0 {
        let params = rusqlite::params_from_iter(included_tags.iter());
        query_helpers::query_rows_as_files(&mut stmt, params)
    } else {
        query_helpers::query_rows_as_files(&mut stmt, [])
    }
}

pub(crate) fn get_tags_for_file(connection: &Connection, filename: &str) -> Result<Vec<PantsuTag>> {
    let mut stmt = connection.prepare(sqlite_statements::SELECT_TAGS_FOR_FILE)?;
    query_helpers::query_rows_as_tags(&mut stmt, [filename])
}

pub(crate) fn get_all_tags(connection: &Connection) -> Result<Vec<PantsuTag>> {
    let mut stmt = connection.prepare(sqlite_statements::SELECT_ALL_TAGS)?;
    query_helpers::query_rows_as_tags(&mut stmt, [])
}

pub(crate) fn get_tags_with_types(connection: &Connection, types: &Vec<PantsuTagType>) -> Result<Vec<PantsuTag>> {
    let formatted_stmt = sqlite_statements::SELECT_TAGS_WITH_TYPE
        .replace(sqlite_statements::SELECT_TAGS_WITH_TYPE_PLACEHOLDER, &query_helpers::repeat_vars(types.len()));
    let mut stmt = connection.prepare(&formatted_stmt)?;

    let params = rusqlite::params_from_iter(types.iter().map(|t| t.to_string()).into_iter());
    query_helpers::query_rows_as_tags(&mut stmt, params)
}

pub(crate) fn get_tags_for_file_with_types(connection: &Connection, filename: &str, types: &Vec<PantsuTagType>) -> Result<Vec<PantsuTag>> {
    let formatted_stmt = sqlite_statements::SELECT_TAGS_FOR_FILE_WITH_TYPE
        .replace(sqlite_statements::SELECT_TAGS_WITH_TYPE_PLACEHOLDER, &query_helpers::repeat_vars(types.len()));
    let mut stmt = connection.prepare(&formatted_stmt)?;

    let mut vec = vec![filename.to_string()];
    vec.extend(types.iter().map(|t| t.to_string()).into_iter());
    let params = rusqlite::params_from_iter(vec);
    query_helpers::query_rows_as_tags(&mut stmt, params)
}

mod query_helpers {
    use std::str::FromStr;

    use rusqlite::{Params, Row, Statement};
    use rusqlite::types::Type;
    use crate::common::error::Result;
    use crate::common::image_handle::{EXISTING_FLAG, NOT_EXISTING_FLAG, NOT_CHECKED_FLAG, ImageHandle};
    use crate::common::pantsu_tag::PantsuTag;
    use crate::{Error, Sauce, PantsuTagType};

    pub fn query_row_as_file<P: Params>(stmt: &mut Statement, params: P) -> Result<Option<ImageHandle>> {
        let file = stmt.query_row(params, image_handle_from_row);
         match file {
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Ok(file) => Ok(Some(file)),
            Err(e) => Err(e)?
        }
    }

    pub fn query_rows_as_files<P: Params>(stmt: &mut Statement, params: P) -> Result<Vec<ImageHandle>> {
        let rows: Vec<std::result::Result<ImageHandle, rusqlite::Error>> = stmt.query_map(params, image_handle_from_row)?.collect();
        let rows: std::result::Result<Vec<ImageHandle>, rusqlite::Error> = rows.into_iter().collect();
        Ok(rows?)

        /*let mut rows = stmt.query([]).unwrap();
    let mut files: Vec<String> = Vec::new();
    while let Some(row) = rows.next()? {
        files.push(row.get(0)?);
    }
    Ok(files)*/
    }

    fn image_handle_from_row(row: &Row) -> rusqlite::Result<ImageHandle> {
        Ok(
            ImageHandle::new(
                row.get(0)?,
                match row.get::<usize, String>(1)?.as_str() {
                    EXISTING_FLAG => Sauce::Match(row.get(2)?),
                    NOT_EXISTING_FLAG => Sauce::NotExisting,
                    NOT_CHECKED_FLAG => Sauce::NotChecked,
                    s => return Err(rusqlite::Error::FromSqlConversionFailure(0, Type::Text, Box::new(Error::InvalidTagType(s.to_string()))))
                },
                (row.get(3)?, row.get(4)?)
            )
        )
    }

    pub fn query_rows_as_tags<P: Params>(stmt: &mut Statement, params: P) -> Result<Vec<PantsuTag>> {
        let rows: Vec<PantsuTag> = stmt.query(params)?
            .mapped(|row| -> rusqlite::Result<(String, String)> {
                Ok((row.get(0)?, row.get::<usize, String>(1)?))
            })
            .map(|r| {
                match r {
                    Ok((n, t)) => Ok(PantsuTag { tag_type: PantsuTagType::from_str(&t)? , tag_name: n }),
                    Err(e) => Err(Error::SQLError(e))
                }
            })
            .collect::<Result<Vec<PantsuTag>>>()?;
        Ok(rows)
    }

    pub fn repeat_vars(count: usize) -> String {
        assert_ne!(count, 0);
        let mut s = "?,".repeat(count);
        // Remove trailing comma
        s.pop();
        s
    }
}