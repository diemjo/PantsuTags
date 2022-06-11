use std::fmt::Debug;
use rusqlite::ToSql;
use rusqlite::types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef};
use crate::LIB_PATH;
use crate::Sauce::{NotExisting, NotChecked, Match};


#[derive(Debug, PartialEq, Clone)]
pub struct ImageHandle {
    filename: String,
    file_source: Sauce,
    file_res: (u32, u32),
}

impl ImageHandle {
    pub(crate) fn new(filename: String, file_source: Sauce, file_res: (u32, u32)) -> ImageHandle {
        ImageHandle {
            filename,
            file_source,
            file_res
        }
    }

    pub fn get_filename(&self) -> &str {
        self.filename.as_str()
    }

    pub fn get_path(&self) -> String { format!("{}{}", LIB_PATH, self.filename.as_str()) }

    pub fn get_sauce(&self) -> &Sauce {
        &self.file_source
    }

    pub fn get_res(&self) -> (u32, u32) {
        self.file_res
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Sauce {
    Match(String),
    NotExisting,
    NotChecked
}

impl ToSql for Sauce {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let sql = match self {
            Match(sauce) => ToSqlOutput::from(sauce.as_str()),
            NotExisting => ToSqlOutput::from(NOT_EXISTING_FLAG),
            NotChecked => ToSqlOutput::from(NOT_CHECKED_FLAG)
        };
        rusqlite::Result::Ok(sql)
    }
}

impl FromSql for Sauce {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let row = match value.as_str().unwrap() {
            NOT_EXISTING_FLAG => NotExisting,
            NOT_CHECKED_FLAG => NotChecked,
            sauce => Match(String::from(sauce))
        };
        FromSqlResult::Ok(row)
    }
}

pub const NOT_EXISTING_FLAG: &str =
    "NOT_EXISTING";

pub const NOT_CHECKED_FLAG: &str =
    "NOT_CHECKED";