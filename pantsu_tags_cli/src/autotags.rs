use std::io;
use std::path::Path;
use colored::Colorize;
use pantsu_tags::{ImageHandle, Sauce, SauceMatch};
use pantsu_tags::db::PantsuDB;
use crate::{AppError, feh};
use crate::common::AppResult;
use crate::feh::FehProcesses;

// sauce matches with a higher similarity will be automatically accepted
const FOUND_SIMILARITY_THRESHOLD: i32 = 90;
// sauce matches with a higher similarity are relevant. (Others will be discarded)
const RELEVANT_SIMILARITY_THESHOLD: i32 = 45;

pub fn auto_add_tags(pdb: &mut PantsuDB, image_handle: ImageHandle) -> AppResult<()> {
    let sauces = pantsu_tags::get_image_sauces(&image_handle)?;
    let relevant_sauces: Vec<SauceMatch> = sauces.into_iter().filter(|s| s.similarity > RELEVANT_SIMILARITY_THESHOLD).collect();
    match relevant_sauces.first() {
        Some(sauce) => {
            if sauce.similarity > FOUND_SIMILARITY_THRESHOLD {
                let tags = pantsu_tags::get_sauce_tags(sauce)?;
                pdb.update_file_sauce_with_tags(image_handle, Sauce::Match(sauce.link.clone()), &tags)?;
            }
            else { // tags can be added in the sauce resolution
                return Err(AppError::SauceUnsure(image_handle, relevant_sauces));
            }
        }
        None => { // mark in db that there are no sources for this image
            pdb.update_file_source(image_handle, Sauce::NonExistent)?;
            return Err(AppError::NoRelevantSauces);
        }
    }
    Ok(())
}

pub fn resolve_sauce_unsure(pdb: &mut PantsuDB, images_to_resolve: Vec<SauceUnsure>, no_feh: bool) -> AppResult<()>{
    if images_to_resolve.is_empty() {
        return Ok(());
    }
    let use_feh = !no_feh && feh::feh_available();
    let mut input = String::new();
    let stdin = io::stdin();
    println!("\n\nResolving {} images with unsure sources manually:", images_to_resolve.len());
    for image in images_to_resolve {
        let image_name = image.path.to_str().unwrap_or("(can't display image name)");
        let mut thumbnails = ThumbnailDisplayer::new(use_feh);
        println!("\nImage {}:\n", image_name);
        for (index, sauce) in image.matches.iter().enumerate() {
            thumbnails.add_thumbnail_link(sauce);
            println!("{} - {}", index, sauce.link);
        }
        thumbnails.feh_display(image_name);
        loop {
            println!("If one of the sources is correct, enter the corresponding number.");
            println!("Enter 'n' if there is no match, or 's' to skip all remaining images.");
            input.clear();
            stdin.read_line(&mut input).or_else(|e| Err(AppError::StdinReadError(e)))?;
            let input = input.trim();
            if let Ok(num) = input.parse::<usize>() {
                if num >= image.matches.len() {
                    println!("Number too big, the last source has number {}", image.matches.len()-1);
                    continue;
                }
                let correct_sauce = &image.matches[num];
                let tags = pantsu_tags::get_sauce_tags(correct_sauce)?;
                pdb.update_file_sauce_with_tags(image.image_handle, Sauce::Match(correct_sauce.link.clone()), &tags)?;
                println!("{}", "Successfully added tags to image".green());
                break;
            }
            if input.eq("n") {
                pdb.update_file_source(image.image_handle, Sauce::NonExistent)?;
                println!("No tags added");
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

pub struct SauceUnsure<'a> {
    pub path: &'a Path,
    pub image_handle: ImageHandle,
    pub matches: Vec<SauceMatch>,
}

struct ThumbnailDisplayer {
    thumbnail_links: Vec<String>,
    enabled: bool,
    feh_processes: Option<FehProcesses>,
}
impl ThumbnailDisplayer {
    fn new(enable: bool) -> Self {
        ThumbnailDisplayer {
            thumbnail_links: Vec::new(),
            enabled: enable,
            feh_processes: None,
        }
    }

    fn add_thumbnail_link(&mut self, sauce: &SauceMatch) {
        if !self.enabled {
            return;
        }
        let link = match pantsu_tags::get_thumbnail_link(sauce) {
            Ok(link) => link,
            Err(_) => {
                self.enabled = false; // if left enabled without adding a thumbnail, the indices will be wrong.
                return;
            },
        };
        self.thumbnail_links.push(link);
    }

    fn feh_display(&mut self, image: &str) {
        if !self.enabled {
            return;
        }
        let links = self.thumbnail_links.iter().map(|s| s.as_str()).collect();
        self.feh_processes = Some(feh::feh_compare_image(
            image,
            &links,
            "Original",
            "Potential Source"
        ));
    }

    fn kill_feh(&mut self) {
        let procs = self.feh_processes.take();
        if let Some(procs) = procs {
            procs.kill();
        }

    }
}