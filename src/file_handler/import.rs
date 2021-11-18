use std::io::ErrorKind::AlreadyExists;
use std::path::PathBuf;

use crate::common::error::Error;

pub fn import_file(lib: &str, file: &str, new_filename: &str) -> Result<(), Error> {
    let lib_path = PathBuf::from(lib);
    std::fs::create_dir_all(&lib_path).or_else(|err|
        Err(Error::DirectoryCreateError(err, String::from(lib)))
    )?;

    let mut new_path = PathBuf::from(&lib_path);
    new_path.push(new_filename);
    std::fs::hard_link(PathBuf::from(file), new_path).or_else(|err| {
        if err.kind()==AlreadyExists {
            Ok(())
        } else {
            Err(Error::HardLinkError(err, String::from(file)))
        }
    })?;
    Ok(())
}