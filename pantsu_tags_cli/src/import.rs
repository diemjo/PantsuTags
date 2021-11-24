use std::path::{Path, PathBuf};
use colored::Colorize;
use pantsu_tags::db::PantsuDB;
use pantsu_tags::Error;
use crate::common::{AppError, AppResult};
use crate::autotags;
use crate::autotags::SauceUnsure;

pub fn import(no_auto_sources: bool, no_feh: bool, images: Vec<PathBuf>) -> AppResult<()> {
    #[derive(Default)]
    struct ImportStats {
        success: u64,
        already_exists: u64,
        similar_exists: u64,
        could_not_open: u64,
        no_source: u64,
        source_unsure: u64,
    }

    let mut import_stats = ImportStats::default();
    let mut unsure_source_images: Vec<SauceUnsure> = Vec::new();
    let pdb_path = Path::new("./pantsu_tags.db");
    let mut pdb = PantsuDB::new(pdb_path)?;
    for image in &images {
        let image_name = image.to_str().unwrap_or("(can't display image name)");
        let res = if no_auto_sources {
            import_one_image(&mut pdb, &image)
        }
        else {
            import_one_image_auto_source(&mut pdb, &image)
        };
        match res {
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
            Err(AppError::SauceUnsure(image_handle, sauce_matches)) => {
                import_stats.source_unsure += 1;
                unsure_source_images.push(SauceUnsure {
                    path: &image,
                    image_handle,
                    matches: sauce_matches,
                });
                println!("{} - {}", "Source could be wrong      ".yellow(), image_name);
            },
            Err(e@AppError::StdinReadError(_)) => eprintln!("Unexpected error: {}", e),
        }
    }
    println!("\n\n{}", "Done".green().blink());
    println!("Successfully imported: {}", import_stats.success);
    println!("Similar image exists:  {}", import_stats.similar_exists);
    println!("Source unsure:         {}", import_stats.source_unsure);
    println!("Source not found:      {}", import_stats.no_source);
    println!("Already exists:        {}", import_stats.already_exists);
    println!("Couldn't open image:   {}", import_stats.could_not_open);

    // todo: handle similar images here

    autotags::resolve_sauce_unsure(&mut pdb, unsure_source_images, no_feh)?;
    Ok(())
}

fn import_one_image_auto_source(pdb: &mut PantsuDB, image: &PathBuf) -> AppResult<()> {
    let image_handle = pantsu_tags::new_image_handle(pdb, &image, true)?;
    autotags::auto_add_tags(pdb, image_handle)
}

fn import_one_image(pdb: &mut PantsuDB, image: &Path) -> AppResult<()> {
    pantsu_tags::new_image_handle(pdb, &image, true)?;
    Ok(())
}