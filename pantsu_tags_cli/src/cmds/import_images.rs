use std::io;
use std::num::ParseIntError;
use std::path::PathBuf;
use colored::Colorize;
use pantsu_tags::db::PantsuDB;
use pantsu_tags::Error;
use pantsu_tags::image_similarity::{ImageToImport, SimilarImagesGroup};
use crate::common::{AppError, AppResult};
use crate::{CONFIGURATION, feh};
use crate::feh::FehProcesses;

pub fn import_images(no_feh: bool, images: Vec<PathBuf>) -> AppResult<()> {
    let mut import_stats = ImportStats::default();
    let mut valid_images: Vec<ImageToImport> = Vec::new();
    let mut pdb = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    for image in &images {
        let image_name = image.to_str().unwrap_or("(can't display image name)");
        match pantsu_tags::check_image(&mut pdb, image) {
            Ok(img) => valid_images.push(img),
            Err(Error::ImageAlreadyExists(_)) => {
                import_stats.already_exists += 1;
                println!("{} - {}", "Image already exists       ", image_name);
            }
            Err(Error::ImageLoadError(_)) => {
                import_stats.could_not_open += 1;
                println!("{} - {}", "Failed to open image       ", image_name);
            }
            Err(error) => return Err(AppError::LibError(error)),
        }
    }

    let images_in_db = pdb.get_images_transaction().execute()?;
    let image_groups = pantsu_tags::image_similarity::group_similar_images(&valid_images, &images_in_db)
        .map_err(|e| AppError::LibError(e))?;

    let mut image_groups_with_similars: Vec<SimilarImagesGroup> = Vec::new();
    for group in image_groups {
        if group.is_single_image() {
            let image = group.new_images.into_iter().next().unwrap();
            pantsu_tags::import_image(&mut pdb, image).unwrap(); // todo: handle error
            import_stats.success += 1;
            let image_name = image.current_path.to_str().unwrap_or("(can't display image name)");
            println!("{} - {}", "Successfully imported image".green(), image_name);
        }
        else {
            for image in &group.new_images {
                let image_name = image.current_path.to_str().unwrap_or("(can't display image name)");
                println!("{} - {}", "Similar images exist       ".yellow(), image_name);
            }
            image_groups_with_similars.push(group);
        }
    }

    resolve_similar_image_groups(&mut pdb, image_groups_with_similars, &mut import_stats, no_feh)?;
    println!();
    import_stats.print_stats();
    Ok(())
}

fn resolve_similar_image_groups(pdb: &mut PantsuDB, similar_images_groups: Vec<SimilarImagesGroup>, stats: &mut ImportStats, no_feh: bool) -> AppResult<()> {
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
        println!("New images:");
        let new_images = &group.new_images;
        for (idx, new_img) in new_images.iter().enumerate() {
            let image_name = new_img.current_path.to_str().unwrap_or("(can't display image name)");
            println!("{} - {}", idx, image_name)
        }
        println!("\nImages already in PantsuTags:");
        for old_img in &group.old_images {
            let image_name = old_img.get_filename();
            println!(" - {}", image_name)
        }

        // todo: dispaly images in feh
        //let procs = feh_display_similar(image_name, &case.similar_images, use_feh);
        loop {
            println!("Enter the numbers of the new images that should be added to PantsuTags.");
            input.clear();
            stdin.read_line(&mut input).or_else(|e| Err(AppError::StdinReadError(e)))?;
            let input = input.trim();
            let input_numbers = input.split_whitespace()
                .map(|num| num.parse::<usize>())
                .collect::<Result<Vec<usize>, ParseIntError>>();

            if let Ok(numbers) = input_numbers { // todo: no input adds no images, maybe make clearer?
                let num_new_images = new_images.len();
                if numbers.iter().all(|&num| num < num_new_images) {
                    for (idx, new_image) in new_images.iter().enumerate() {
                        let image_name = new_image.current_path.to_str().unwrap_or("(can't display image name)");
                        if numbers.iter().any(|&num| idx == num) {
                            match pantsu_tags::import_image(pdb, new_image) {
                                Ok(_) => {
                                    stats.similar_imported += 1;
                                    println!("Imported new image {}: {}", idx, image_name)
                                }
                                Err(e) => {
                                    eprintln!("Failed to import new image {}, Error: {}", image_name, e);
                                }
                            }
                        }
                        else {
                            stats.similar_not_imported += 1;
                            println!("New image {} {} imported", image_name, "was not".bold());
                        }
                    }

                    break;
                } else {
                    println!("Invalid input: the highest image number is {}", num_new_images - 1);
                    continue;
                }
            }

            println!("Invalid input");
        }

        //procs.kill();
    }
    Ok(())
}

// todo: remove
/*
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
*/

// todo: remove or adapt
/*
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
*/

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