use std::fmt;
use std::fmt::Formatter;

pub struct PantsuTag {
    pub tag_name: String,
    pub tag_type: PantsuTagType
}

#[derive(Debug)]
pub enum PantsuTagType {
    Artist,
    Source,
    Character,
    Generic
}

impl fmt::Display for PantsuTagType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}