use std::path::Path;

pub mod error;
pub mod pantsu_tag;
pub mod image_handle;

pub fn get_path(path: &Path) -> String {
    String::from(path.to_str().unwrap_or("cannot display path"))
}