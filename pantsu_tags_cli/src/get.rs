use std::path::{Path, PathBuf};
use pantsu_tags::db::PantsuDB;
use pantsu_tags::{Error, ImageHandle};
use crate::common::AppResult;

pub fn get(included_tags: Vec<String>, excluded_tags: Vec<String>, temp_dir: Option<PathBuf>) -> AppResult<()> {
    let lib_dir = Path::new("./test_image_lib/");
    let pdb = PantsuDB::new(Path::new("./pantsu_tags.db"))?;
    let files = pdb.get_files_with_tags_except(&included_tags, &excluded_tags)?;

    match temp_dir {
        Some(path) => link_files_to_tmp_dir(&files, lib_dir, &path),
        None => print_file_paths(&files, lib_dir),
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