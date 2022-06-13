use std::path::{Path, PathBuf};
use pantsu_tags::db::{AspectRatio, PantsuDB};
use pantsu_tags::{Error, ImageHandle};
use crate::common::AppResult;

pub fn list_images(included_tags: &Vec<String>, excluded_tags: &Vec<String>, ratio: AspectRatio,
                   sauce_existing: bool, sauce_not_existing: bool, sauce_not_checked: bool,
                   temp_dir: Option<PathBuf>) -> AppResult<()> {
    let lib_dir = Path::new("./test_image_lib/");
    let pdb = PantsuDB::new(Path::new("./pantsu_tags.db"))?;
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

    match temp_dir {
        Some(path) => link_files_to_tmp_dir(&images, lib_dir, &path),
        None => print_file_paths(&images, lib_dir),
    }
}

fn link_files_to_tmp_dir(files: &Vec<ImageHandle>, lib_dir: &Path, tmp_path: &PathBuf) -> AppResult<()> {
    let paths = get_file_paths(files, lib_dir)?;
    std::fs::create_dir_all(tmp_path).or_else(|err| Err(Error::DirectoryCreateError(err, String::from(tmp_path.to_str().unwrap_or_default()))))?;
    for (lib_path, i) in paths.iter().zip(files) {
        let tmp_path = tmp_path.join(i.get_filename());
        #[cfg(not(target_os = "windows"))]
        std::os::unix::fs::symlink(lib_path, &tmp_path)
            .or_else(|err| Err(Error::HardLinkError(err, String::from(lib_path.to_str().unwrap_or_default()))))?;
        #[cfg(target_os = "windows")]
        std::os::windows::symlink_file(lib_path, &tmp_path)?;
        println!("Linked file: {}", tmp_path.to_str().unwrap_or_default());
    }
    Ok(())
}

fn print_file_paths(files: &Vec<ImageHandle>, lib_dir: &Path) -> AppResult<()> {
    for p in get_file_paths(files, lib_dir)? {
        println!("{}", p.to_str().unwrap())
    }
    Ok(())
}

fn get_file_paths(files: &Vec<ImageHandle>, lib_dir: &Path) -> AppResult<Vec<PathBuf>> {
    std::fs::create_dir_all(lib_dir).or_else(|err|
        Err(Error::DirectoryCreateError(err, String::from(lib_dir.to_str().unwrap_or_default())))
    )?;
    let lib_dir_abs = lib_dir.canonicalize().unwrap();
    Ok(files.iter()
        .map(|f| {
            lib_dir_abs.join(f.get_filename())
        })
        .collect())
}