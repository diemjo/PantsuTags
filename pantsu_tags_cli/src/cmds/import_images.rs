use std::io;
use std::path::{Path, PathBuf};
use colored::Colorize;
use pantsu_tags::db::PantsuDB;
use pantsu_tags::{Error, ImageHandle};
use crate::common::{AppError, AppResult};
use crate::feh;
use crate::feh::FehProcesses;

pub fn import_images(no_feh: bool, images: Vec<PathBuf>) -> AppResult<()> {
    let mut import_stats = ImportStats::default();
    let mut similar_images_cases: Vec<SimilarImagesExistCase> = Vec::new();
    let pdb_path = Path::new("./pantsu_tags.db");
    let mut pdb = PantsuDB::new(pdb_path)?;
    for image in &images {
        let image_name = image.to_str().unwrap_or("(can't display image name)");
        let res = pantsu_tags::new_image_handle(&mut pdb, image, true);
        match res {
            Ok(_) => {
                import_stats.success += 1;
                println!("{} - {}", "Successfully imported image".green(), image_name);
            }
            Err(Error::ImageAlreadyExists(_)) => {
                import_stats.already_exists += 1;
                println!("{} - {}", "Image already exists       ", image_name);
            },
            Err(Error::SimilarImagesExist(img, similar_images)) => {
                similar_images_cases.push(SimilarImagesExistCase {
                    new_image_path: img,
                    similar_images
                });
                println!("{} - {}", "Similar images exist       ".yellow(), image_name);
            },
            Err(Error::ImageLoadError(_)) => {
                import_stats.could_not_open += 1;
                println!("{} - {}", "Failed to open image       ", image_name);
            }
            Err(error) => return Err(AppError::LibError(error)),
        }
    }
    resolve_similar_images(&mut pdb, similar_images_cases, &mut import_stats, no_feh)?;
    println!();
    import_stats.print_stats();
    Ok(())
}

fn resolve_similar_images(pdb: &mut PantsuDB, images_to_resolve: Vec<SimilarImagesExistCase>, stats: &mut ImportStats, no_feh: bool) -> AppResult<()> {
    if images_to_resolve.is_empty() {
        return Ok(());
    }
    let use_feh = !no_feh && feh::feh_available();
    let mut input = String::new();
    let stdin = io::stdin();
    println!("\n\nResolving {} images which are similar to images in PantsuTags:", images_to_resolve.len());
    for case in &images_to_resolve {
        let image_name = case.new_image_path.to_str().unwrap_or("(can't display image name)");
        println!("\nImage {} is similar to", image_name);
        for similar_img in &case.similar_images {
            println!(" - {}", similar_img.get_filename());
        }
        let procs = feh_display_similar(image_name, &case.similar_images, use_feh);
        println!("Do you still want to add the new image to PantsuTags?");
        println!("[y/N]");
        input.clear();
        stdin.read_line(&mut input).or_else(|e| Err(AppError::StdinReadError(e)))?;
        let input = input.trim();
        if input.eq_ignore_ascii_case("y")  {
            match pantsu_tags::new_image_handle(pdb, &case.new_image_path, false) {
                Ok(_) => {
                    stats.similar_imported += 1;
                    println!("Imported new image");
                },
                Err(e) => {
                    eprintln!("Failed to import new image, Error: {}", e);
                }
            }
        }
        else {
            stats.similar_not_imported += 1;
            println!("New image {} imported", "was not".bold());
        }
        procs.kill();
    }
    Ok(())
}

fn feh_display_similar(image_path: &str, similar_images: &Vec<ImageHandle>, use_feh: bool) -> FehProcesses {
    if use_feh {
        let similar_images: Vec<String> = similar_images.iter().map(|img| img.get_path()).collect();
        let similar_images = similar_images.iter().map(|path| path.as_str()).collect();
        return feh::feh_compare_image(
            image_path,
            &similar_images,
            "New image",
            "Similar image already stored in PantsuTags"
        );
    }
    FehProcesses::new_empty()
}

#[derive(Default)]
struct ImportStats {
    success: u64,
    similar_imported: u64,
    similar_not_imported: u64,
    already_exists: u64,
    could_not_open: u64,
}
impl ImportStats {
    fn print_stats(&self) {
        if self.success > 0 {
            println!("Successfully imported: {}", self.success);
        }
        if self.similar_imported > 0 || self.similar_not_imported > 0 {
            println!("Similar image exists:");
            if self.similar_imported > 0 {
                println!("- Still imported:      {}", self.similar_imported);
            }
            if self.similar_not_imported > 0 {
                println!("- Thus not imported:   {}", self.similar_not_imported);
            }
        }
        if self.already_exists > 0 {
            println!("Already exists:        {}", self.already_exists);
        }
        if self.could_not_open > 0 {
            println!("Couldn't open image:   {}", self.could_not_open);
        }
    }
}


struct SimilarImagesExistCase {
    new_image_path: PathBuf,
    similar_images: Vec<ImageHandle>,
}