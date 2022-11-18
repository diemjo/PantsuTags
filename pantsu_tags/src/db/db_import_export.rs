use crate::PantsuTag;
use crate::Sauce;
use crate::Error;
use crate::common::image_handle::NOT_CHECKED_FLAG;
use crate::common::image_handle::NOT_EXISTING_FLAG;
use std::path::Path;
use std::str::FromStr;
use crate::common::error::Result;
use crate::common;

use super::PantsuDB;

pub(crate) fn import_db_file(pdb: &mut PantsuDB, file: &Path) -> Result<()> {
    let content = std::fs::read_to_string(file).or_else(|e| Err(Error::FileNotFound(e, common::get_path(file))))?;
    let images = content.lines()
        .filter(|l| !l.is_empty())
        .map(|l| parse_image_info(file, l.to_string()))
        .collect::<Result<Vec<(String, Sauce, Vec<PantsuTag>)>>>()
        .or_else(|e| match e {
            Error::InvalidImportFileFormat(_, _) => Err(e),
            _ => Err(Error::InvalidImportFileFormat(common::get_path(file), Some(Box::new(e))))
         })?;
    for (name, sauce, tags) in images {
        let local_image = match pdb.get_image_transaction(&name).execute()? {
            Some(image) => image,
            None => continue
        };
        let mut transaction = pdb.update_images_transaction()
            .for_image(&name)
            .add_tags(&tags);
        let local_sauce = local_image.get_sauce();
        transaction = match (local_sauce, &sauce) {
            (Sauce::Match(_), _) => transaction,
            (_, Sauce::NotChecked) => transaction,
            (_, _) => transaction.update_sauce(&sauce)
        };
        transaction.execute()?;
    }
    Ok(())
}

fn parse_image_info(file: &Path, line: String) -> Result<(String, Sauce, Vec<PantsuTag>)> {
    let items: Vec<&str> = line.splitn(3, ',').collect();
    if items.len() != 3 {
        return Err(Error::InvalidImportFileFormat(common::get_path(file), None));
    }
    let name = items[0].to_string();
    let sauce = parse_image_sauce(items[1]);
    let tags = parse_image_tags(items[2])?;
    Ok((name, sauce, tags))
}

fn parse_image_sauce(value: &str) -> Sauce {
    match value {
        NOT_CHECKED_FLAG => Sauce::NotChecked,
        NOT_EXISTING_FLAG => Sauce::NotExisting,
        sauce => Sauce::Match(sauce.to_string())
    }
}

fn parse_image_tags(value: &str) -> Result<Vec<PantsuTag>> {
    value.split(',')
        .filter(|l| !l.is_empty())
        .map(|t| PantsuTag::from_str(t))
        .collect::<Result<Vec<PantsuTag>>>()
}