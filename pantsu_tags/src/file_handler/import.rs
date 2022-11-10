use std::io::ErrorKind::AlreadyExists;
use std::path::{Path, PathBuf};

use crate::common::error;
use crate::common::error::{Error, Result};

pub fn import_file(lib: &Path, file: &Path, new_filename: &str, always_copy: bool) -> Result<()> {
    let lib_path = PathBuf::from(lib);
    std::fs::create_dir_all(&lib_path).or_else(|err|
        Err(Error::DirectoryCreateError(err, String::from(lib.to_str().unwrap())))
    )?;

    let mut new_path = PathBuf::from(&lib_path);
    new_path.push(new_filename);
    if always_copy {
        std::fs::copy(file, new_path).or_else(|err| Err(Error::CopyError(err, error::get_path(file))))?;
    } else {
        std::fs::hard_link(file, new_path.as_path()).or_else(|err| {
            if err.kind() == AlreadyExists {
                Ok(())
            } else {
                match std::fs::copy(file, new_path.as_path()) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(Error::HardLinkError(err, error::get_path(file)))
                }
            }
        })?;
    }
    Ok(())
}