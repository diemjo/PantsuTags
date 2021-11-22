use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use colored::Colorize;
use structopt::StructOpt;
use pantsu_tags::db::PantsuDB;
use pantsu_tags::{Error, get_thumbnail_link};
use pantsu_tags::SauceMatch;
use crate::cli::Args;
use crate::common::AppError;
use crate::common::AppResult;
mod common;
mod cli;
mod get;

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
            get::get(include_tags, exclude_tags, temp_dir)
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
            Err(AppError::SauceUnsure(sauce_matches)) => {
                import_stats.source_unsure += 1;
                unsure_source_images.push(SauceUnsure {
                    path: &image,
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

    resolve_sauce_unsure(&mut pdb, &unsure_source_images)?;
    Ok(())
}

fn import_one_image_auto_source(pdb: &mut PantsuDB, image: &PathBuf) -> Result<(), AppError> {
    let image_handle = pantsu_tags::new_image_handle(pdb, &image, true)?;
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
    Ok(())
}

fn import_one_image(pdb: &mut PantsuDB, image: &Path) -> AppResult<()> {
    let image_handle = pantsu_tags::new_image_handle(pdb, &image, true)?;
    pantsu_tags::store_image_with_tags(pdb, &image_handle, &Vec::new()).map_err(|e| AppError::LibError(e))
}

fn resolve_sauce_unsure(pdb: &mut PantsuDB, images_to_resolve: &Vec<SauceUnsure>) -> AppResult<()>{
    if images_to_resolve.is_empty() {
        return Ok(());
    }
    let mut input = String::new();
    let stdin = io::stdin();
    println!("\n\nResolving {} images with unsure sources manually:", images_to_resolve.len());
    for image in images_to_resolve {
        let image_name = image.path.to_str().unwrap_or("(can't display image name)");
        let mut sauce_thumbnails: Vec<String> = Vec::new();
        println!("\nImage {}:\n", image_name);
        for (index, sauce) in image.matches.iter().enumerate() {
            sauce_thumbnails.push(get_thumbnail_link(sauce)?);
            println!("{} - {}", index, sauce.link);
        }
        let links = sauce_thumbnails.iter().map(|s| s.as_str()).collect();
        let feh_commands = feh_compare_image(
            image_name,
            &links,
            "Original",
            "Potential Source"
        );
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
                let image_handle = pantsu_tags::new_image_handle(pdb, &image.path, false)?; // at this point, if there is a similar image it's approved by the user
                pantsu_tags::store_image_with_tags_from_sauce(pdb, &image_handle, correct_sauce, &tags)?;
                println!("{}", "Successfully added tags to image".green());
                break;
            }
            if input.eq("n") {
                // todo: mark in db that image has no source
                println!("No tags added");
                break;
            }
            else if input.eq("s") {
                println!("Skip remaining images");
                feh_commands.map(|mut cmd| {
                    let _ = cmd.0.kill();
                    let _ = cmd.1.kill();
                });
                return Ok(());
            }
            println!("Invalid input");
        }
        feh_commands.map(|mut cmd| {
            let _ = cmd.0.kill();
            let _ = cmd.1.kill();
        });
    }
    Ok(())
}

//feh --info 'echo "%u"' https://img3.gelbooru.com/images/bb/62/bb626c2a621cbc1642256c0ebefbd219.jpg https://img3.gelbooru.com/images/12/ee/12ee1ac61779f5ccfcc383485c7c3191.png
//zero indexed:
//feh --info 'echo "$((%u -1))"' https://img3.gelbooru.com/images/bb/62/bb626c2a621cbc1642256c0ebefbd219.jpg https://img3.gelbooru.com/images/12/ee/12ee1ac61779f5ccfcc383485c7c3191.png

fn feh_compare_image(image: &str, other_images: &Vec<&str>, image_label: &str, other_images_label: &str) -> Option<(Child, Child)> {
    let command_image = format!("feh -q.ZB black --info \'echo \"{}\"\' \'{}\'", image_label, image);

    let mut command_other_images = format!("feh -q.ZB black --info \'echo \"$((%u -1)) {}\"\'", other_images_label);
    for &image in other_images {
        command_other_images.push_str(" \'");
        command_other_images.push_str(image);
        command_other_images.push('\'');
    }

    let cmd_image = Command::new("sh")
        .arg("-c")
        .arg(command_image)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    let mut cmd_image = match cmd_image {
        Ok(cmd) => cmd,
        Err(_) => return None,
    };

    let cmd_other_images = Command::new("sh")
        .arg("-c")
        .arg(command_other_images)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    let cmd_other_images = match cmd_other_images {
        Ok(cmd) => cmd,
        Err(_) => {
            let _ = cmd_image.kill();
            return None;
        }
    };
    Some((cmd_image, cmd_other_images))
}

struct SauceUnsure<'a> {
    pub path: &'a Path,
    pub matches: Vec<SauceMatch>,
}