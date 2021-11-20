use std::path::{Path, PathBuf};
use colored::Colorize;
use structopt::StructOpt;
use pantsu_tags::db::PantsuDB;
use pantsu_tags::{Error, ImageFile, PantsuTag};
use pantsu_tags::sauce::SauceMatch;
use crate::cli::Args;

mod cli;

// sauce matches with a higher similarity will be automatically accepted
const FOUND_SIMILARITY_THRESHOLD: i32 = 90;
// sauce matches with a higher similarity are relevant. (Others will be discarded)
const RELEVANT_SIMILARITY_THESHOLD: i32 = 45;

fn main() -> Result<(), AppError> {
    let args = Args::from_args();
    println!("Got arguments {:?}", args);
    let res = match args {
        Args::Import{no_auto_sources, images} => {
            import(no_auto_sources, images)
        }
        Args::Get{ include_tags, exclude_tags, temp_dir }  => {
            get(include_tags, exclude_tags, temp_dir)
        }
        Args::ListTags { tag_type: _tag_types } => {
            //TODO
            Ok(())
        }
    };

    match res {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(e)
        },
    }
}

fn import(no_auto_sources: bool, images: Vec<PathBuf>) -> Result<(), AppError> {
    #[derive(Default)]
    struct ImportStats {
        success: i64,
        already_exists: i64,
        similar_exists: i64,
        could_not_open: i64,
        no_source: i64,
        source_unsure: i64,
    }

    let mut import_stats = ImportStats::default();
    let pdb_path = Path::new("./pantsu_tags.db");
    let mut pdb = PantsuDB::new(pdb_path)?;
    for image in images {
        let image_name = image.to_str().unwrap_or("(can't display image name)");
        match import_one_image(&mut pdb, &image, no_auto_sources) {
            Ok(_) => {
                import_stats.success += 1;
                println!("{} - {}", "Successfully imported image".green(), image_name);
            }
            Err(AppError::LibError(e)) => {
                match e {
                    Error::ImageAlreadyExists(_) => {
                        import_stats.already_exists += 1;
                        println!("{} - {}", "Image already exists       ", image_name);
                    },
                    Error::SimilarImagesExist(img, similar_images) => {
                        import_stats.similar_exists += 1;
                        println!("{} - {}", "Similar images exist       ".yellow(), image_name);
                    },
                    Error::ImageLoadError(_) | Error::FileNotFound(_, _) => {
                        import_stats.could_not_open += 1;
                        println!("{} - {}", "Failed to open image       ", image_name);
                    }
                    error => return Err(AppError::LibError(error)),
                }
            }
            Err(AppError::NoRelevantSauces) => {
                import_stats.no_source += 1;
                println!("{} - {}", "No source found            ".red(), image_name);
            },
            Err(AppError::SauceUnsure(sauce_matches)) => {
                import_stats.source_unsure += 1;
                println!("{} - {}", "Source could be wrong      ".yellow(), image_name);
            }
        }
    }
    println!("\n\n{}", "Done".green().blink());
    println!("Successfully imported: {}", import_stats.success);
    println!("Similar image exists:  {}", import_stats.similar_exists);
    println!("Source unsure:         {}", import_stats.source_unsure);
    println!("Source not found:      {}", import_stats.no_source);
    println!("Already exists:        {}", import_stats.already_exists);
    println!("Couldn't open image:   {}", import_stats.could_not_open);
    Ok(())
}

fn import_one_image(pdb: &mut PantsuDB, image: &PathBuf, no_auto_sources: bool) -> Result<(), AppError> {
    let image_handle = pantsu_tags::new_image_handle(pdb, &image, true)?;
    if no_auto_sources {
        let no_tags: Vec<PantsuTag> = Vec::new();
        pantsu_tags::store_image_with_tags(pdb, &image_handle, &no_tags)?;
    }
    else {
        let sauces = pantsu_tags::get_image_sauces(&image_handle)?;
        let relevant_sauces: Vec<SauceMatch> = sauces.into_iter().filter(|s| s.similarity > RELEVANT_SIMILARITY_THESHOLD).collect();
        match relevant_sauces.first() {
            Some(sauce) => {
                if sauce.similarity > FOUND_SIMILARITY_THRESHOLD {
                    let tags = pantsu_tags::get_sauce_tags(sauce)?;
                    pantsu_tags::store_image_with_tags_from_sauce(pdb, &image_handle, sauce, &tags)?;
                }
                else {
                    return Err(AppError::SauceUnsure(relevant_sauces));
                }
            }
            None => {
                return Err(AppError::NoRelevantSauces);
            }
        }
    }
    Ok(())
}

fn get(included_tags: Vec<String>, excluded_tags: Vec<String>, temp_dir: Option<PathBuf>) -> Result<(), AppError> {
    let lib_dir = Path::new("./test_image_lib/");
    let pdb = PantsuDB::new(Path::new("./pantsu_tags.db")).unwrap();
    let files = pdb.get_files_with_tags_but(&included_tags, &excluded_tags)?;

    match temp_dir {
        Some(path) => link_files_to_tmp_dir(&files, lib_dir, &path),
        None => print_file_paths(&files, lib_dir),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Couldn't find relevant sauces")]
    NoRelevantSauces,

    #[error("Not sure whether sauce is correct or not")]
    SauceUnsure(Vec<SauceMatch>),

    #[error(transparent)]
    LibError(#[from] Error),
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