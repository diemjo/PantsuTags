use std::fmt::{Debug, Display, Formatter};
use crate::LIB_PATH;
use crate::Sauce::{NotExisting, NotChecked, Match};


#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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

impl Display for ImageHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: res={:>12}, sauce='{}'", self.filename, format!("({},{})", self.file_res.0, self.file_res.1), match self.get_sauce() {
            Match(url) => url,
            NotExisting => NOT_EXISTING_FLAG,
            NotChecked => NOT_CHECKED_FLAG,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Sauce {
    Match(String),
    NotExisting,
    NotChecked
}

impl Sauce {
    pub fn get_type(&self) -> &str {
        match self {
            Match(_) => EXISTING_FLAG,
            NotChecked => NOT_CHECKED_FLAG,
            NotExisting => NOT_EXISTING_FLAG,
        }
    }

    pub fn get_value(&self) -> Option<&str> {
        match self {
            Match(value) => Some(value.as_str()),
            NotChecked => None,
            NotExisting => None,
        }
    }
}

/*impl ToSql for Sauce {
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
}*/

pub const EXISTING_FLAG: &str =
    "EXISTING";
pub const NOT_EXISTING_FLAG: &str =
    "NOT_EXISTING";

pub const NOT_CHECKED_FLAG: &str =
    "NOT_CHECKED";