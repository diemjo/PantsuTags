use std::path::Path;

use pantsu_tags::db::PantsuDB;

use crate::{common::AppResult, CONFIGURATION};

pub fn import_tags(path: &Path) -> AppResult<()> {
    let mut pdb = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    pdb.import_tags(path)?;
    Ok(())
}

pub fn export_tags(path: &Path) -> AppResult<()> {
    let mut pdb = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    pdb.export_tags(path)?;
    Ok(())
}