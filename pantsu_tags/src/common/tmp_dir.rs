use std::{path::{Path, PathBuf}, fs::{self, File}};
use log::warn;

use crate::common;
use crate::{Result, Error};

const TMP_TOP_DIR_NAME: &str = "pantsu-tags-tmp";

pub struct TmpFile {
    path: PathBuf,
}

impl TmpFile {
    pub(crate) fn new(path: PathBuf) -> TmpFile {
        TmpFile { path }
    }
    pub fn get_path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TmpFile {
    fn drop(&mut self) {
        assert!(self.path.starts_with(std::env::temp_dir()));
        if let Err(_) = fs::remove_file(&mut self.path) {
            warn!("warning: failed to remove temporary file '{}'", common::get_path(&self.path));
        }
    }
}

pub fn create_tmp_file(sub_dir_name: &str, filename: &str) -> Result<(TmpFile,File)> {
    let mut path = get_tmp_dir(sub_dir_name)?;
    path.push(filename);
    let file = File::create(&path)
        .or_else(|e| Err(Error::FileCreateError(e, common::get_path(&path))))?;
    let tmp_path = TmpFile::new(path);
    Ok((tmp_path,file))
}

pub fn get_tmp_dir(sub_dir_name: &str) -> Result<PathBuf> {
    let mut tmp_dir = std::env::temp_dir();
    tmp_dir.push(TMP_TOP_DIR_NAME);
    tmp_dir.push(sub_dir_name);
    fs::create_dir_all(&tmp_dir)
        .or_else(|err| Err(Error::DirectoryCreateError(err, common::get_path(&tmp_dir))))?;
    Ok(tmp_dir)
}