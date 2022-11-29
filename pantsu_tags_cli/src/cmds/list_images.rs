use std::path::{Path, PathBuf};
use std::str::FromStr;
use pantsu_tags::db::{AspectRatio, PantsuDB};
use pantsu_tags::{Error, PantsuTag, ImageInfo};
use crate::common::{AppResult};
use crate::{common, CONFIGURATION};

pub fn list_images(included_tags: &Vec<String>, excluded_tags: &Vec<String>, ratio: AspectRatio, do_print_filenames: bool,
                   sauce_existing: bool, sauce_not_existing: bool, sauce_not_checked: bool,
                   temp_dir: Option<PathBuf>) -> AppResult<()> {
    let pdb = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    let included_tags = included_tags.into_iter()
        .map(|t| PantsuTag::from_str(t).or_else(|_| Ok(PantsuTag::new(t.to_string(), pantsu_tags::PantsuTagType::General))))
        .collect::<AppResult<Vec<PantsuTag>>>()?;
    let excluded_tags = excluded_tags.into_iter()
        .map(|t| PantsuTag::from_str(t).or_else(|_| Ok(PantsuTag::new(t.to_string(), pantsu_tags::PantsuTagType::General))))
        .collect::<AppResult<Vec<PantsuTag>>>()?;
    let images_transaction = pdb.get_images_transaction()
        .including_tags(&included_tags)
        .excluding_tags(&excluded_tags)
        .with_ratio(ratio);

    let images_transaction = if sauce_existing {
        images_transaction.with_existing_sauce()
    } else if sauce_not_existing {
        images_transaction.with_not_existing_sauce()
    } else if sauce_not_checked {
        images_transaction.with_not_checked_sauce()
    } else {
        images_transaction
    };

    let images = images_transaction.execute()?;

    let lib_dir = CONFIGURATION.library_path.as_path();
    match temp_dir {
        Some(path) => link_files_to_tmp_dir(&images, lib_dir, &path),
        None => match do_print_filenames {
            false => print_file_paths(&images, lib_dir),
            true => print_filenames(&images),
        }
    }
}

fn link_files_to_tmp_dir(files: &Vec<ImageInfo>, lib_dir: &Path, tmp_path: &PathBuf) -> AppResult<()> {
    let paths = get_file_paths(files, lib_dir)?;
    std::fs::create_dir_all(tmp_path).or_else(|err| Err(Error::DirectoryCreateError(err, common::get_path(tmp_path))))?;
    for (lib_path, i) in paths.iter().zip(files) {
        let tmp_path = tmp_path.join(i.get_image().get_filename());
        #[cfg(not(target_os = "windows"))]
        std::os::unix::fs::symlink(lib_path, &tmp_path)
            .or_else(|err| Err(Error::HardLinkError(err, common::get_path(lib_path))))?;
        #[cfg(target_os = "windows")]
        std::os::windows::symlink_file(lib_path, &tmp_path)?;
        println!("Linked file: {}", common::get_path(&tmp_path));
    }
    Ok(())
}

fn print_file_paths(files: &Vec<ImageInfo>, lib_dir: &Path) -> AppResult<()> {
    for p in get_file_paths(files, lib_dir)? {
        println!("{}", common::get_path(&p))
    }
    Ok(())
}

fn print_filenames(images: &Vec<ImageInfo>) -> AppResult<()> {
    for image in images {
        println!("{}", image.get_image().get_filename())
    }
    Ok(())
}

fn get_file_paths(files: &Vec<ImageInfo>, lib_dir: &Path) -> AppResult<Vec<PathBuf>> {
    std::fs::create_dir_all(lib_dir).or_else(|err|
        Err(Error::DirectoryCreateError(err, common::get_path(lib_dir)))
    )?;
    let lib_dir_abs = lib_dir.canonicalize().unwrap();
    Ok(files.iter()
        .map(|f| {
            lib_dir_abs.join(f.get_image().get_filename())
        })
        .collect())
}