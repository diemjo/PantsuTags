use log::info;

use crate::Sauce;
use crate::Error;
use crate::common::image_info::ImageInfo;
use crate::common::pantsu_tag::PantsuTagInfo;
use std::io::Write;
use std::path::Path;
use crate::common::error::Result;
use crate::{common};

use super::PantsuDB;

pub(crate) fn import_tags(pdb: &mut PantsuDB, file: &Path) -> Result<()> {
    let content = std::fs::read_to_string(file).or_else(|e| Err(Error::FileNotFound(e, common::get_path(file))))?;
    let mut lines = content.lines();
    let first_line = lines.next();
    match first_line {
        Some(file_version) => {
            let local_version = pdb.get_db_version()?;
            match file_version.parse::<usize>() {
                Ok(file_version) => if local_version != file_version {
                    return Err(Error::DatabaseVersionMismatch(common::get_path(file), local_version, file_version));
                },
                Err(_) => {
                    return Err(Error::InvalidImportFileFormat(common::get_path(file), None));
                },
            }
        },
        None => {
            return Err(Error::InvalidImportFileFormat(common::get_path(file), None));
        }
    }
    let images = lines
        .filter(|l| !l.is_empty())
        .map(|l| deserialize_line(l.to_string()))
        .collect::<Result<Vec<(ImageInfo, Vec<PantsuTagInfo>)>>>()
        .or_else(|e| Err(Error::InvalidImportFileFormat(common::get_path(file), Some(Box::new(e)))))?;
    for (image_info, tags) in images {
        let local_image = match pdb.get_image_transaction(image_info.get_image()).execute()? {
            Some(image) => image,
            None => continue
        };
        for tag in tags {
            pdb.update_images_transaction()
            .for_image(image_info.get_image())
            .tag_author(&tag.tag_author)
            .add_tag(&tag.tag)
            .execute()?;
        }
        let local_sauce = local_image.get_sauce();
        let transaction = pdb.update_images_transaction()
            .for_image(image_info.get_image());
        match (local_sauce, image_info.get_sauce()) {
            (Sauce::Match(_), _) => transaction.execute(),
            (_, Sauce::NotChecked) => transaction.execute(),
            (_, _) => transaction.update_sauce(image_info.get_sauce()).execute()
        }?;
        info!("Updated image '{}' from import file", image_info.get_image().get_filename())
    }
    Ok(())
}

pub(crate) fn export_tags(pdb: &PantsuDB, path: &Path) -> Result<()> {
    let images = pdb.get_images_transaction()
        .execute()?;
    let lines = images.into_iter()
        .map(|i| serialize_image(&pdb, &i))
        .collect::<Result<Vec<String>>>()?;
    let mut file = std::fs::File::create(path).or_else(|e| Err(Error::FileCreateError(e, common::get_path(path))))?;
    let content = format!("{}\n{}", pdb.get_db_version()?, lines.join("\n"));
    file.write_all(content.as_bytes()).or_else(|e| Err(Error::FileWriteError(e, common::get_path(path))))?;
    Ok(())
}


fn serialize_image(pdb: &PantsuDB, image: &ImageInfo) -> Result<String> {
    let tags = pdb.get_image_tags_transaction(image.get_image())
        .execute()?;
    let image_info = image.serialize();
    let tags = tags.iter().map(|t| t.serialize()).collect::<Vec<String>>().join(",");
    Ok(format!("{},{}", image_info, tags))
}

fn deserialize_line(line: String) -> Result<(ImageInfo, Vec<PantsuTagInfo>)> {
    let items: Vec<&str> = line.splitn(2, ',').collect();
    if items.len() != 2 {
        return Err(Error::InvalidImportFileLineFormat(line));
    }
    let image_info = ImageInfo::deserialize(items[0])?;
    let tags = parse_image_tags(items[1])?;
    Ok((image_info, tags))
}

fn parse_image_tags(value: &str) -> Result<Vec<PantsuTagInfo>> {
    value.split(',')
        .filter(|l| !l.is_empty())
        .map(|t| PantsuTagInfo::deserialize(t))
        .collect::<Result<Vec<PantsuTagInfo>>>()
}