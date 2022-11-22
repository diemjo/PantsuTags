use std::io;
use std::num::ParseIntError;
use std::path::PathBuf;
use colored::Colorize;
use log::{error, info, warn};
use pantsu_tags::db::PantsuDB;
use pantsu_tags::{Error, image_similarity};
use pantsu_tags::image_similarity::{ImageToImport, SimilarImagesGroup};
use crate::common::{AppError, AppResult};
use crate::{common, CONFIGURATION, feh};
use crate::feh::FehProcesses;

pub fn import_images(no_feh: bool, images: Vec<PathBuf>, always_copy_images: bool) -> AppResult<()> {
    let mut import_stats = ImportStats::default();
    let mut valid_images: Vec<ImageToImport> = Vec::new();
    let mut pdb = PantsuDB::new(CONFIGURATION.database_path.as_path())?;

    for image in &images {
        let image_name = common::get_path(image);
        match pantsu_tags::check_image(&mut pdb, image) {
            Ok(img) => valid_images.push(img),
            Err(Error::ImageAlreadyExists(_)) => {
                import_stats.already_exists += 1;
                println!("{} - {}", "Image already exists       ", image_name);
                warn!("Image already exists: '{}'", image_name);
            }
            Err(Error::ImageLoadError(_)) => {
                import_stats.could_not_open += 1;
                println!("{} - {}", "Failed to open image       ", image_name);
                error!("Failed to open image: '{}'", image_name);
            }
            Err(error) => return Err(AppError::LibError(error)),
        }
    }

    let images_in_db = pdb.get_images_transaction().execute()?;
    let image_groups = image_similarity::group_similar_images(&valid_images, &images_in_db)?;

    let mut image_groups_with_similars: Vec<SimilarImagesGroup> = Vec::new();
    for group in image_groups {
        if group.is_single_image() {
            let image = group.new_images.into_iter().next().unwrap();
            pantsu_tags::import_image(&mut pdb, CONFIGURATION.library_path.as_path(), image, always_copy_images)?;
            import_stats.success += 1;
            let image_name = common::get_path(&image.current_path);
            println!("{} - {}", "Successfully imported image".green(), image_name);
            info!("Imported image: '{}'", image_name);
        }
        else {
            for image in &group.new_images {
                let image_name = common::get_path(&image.current_path);
                println!("{} - {}", "Similar images exist       ".yellow(), image_name);
                warn!("Similar images exist to: '{}'", image_name);
            }
            image_groups_with_similars.push(group);
        }
    }

    resolve_similar_image_groups(&mut pdb, image_groups_with_similars, &mut import_stats, always_copy_images, no_feh)?;
    println!();
    import_stats.print_stats();
    Ok(())
}

fn resolve_similar_image_groups(pdb: &mut PantsuDB, similar_images_groups: Vec<SimilarImagesGroup>, stats: &mut ImportStats, always_copy_images: bool, no_feh: bool) -> AppResult<()> {
    if similar_images_groups.is_empty() {
        return Ok(());
    }
    let use_feh = !no_feh && feh::feh_available();
    let mut input = String::new();
    let stdin = io::stdin();
    let num_groups = similar_images_groups.len();
    println!("\n\nResolving {} groups of images which are similar to each other.", num_groups);
    for (group_idx, group) in similar_images_groups.iter().enumerate() {
        println!("\nGroup {} of {}:", group_idx+1, num_groups);
        println!("  New images:");
        let new_images = &group.new_images;
        for (idx, new_img) in new_images.iter().enumerate() {
            let image_name = common::get_path(&new_img.current_path);
            println!("    {} - {}", idx+1, image_name)
        }
        if !group.old_images.is_empty() {
            println!("  Images already in PantsuTags:");
            for old_img in &group.old_images {
                let image_name = old_img.get_filename();
                println!("      - {}", image_name)
            }
        }

        let procs = feh_display_similar(group, use_feh);
        loop {
            println!("Select the new images that should be added to PantsuTags: (eg: \"1\", \"1 3 12\". \"0\" or empty to select none)");
            input.clear();
            stdin.read_line(&mut input).or_else(|e| Err(AppError::StdinReadError(e)))?;
            let input = input.trim();
            let input_numbers = input.split_whitespace()
                .map(|num| num.parse::<usize>())
                .collect::<Result<Vec<usize>, ParseIntError>>();

            if let Ok(numbers) = input_numbers { // todo: no input adds no images, maybe make clearer?
                let num_new_images = new_images.len();
                if numbers.iter().all(|&num| num <= num_new_images) {
                    for (idx, new_image) in new_images.iter().enumerate() {
                        let image_name = common::get_path(&new_image.current_path);
                        if numbers.iter().any(|&num| idx == num-1) {
                            pantsu_tags::import_image(pdb, CONFIGURATION.library_path.as_path(), new_image, always_copy_images)?;
                            stats.similar_imported += 1;
                            println!("Imported new image {}: {}", idx+1, image_name);
                            info!("Imported similar image: '{}'", image_name);
                        }
                        else {
                            stats.similar_not_imported += 1;
                            println!("New image {} {} imported", image_name, "was not".bold());
                            warn!("Skipping similar images: '{}'", image_name);
                        }
                    }

                    break;
                } else {
                    println!("Invalid input: the highest image number is {}", num_new_images);
                    continue;
                }
            }

            println!("Invalid input");
        }

        procs.kill();
    }
    Ok(())
}

fn feh_display_similar(similar_images: &SimilarImagesGroup, use_feh: bool) -> FehProcesses  {
    let mut feh_proc = FehProcesses::new_empty();
    if !use_feh {
        return feh_proc;
    }

    let new_image_names: AppResult<Vec<String>> = similar_images.new_images.iter()
        .map(|img| common::try_get_path(&img.current_path)).collect();
    match new_image_names {
        Ok(new_image_names) => feh_proc = feh::feh_display_images(new_image_names.iter().map(|img_name| img_name.as_str()), "New image", feh_proc),
        Err(_) => return feh_proc,
    }

    if !similar_images.old_images.is_empty() {
        let lib_path = CONFIGURATION.library_path.as_path();
        // store as vector since we need to pass a &str iterator to feh_display_images()
        let res_old_images: AppResult<Vec<String>> = similar_images.old_images.iter()
            .map(|img| common::try_get_path(&img.get_path(lib_path))).collect();
        if let Ok(old_images) = res_old_images {
            feh_proc = feh::feh_display_images(old_images.iter().map(|img| img.as_str()),
                                            "Image already stored in PantsuTags", feh_proc);
        }
    }
    feh_proc
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