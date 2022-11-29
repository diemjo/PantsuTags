use std::path::Path;

pub mod error;
pub mod pantsu_tag;
pub mod image_handle;
pub mod image_info;
pub mod tmp_dir;

pub use tmp_dir::tmp_dir_async as tmp_dir_async;

pub fn get_path(path: &Path) -> String {
    String::from(path.to_str().unwrap_or("cannot display path"))
}