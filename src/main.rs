use std::path::{Path, PathBuf};
use colored::Colorize;
use structopt::StructOpt;
use pantsu_tags::db::PantsuDB;
use pantsu_tags::Error;
use pantsu_tags::PantsuTag;
use pantsu_tags::sauce::SauceMatch;

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
        Args::Get{ .. }  => {
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


#[derive(Debug, StructOpt)]
#[structopt(name = "PantsuTags", about = "PantsuTags CLI")]
enum Args {
    Import {
        #[structopt(short="s", long)]
        no_auto_sources: bool,

        #[structopt(parse(from_os_str))]
        images: Vec<PathBuf>,
    },
    Get {
        #[structopt(short, long)]
        dummy: bool,
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
