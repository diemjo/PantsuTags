use std::collections::HashSet;
use std::{io, iter};
use std::path::{PathBuf};
use colored::Colorize;
use log::{info, warn};
use pantsu_tags::{ImageHandle, PantsuTag, Sauce, SauceMatch, TmpFile};
use pantsu_tags::db::PantsuDB;
use crate::{AppError, CONFIGURATION, feh};
use crate::common::{AppResult, valid_filename_from_path};
use crate::feh::FehProcesses;

// sauce matches with a higher similarity will be automatically accepted
const FOUND_SIMILARITY_THRESHOLD: i32 = 90;
// sauce matches with a higher similarity are relevant. (Others will be discarded)
const RELEVANT_SIMILARITY_THESHOLD: i32 = 45;

pub fn auto_lookup_tags(image_paths: Vec<PathBuf>, sauce_existing: bool, sauce_not_existing: bool, sauce_not_checked: bool, no_feh: bool) -> AppResult<()> {
    let mut stats = AutoTaggingStats::default();
    let mut unsure_source_images: Vec<SauceUnsure> = Vec::new();
    let mut pdb = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    let images = get_images(&pdb, &image_paths, sauce_existing, sauce_not_existing, sauce_not_checked)?;
    for image in &images {
        let res = auto_add_tags_one_image(&mut pdb, image);
        match res {
            Ok(_) => {
                stats.success += 1;
                println!("{} - {}", "Successfully tagged image".green(), image.get_filename());
            }
            e@Err(AppError::LibError(_)) => {
                return e;
            }
            Err(AppError::NoRelevantSauces) => {
                stats.no_source += 1;
                println!("{} - {}", "No source found          ".red(), image.get_filename());
            },
            Err(AppError::SauceUnsure(image_handle, sauce_matches)) => {
                unsure_source_images.push(SauceUnsure {
                    image_handle,
                    matches: sauce_matches,
                });
                println!("{} - {}", "Source could be wrong    ".yellow(), image.get_filename());
            },
            Err(e) => {
                eprintln!("Unexpected error: {}", e);
                return Err(e);
            },
        }
    }

    resolve_sauce_unsure(&mut pdb, unsure_source_images, &mut stats, no_feh)?;
    println!();
    stats.print_stats();
    Ok(())
}

fn auto_add_tags_one_image(pdb: &mut PantsuDB, image: &ImageHandle) -> AppResult<()> {
    let sauces = pantsu_tags::get_image_sauces(CONFIGURATION.library_path.as_path(), &image)?;
    let relevant_sauces: Vec<SauceMatch> = sauces.into_iter().filter(|s| s.similarity > RELEVANT_SIMILARITY_THESHOLD).collect();
    match relevant_sauces.first() {
        Some(sauce) => {
            if sauce.similarity > FOUND_SIMILARITY_THRESHOLD {
                let tags = pantsu_tags::get_sauce_tags(sauce)?;
                pdb.update_images_transaction()
                    .for_image(image.get_filename())
                    .update_sauce(&Sauce::Match(sauce.link.clone()))
                    .add_tags(&tags)
                    .execute()?;
                info!("Set sauce '{}' to image: '{}'", sauce.link.clone(), image.get_filename());
                info!("Added tags {} to image: '{}'", PantsuTag::vec_to_string(&tags), image.get_filename());
            }
            else { // tags can be added in the sauce resolution
                return Err(AppError::SauceUnsure(image.clone(), relevant_sauces));
            }
        }
        None => { // mark in db that there are no sources for this image
            pdb.update_images_transaction()
                .for_image(image.get_filename())
                .update_sauce(&Sauce::NotExisting)
                .execute()?;
            warn!("Set sauce '{}' to image: '{}'", "NOT_EXISTING", image.get_filename());
            return Err(AppError::NoRelevantSauces);
        }
    }
    Ok(())
}

