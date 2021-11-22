use std::fmt::Debug;

#[derive(Debug, PartialEq)]
pub struct ImageFile {
    pub filename: String,
    pub file_source: Option<String>
}

impl ImageFile {
    pub fn sauce_is_nonexistent(&self) -> bool {
        return match &self.file_source {
            Some(sauce) => sauce.eq(NONEXISTENT_FLAG),
            None => false
        }
    }

    pub fn sauce_is_not_checked_yet(&self) -> bool {
        return match &self.file_source {
            Some(_) => false,
            None => true
        }
    }
}

pub const NONEXISTENT_FLAG: &str =
    "NONEXISTENT";