use std::{path::{Path, PathBuf}};
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
        if let Err(_) = std::fs::remove_file(&mut self.path) {
            warn!("warning: failed to remove temporary file '{}'", common::get_path(&self.path));
        }
    }
}

pub mod tmp_dir_async {
    use super::*;

    pub async fn create_tmp_file(sub_dir_name: &str, filename: &str) -> Result<(TmpFile,tokio::fs::File)> {
        let mut path = get_tmp_dir(sub_dir_name).await?;
        path.push(filename);
        let file = tokio::fs::File::create(&path).await
            .or_else(|e| Err(Error::FileCreateError(e, common::get_path(&path))))?;
        let tmp_path = TmpFile::new(path);
        Ok((tmp_path,file))
    }

    pub async fn get_tmp_dir(sub_dir_name: &str) -> Result<PathBuf> {
        let mut tmp_dir = std::env::temp_dir();
        tmp_dir.push(TMP_TOP_DIR_NAME);
        tmp_dir.push(sub_dir_name);
        tokio::fs::create_dir_all(&tmp_dir).await
            .or_else(|err| Err(Error::DirectoryCreateError(err, common::get_path(&tmp_dir))))?;
        Ok(tmp_dir)
    }
}