fn resolve_sauce_unsure(pdb: &mut PantsuDB, images_to_resolve: Vec<SauceUnsure>, stats: &mut AutoTaggingStats, no_feh: bool) -> AppResult<()>{
    if images_to_resolve.is_empty() {
        return Ok(());
    }
    let use_feh = !no_feh && feh::feh_available();
    let mut input = String::new();
    let stdin = io::stdin();
    let lib_path = CONFIGURATION.library_path.as_path();
    println!("\n\nResolving {} images with unsure sources manually:", images_to_resolve.len());
    for (image_idx, image) in images_to_resolve.iter().enumerate() {
        let image_path = image.image_handle.get_path(lib_path);
        let mut thumbnails = ThumbnailDisplayer::new(use_feh);
        println!("\nImage {} of {}:\n{}\n", image_idx+1, images_to_resolve.len(), image_path);
        thumbnails.set_thumbnails(&image.matches);
        thumbnails.feh_display(&image_path);
        for (index, sauce) in image.matches.iter().enumerate() {
            println!("{} - {}", index+1, sauce.link);
        }
        loop {
            println!("If one of the sources is correct, select the corresponding image.");
            println!("Enter 'n' if there is no match, or 's' to skip all remaining images.");
            input.clear();
            stdin.read_line(&mut input).or_else(|e| Err(AppError::StdinReadError(e)))?;
            let input = input.trim();
            if let Ok(num) = input.parse::<usize>() {
                if num == 0 || num > image.matches.len() {
                    println!("Invalid image number, must be in range 1 to {}", image.matches.len());
                    continue;
                }
                let correct_sauce = &image.matches[num-1];
                let tags = pantsu_tags::get_sauce_tags(correct_sauce)?;
                pdb.update_images_transaction()
                    .for_image(image.image_handle.get_filename())
                    .update_sauce(&Sauce::Match(correct_sauce.link.clone()))
                    .add_tags(&tags)
                    .execute()?;
                stats.success += 1;
                println!("{}", "Successfully added tags to image".green());
                info!("Added tags {} to image: '{}'", PantsuTag::vec_to_string(&tags), image.image_handle.get_filename());
                break;
            }
            if input.eq("n") {
                pdb.update_images_transaction()
                    .for_image(image.image_handle.get_filename())
                    .update_sauce(&Sauce::NotExisting)
                    .execute()?;
                stats.no_source += 1;
                println!("No tags added");
                warn!("Set sauce '{}' to image: '{}'", "NOT_EXISTING", image.image_handle.get_filename());
                break;
            }
            else if input.eq("s") {
                println!("Skip remaining images");
                thumbnails.kill_feh();
                return Ok(());
            }
            println!("Invalid input");
        }
        thumbnails.kill_feh();
    }
    Ok(())
}

fn get_images(pdb: &PantsuDB, image_paths: &Vec<PathBuf>, sauce_existing: bool, sauce_not_existing: bool, sauce_not_checked: bool) -> AppResult<HashSet<ImageHandle>> {
    let mut images: HashSet<ImageHandle> = if sauce_existing {
        pdb.get_images_transaction().with_existing_sauce().execute()?
    } else if sauce_not_existing {
        pdb.get_images_transaction().with_not_existing_sauce().execute()?
    } else if sauce_not_checked {
        pdb.get_images_transaction().with_not_checked_sauce().execute()?
    } else {
        Vec::new()
    }.into_iter().collect();
    for image_path in image_paths {
        let image_name = valid_filename_from_path(image_path)?;
        let image = pdb.get_image_transaction(&image_name).execute()?.ok_or_else(|| AppError::ImageNotFound(image_name))?;
        images.insert(image);
    }
    Ok(images)
}

#[derive(Default)]
struct AutoTaggingStats {
    success: u64,
    no_source: u64,
}
impl AutoTaggingStats {
    fn print_stats(&self) {
        if self.success > 0 {
            println!("Successfully tagged: {}", self.success);
        }
        if self.no_source > 0 {
            println!("Source not found:    {}", self.no_source);
        }
    }
}

struct SauceUnsure {
    pub image_handle: ImageHandle,
    pub matches: Vec<SauceMatch>,
}

struct ThumbnailDisplayer {
    thumbnails: Vec<TmpFile>,
    enabled: bool,
    feh_processes: Option<FehProcesses>,
}
impl ThumbnailDisplayer {
    fn new(enable: bool) -> Self {
        ThumbnailDisplayer {
            thumbnails: Vec::new(),
            enabled: enable,
            feh_processes: None,
        }
    }

    fn set_thumbnails(&mut self, sauces: &Vec<SauceMatch>) {
        if !self.enabled {
            return;
        }
        match pantsu_tags::get_thumbnails(&sauces) {
            Ok(paths) => self.thumbnails = paths,
            Err(_) => {
                // todo: log warning?
                self.enabled = false;
            }
        }
    }

    fn feh_display(&mut self, image_path: &str) {
        if !self.enabled {
            return;
        }
        let paths = self.thumbnails.iter()
            .map(|p| p.get_path().to_str())
            .collect::<Option<Vec<&str>>>();
        let paths = match paths {
            Some(p) => p,
            None => return,
        };
        let mut procs = self.feh_processes.take().unwrap_or(FehProcesses::new_empty());
        procs = feh::feh_display_images(iter::once(image_path), "Local image", procs);
        self.feh_processes = Some(feh::feh_display_images(paths.into_iter(), "Potential source", procs));
    }

    fn kill_feh(&mut self) {
        let procs = self.feh_processes.take();
        if let Some(procs) = procs {
            procs.kill();
        }

    }
}