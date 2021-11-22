use std::io::ErrorKind::AlreadyExists;
use std::path::{Path, PathBuf};
use crate::common::error;

use crate::common::error::Error;
use crate::{ImageHandle, Sauce};

pub fn import_file(lib: &str, file: &Path, new_filename: &str) -> Result<ImageHandle, Error> {
    let lib_path = PathBuf::from(lib);
    std::fs::create_dir_all(&lib_path).or_else(|err|
        Err(Error::DirectoryCreateError(err, String::from(lib)))
    )?;

    let mut new_path = PathBuf::from(&lib_path);
    new_path.push(new_filename);
    std::fs::hard_link(file, new_path).or_else(|err| {
        if err.kind()==AlreadyExists {
            Ok(())
        } else {
            Err(Error::HardLinkError(err, error::get_path(file)))
        }
    })?;
    Ok(ImageHandle::new(String::from(new_filename), Sauce::NotChecked))
}