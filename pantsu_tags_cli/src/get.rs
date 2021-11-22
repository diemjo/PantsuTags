use std::path::{Path, PathBuf};
use pantsu_tags::db::PantsuDB;
use pantsu_tags::{Error, ImageFile};
use crate::AppError;

pub fn get(included_tags: Vec<String>, excluded_tags: Vec<String>, temp_dir: Option<PathBuf>) -> Result<(), AppError> {
    let lib_dir = Path::new("./test_image_lib/");
    let pdb = PantsuDB::new(Path::new("./pantsu_tags.db")).unwrap();
    let files = pdb.get_files_with_tags_but(&included_tags, &excluded_tags)?;

    match temp_dir {
        Some(path) => link_files_to_tmp_dir(&files, lib_dir, &path),
        None => print_file_paths(&files, lib_dir),
    }
}

fn link_files_to_tmp_dir(files: &Vec<ImageFile>, lib_dir: &Path, tmp_path: &PathBuf) -> Result<(), AppError> {
    let paths = get_file_paths(files, lib_dir)?;
    std::fs::create_dir_all(tmp_path).or_else(|err| Err(Error::DirectoryCreateError(err, String::from(tmp_path.to_str().unwrap_or_default()))))?;
    for (lib_path, i) in paths.iter().zip(files) {
        let tmp_path = tmp_path.join(&i.filename);
        //#[cfg(target_os = "unix")]
        {
            std::os::unix::fs::symlink(lib_path, &tmp_path)
                .or_else(|err| Err(Error::HardLinkError(err, String::from(lib_path.to_str().unwrap_or_default()))))?;
            println!("Linked file: {}", tmp_path.to_str().unwrap_or_default());
        }
        #[cfg(target_os = "windows")]
            std::os::windows::symlink_file(lib_path, &tmp_path)?;
    }
    Ok(())
}

fn print_file_paths(files: &Vec<ImageFile>, lib_dir: &Path) -> Result<(), AppError> {
    for p in get_file_paths(files, lib_dir)? {
        println!("{}", p.to_str().unwrap())
    }
    Ok(())
}

fn get_file_paths(files: &Vec<ImageFile>, lib_dir: &Path) -> Result<Vec<PathBuf>, AppError> {
    std::fs::create_dir_all(lib_dir).or_else(|err|
        Err(Error::DirectoryCreateError(err, String::from(lib_dir.to_str().unwrap_or_default())))
    )?;
    let lib_dir_abs = lib_dir.canonicalize().unwrap();
    Ok(files.iter()
        .map(|f| {
            lib_dir_abs.join(&f.filename)
        })
        .collect())
}