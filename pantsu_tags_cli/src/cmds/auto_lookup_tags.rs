use std::collections::HashSet;
use std::{io, iter};
use std::path::{PathBuf, Path};
use colored::Colorize;
use futures::{stream, StreamExt, TryStreamExt};
use log::{info, warn};
use pantsu_tags::{ImageHandle, PantsuTag, Sauce, SauceMatch, TmpFile};
use pantsu_tags::db::PantsuDB;
use tokio::sync::mpsc::{Receiver, self};
use tokio::task;
use crate::{AppError, CONFIGURATION, feh, common};
use crate::common::{AppResult, valid_filename_from_path};
use crate::feh::FehProcesses;

// sauce matches with a higher similarity will be automatically accepted
const FOUND_SIMILARITY_THRESHOLD: i32 = 90;
// sauce matches with a higher similarity are relevant. (Others will be discarded)
const RELEVANT_SIMILARITY_THESHOLD: i32 = 45;

const MAX_CONCURRENT_REQUESTS: usize = 16;
const MAX_PREFETCH_SOURCE_RESOLUTION: usize = 4;

pub fn auto_lookup_tags(image_paths: Vec<PathBuf>, sauce_existing: bool, sauce_not_existing: bool, sauce_not_checked: bool, no_feh: bool) -> AppResult<()> {
    let pdb = PantsuDB::new(CONFIGURATION.database_path.as_path())?;
    let images = get_images(&pdb, &image_paths, sauce_existing, sauce_not_existing, sauce_not_checked)?;
    
    let rt = tokio::runtime::Runtime::new()
        .or_else(|e| Err(pantsu_tags::Error::TokioInitError(e)))?;
    let (pdb, stats, unsure_source_images) = rt.block_on(auto_lookup_tags_async(pdb, images))?;

    let stats = rt.block_on(resolve_sauce_unsure(pdb, unsure_source_images, stats, no_feh))?;
    println!();
    stats.print_stats();
    Ok(())
}


async fn auto_lookup_tags_async(pdb: PantsuDB, images: HashSet<ImageHandle>) -> AppResult<(PantsuDB,AutoTaggingStats,Vec<SauceUnsure>)> {
    let tagging_stats = AutoTaggingStats::default();
    let unsure_sauces: Vec<SauceUnsure> = Vec::new();

    let res = stream::iter(images)
        .map(|image| async move {
            let sauces = pantsu_tags::get_image_sauces(&CONFIGURATION.library_path, &image).await?;
            let judgement = judge_sauce(&image, sauces).await?;
            Ok((image,judgement))
        })
        .buffer_unordered(MAX_CONCURRENT_REQUESTS)
        .try_fold((pdb,tagging_stats,unsure_sauces), |(mut pdb, mut stats, mut unsures), (image,judgement)| async move {
            store_sauce_in_db(&mut pdb, &image, &judgement).await?;
            let image_name = image.get_filename();
            match judgement {
                SauceJudgement::Matching { sauce: _, tags: _ } => {
                    stats.success += 1;
                    println!("{} - {}", "Successfully tagged image".green(), image_name);
                }
                SauceJudgement::Unsure(unsure) => {
                    unsures.push(unsure);
                    println!("{} - {}", "Source could be wrong    ".yellow(), image_name);
                }
                SauceJudgement::NotExisting => {
                    stats.no_source += 1;
                    println!("{} - {}", "No source found          ".red(), image_name);
                }
            }
            Ok((pdb,stats,unsures))
        }).await;

    res
}

async fn judge_sauce(image: &ImageHandle, sauces: Vec<SauceMatch>) -> AppResult<SauceJudgement> {
    let relevant_sauces: Vec<SauceMatch> = sauces.into_iter().filter(|s| s.similarity > RELEVANT_SIMILARITY_THESHOLD).collect();
    match relevant_sauces.first() {
        Some(sauce) => {
            if sauce.similarity > FOUND_SIMILARITY_THRESHOLD {
                let tags = pantsu_tags::get_sauce_tags(sauce).await?;
                let sauce = relevant_sauces.into_iter().next().unwrap();
                Ok(SauceJudgement::Matching { sauce, tags })
            }
            else {
                Ok(SauceJudgement::Unsure(SauceUnsure { image_handle: image.clone(), matches: relevant_sauces }))
            }
        }
        None => Ok(SauceJudgement::NotExisting),
    }
}

async fn store_sauce_in_db(pdb: &mut PantsuDB, image: &ImageHandle, sauce_judgement: &SauceJudgement) -> AppResult<()> {
    match sauce_judgement {
        SauceJudgement::Matching { sauce, tags } => {
            pdb.update_images_transaction()
                .for_image(image.get_filename())
                .update_sauce(&Sauce::Match(pantsu_tags::url_from_str(&sauce.link)?))
                .add_tags(&tags)
                .execute()?;
            info!("Set sauce '{}' to image: '{}'", sauce.link.clone(), image.get_filename());
            info!("Added tags {} to image: '{}'", PantsuTag::vec_to_string(&tags), image.get_filename());
        },
        SauceJudgement::NotExisting => { // mark in db that there are no sources for this image
            pdb.update_images_transaction()
                .for_image(image.get_filename())
                .update_sauce(&Sauce::NotExisting)
                .execute()?;
            warn!("Set sauce '{}' to image: '{}'", "NOT_EXISTING", image.get_filename());
        },
        SauceJudgement::Unsure(_) => {}, // tags can be added in the sauce resolution
    }
    Ok(())
}


