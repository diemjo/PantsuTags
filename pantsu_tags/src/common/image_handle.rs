use std::fmt::Debug;
use rusqlite::ToSql;
use rusqlite::types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef};
use crate::common::image_handle::Sauce::Match;
use crate::Sauce::{NonExistent, NotChecked};


#[derive(Debug, PartialEq)]
pub struct ImageHandle {
    filename: String,
    file_source: Sauce
}

impl ImageHandle {
    pub(crate) fn new(filename: String, file_source: Sauce) -> ImageHandle {
        ImageHandle {
            filename,
            file_source
        }
    }

    pub fn get_filename(&self) -> &str {
        self.filename.as_str()
    }

    pub fn get_sauce(&self) -> &Sauce {
        &self.file_source
    }
}

#[derive(Debug, PartialEq)]
pub enum Sauce {
    Match(String),
    NonExistent,
    NotChecked
}

impl ToSql for Sauce {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let sql = match self {
            Match(sauce) => ToSqlOutput::from(sauce.as_str()),
            NonExistent => ToSqlOutput::from(NON_EXISTENT_FLAG),
            NotChecked => ToSqlOutput::from(NOT_CHECKED_FLAG)
        };
        rusqlite::Result::Ok(sql)
    }
}

impl FromSql for Sauce {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let row = match value.as_str().unwrap() {
            NON_EXISTENT_FLAG => NonExistent,
            NOT_CHECKED_FLAG => NotChecked,
            sauce => Match(String::from(sauce))
        };
        FromSqlResult::Ok(row)
    }
}

const NON_EXISTENT_FLAG: &str =
    "NON_EXISTENT";

const NOT_CHECKED_FLAG: &str =
    "NOT_CHECKED";