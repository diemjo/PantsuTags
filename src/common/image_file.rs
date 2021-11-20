use std::fmt::Debug;

#[derive(Debug, PartialEq)]
pub struct ImageFile {
    pub filename: String,
    pub file_source: Option<String>
}