async fn resolve_sauce_unsure(pdb: PantsuDB, images_to_resolve: Vec<SauceUnsure>, stats: AutoTaggingStats, no_feh: bool) -> AppResult<AutoTaggingStats> {
    if images_to_resolve.is_empty() {
        return Ok(stats);
    }
    let use_feh = !no_feh && feh::feh_available();
    let num_images_to_resolve = images_to_resolve.len();
    let (tx,rx) = mpsc::channel(1);
    let resolver_thread = task::spawn_blocking(move || { resolve_sauce_thread(pdb, rx, num_images_to_resolve, stats, use_feh)} );

    println!("\n\nResolving {} images with unsure sources manually:", num_images_to_resolve);
    let _ = stream::iter(images_to_resolve)
        .map(|image| {
            let tx = tx.clone();
            async move {
                let thumbnails = pantsu_tags::get_thumbnails(&image.matches).await;
                tx.send((image, thumbnails)).await
                    .or(Err(AppError::TaskCommunicationError))
            }
        })
        .buffer_unordered(MAX_PREFETCH_SOURCE_RESOLUTION)
        .try_for_each(|_| async move { Ok(()) }).await;

    drop(tx);
    let stats = resolver_thread.await??;

    Ok(stats)
}

type ResolveRequest = (SauceUnsure, pantsu_tags::Result<Vec<TmpFile>>);

fn resolve_sauce_thread(mut pdb: PantsuDB, mut rx: Receiver<ResolveRequest>, num_images_to_resolve: usize, mut stats: AutoTaggingStats, use_feh: bool) -> AppResult<AutoTaggingStats> {
    let rt = tokio::runtime::Runtime::new()
        .or_else(|e| Err(pantsu_tags::Error::TokioInitError(e)))?;
    let mut thumb_displayer = ThumbnailDisplayer::new(use_feh);
    let mut input = String::new();
    let stdin = io::stdin();
    let lib_path = CONFIGURATION.library_path.as_path();
    let mut image_idx = 0;

    while let Some((image, thumbnails)) = rx.blocking_recv() {
        let image_path = image.image_handle.get_path(lib_path);
        assert!(image_idx < num_images_to_resolve);
        image_idx += 1;
        println!("\nImage {} of {}:\n{}\n", image_idx, num_images_to_resolve, common::get_path(&image_path));
        thumb_displayer.set_thumbnails(thumbnails);
        thumb_displayer.feh_display(&image_path);
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
                let tags = rt.block_on(pantsu_tags::get_sauce_tags(correct_sauce))?;
                pdb.update_images_transaction()
                    .for_image(image.image_handle.get_filename())
                    .update_sauce(&Sauce::Match(pantsu_tags::url_from_str(&correct_sauce.link)?))
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
                thumb_displayer.kill_feh();

                // todo: adjust stats (include skipped images)
                return Ok(stats);
            }
            println!("Invalid input");
        }
        thumb_displayer.kill_feh();
    }

    Ok(stats)
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

enum SauceJudgement {
    Matching { sauce: SauceMatch, tags: Vec<PantsuTag> },
    Unsure (SauceUnsure),
    NotExisting,
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

    fn set_thumbnails(&mut self, thumbnails: pantsu_tags::Result<Vec<TmpFile>>) {
        if !self.enabled {
            return;
        }
        match thumbnails {
            Ok(paths) => self.thumbnails = paths,
            Err(_) => self.disable("Unable to download source thumbnails"),
        }
    }

    fn feh_display(&mut self, image_path: &Path) {
        if !self.enabled {
            return;
        }
        let image_path_str = match common::try_get_path(image_path) {
            Ok(path) => path,
            Err(_) => { self.disable("Image path is invalid"); return; },
        };

        let paths = self.thumbnails.iter()
            .map(|p| p.get_path().to_str())
            .collect::<Option<Vec<&str>>>();
        let paths = match paths {
            Some(p) => p,
            None => { self.disable("Source thumbnail path is invalid"); return; },
        };
        let mut procs = self.feh_processes.take().unwrap_or(FehProcesses::new_empty());
        procs = feh::feh_display_images(iter::once(image_path_str.as_str()), "Local image", procs);
        self.feh_processes = Some(feh::feh_display_images(paths.into_iter(), "Potential source", procs));
    }

    fn disable(&mut self, msg: &str) {
        self.enabled = false;
        warn!("Disable feh: {}", msg);
    }

    fn kill_feh(&mut self) {
        let procs = self.feh_processes.take();
        if let Some(procs) = procs {
            procs.kill();
        }

    }
}