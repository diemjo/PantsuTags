use crate::ImageHandle;
use crate::PantsuTag;
use crate::Sauce;
use crate::Error;
use crate::common::image_handle::NOT_CHECKED_FLAG;
use crate::common::image_handle::NOT_EXISTING_FLAG;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use crate::common::error::Result;
use crate::common;

use super::PantsuDB;

pub(crate) fn import_db_file(pdb: &mut PantsuDB, file: &Path) -> Result<()> {
    let content = std::fs::read_to_string(file).or_else(|e| Err(Error::FileNotFound(e, common::get_path(file))))?;
    let images = content.lines()
        .filter(|l| !l.is_empty())
        .map(|l| decode_image_info(file, l.to_string()))
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

pub(crate) fn export_db_file(pdb: &PantsuDB, path: &Path) -> Result<()> {
    let images = pdb.get_images_transaction()
        .execute()?;
    let lines = images.into_iter()
        .map(|i| encode_image_info(&pdb, i))
        .collect::<Result<Vec<String>>>()?;
    let mut file = std::fs::File::create(path).or_else(|e| Err(Error::FileCreateError(e, common::get_path(path))))?;
    let content = lines.join("\n");
    file.write_all(content.as_bytes()).or_else(|e| Err(Error::FileWriteError(e, common::get_path(path))))?;
    Ok(())
}


fn encode_image_info(pdb: &PantsuDB, i: ImageHandle) -> Result<String> {
    let tags = pdb.get_tags_transaction()
        .for_image(i.get_filename())
        .execute()?;
    let name = i.get_filename();
    let sauce = match i.get_sauce() {
        Sauce::Match(m) => m,
        Sauce::NotExisting => Sauce::NotExisting.get_type(),
        Sauce::NotChecked => Sauce::NotChecked.get_type(),
    };
    let tags = tags.iter().map(|t| t.to_string()).collect::<Vec<String>>().join(",");
    Ok(format!("{},{},{}", name, sauce, tags))
}

fn decode_image_info(file: &Path, line: String) -> Result<(String, Sauce, Vec<PantsuTag>)> {
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