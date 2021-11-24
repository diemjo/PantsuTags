use std::{io, thread};
use std::io::{BufRead, BufReader, Stdin};
use std::path::{Path, PathBuf};
use std::process::ChildStdout;
use std::sync::{Arc, mpsc};
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use colored::Colorize;
use pantsu_tags::db::PantsuDB;
use pantsu_tags::{Error, get_thumbnail_link, ImageHandle, Sauce, SauceMatch};
use crate::common::{AppError, AppResult};
use crate::feh;
use crate::feh::FehProcesses;
use crate::stdin_thread::StdinThread;

// sauce matches with a higher similarity will be automatically accepted
const FOUND_SIMILARITY_THRESHOLD: i32 = 90;
// sauce matches with a higher similarity are relevant. (Others will be discarded)
const RELEVANT_SIMILARITY_THESHOLD: i32 = 45;

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

    resolve_sauce_unsure(&mut pdb, unsure_source_images, no_feh)?;
    Ok(())
}

fn import_one_image_auto_source(pdb: &mut PantsuDB, image: &PathBuf) -> AppResult<()> {
    let image_handle = pantsu_tags::new_image_handle(pdb, &image, true)?;
    let sauces = pantsu_tags::get_image_sauces(&image_handle)?;
    let relevant_sauces: Vec<SauceMatch> = sauces.into_iter().filter(|s| s.similarity > RELEVANT_SIMILARITY_THESHOLD).collect();
    match relevant_sauces.first() {
        Some(sauce) => {
            if sauce.similarity > FOUND_SIMILARITY_THRESHOLD {
                let tags = pantsu_tags::get_sauce_tags(sauce)?;
                pdb.update_file_sauce_with_tags(image_handle, Sauce::Match(sauce.link.clone()), &tags)?;
            }
            else { // store image without tags for now, tags can be added in the sauce resolution
                return Err(AppError::SauceUnsure(image_handle, relevant_sauces));
            }
        }
        None => { // store image without tags
            pdb.update_file_source(image_handle, Sauce::NonExistent)?;
            return Err(AppError::NoRelevantSauces);
        }
    }
    Ok(())
}

fn import_one_image(pdb: &mut PantsuDB, image: &Path) -> AppResult<()> {
    pantsu_tags::new_image_handle(pdb, &image, true)?;
    Ok(())
}

fn resolve_sauce_unsure(pdb: &mut PantsuDB, images_to_resolve: Vec<SauceUnsure>, no_feh: bool) -> AppResult<()>{
    if images_to_resolve.is_empty() {
        return Ok(());
    }
    let use_feh = !no_feh && feh::feh_available();
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

            //stdin.read_line(&mut input).or_else(|e| Err(AppError::StdinReadError(e)))?;
            let input = read_input(&mut thumbnails)?;
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

fn read_input(thumbnails: &mut ThumbnailDisplayer) -> AppResult<String>{
    let stdin = io::stdin();
    let feh_reader = thumbnails.take_feh_reader();
    if let None = feh_reader {
        let mut input = String::new();
        stdin.read_line(&mut input).or_else(|e| Err(AppError::StdinReadError(e)))?;
        return Ok(input)
    }
    let mut feh_reader = feh_reader.unwrap();
    let mut input = String::new();
    let res = match feh_reader.read_line(&mut input) {
        Ok(_) => {
            println!("got: {}", &input);
            Ok(input)
        },
        Err(e) => Err(AppError::StdinReadError(e)),
    };
    thumbnails.give_feh_reader(feh_reader);

    res
    /*let mut feh_reader1 = Arc::new(feh_reader.unwrap());
    let mut feh_reader2 = feh_reader1.clone();

    let (tx1, rx) = mpsc::channel();
    let tx2 = tx1.clone();
    let thread1 = thread::spawn(move || {
        let mut input = String::new();
        let res = match stdin.read_line(&mut input) {
            Ok(_) => Ok(input),
            Err(e) => Err(AppError::StdinReadError(e)),
        };
        let _ = tx1.send(res);
    });
    let thread2 = thread::spawn(move || {
        let mut input = String::new();
        let res = match feh_reader2.read_line(&mut input) {
            Ok(_) => Ok(input),
            Err(e) => Err(AppError::StdinReadError(e)),
        };
        let _ = tx2.send(res);
    });

    let res = rx.recv().map_err(|_| AppError::NoRelevantSauces); // todo: replace error
    thread1.*/
}

struct SauceUnsure<'a> {
    pub path: &'a Path,
    pub image_handle: ImageHandle,
    pub matches: Vec<SauceMatch>,
}

struct ThumbnailDisplayer<'a> {
    thumbnail_links: Vec<String>,
    enabled: bool,
    feh_processes: Option<FehProcesses>,
    input_threads_rx: Option<&'a Receiver<AppResult<String>>>,
}
impl <'a> ThumbnailDisplayer {
    fn new(enable: bool) -> Self {
        ThumbnailDisplayer {
            thumbnail_links: Vec::new(),
            enabled: enable,
            feh_processes: None,
            input_threads_rx: None,
        }
    }

    fn add_thumbnail_link(&mut self, sauce: &SauceMatch) {
        if !self.enabled {
            return;
        }
        let link = match get_thumbnail_link(sauce) {
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

    fn read_input(&mut self, stdin: &mut StdinThread) -> AppResult<String> {
        if let None = self.feh_processes {
            stdin.read_line()
        }

        if self.input_threads_rx.is_none() {
            self.launch_feh_input_thread(stdin);
        }

        self.input_threads_rx.recv().map_err(|_| AppError::NoRelevantSauces) // todo: replace error
    }

    fn launch_feh_input_thread(&mut self, stdin: &mut StdinThread) -> &Receiver<AppResult<String>> {
        let (tx, rx) = stdin.get_tx_rx_ref();
        let feh_reader = self.take_feh_reader();
        if let None = feh_reader {
            return rx
        }
        let feh_reader = feh_reader.unwrap();
        let thread = thread::spawn(move || {
            loop {
                let mut input = String::new();
                let res = match feh_reader.read_line(&mut input) {
                    Ok(_) => Ok(input),
                    Err(e) => Err(AppError::StdinReadError(e)),
                };
                let _ = tx.send(res);
            }
        });

        rx
    }

    fn take_feh_reader(&mut self) -> Option<BufReader<ChildStdout>> {
        if let Some(procs) = &mut self.feh_processes {
            return procs.take_reader();
        }
        None
    }

    fn give_feh_reader(&mut self, reader: BufReader<ChildStdout>) {
        if let Some(procs) = &mut self.feh_processes {
            procs.give_reader(reader)
        }
    }

    fn kill_feh(&mut self) {
        let procs = self.feh_processes.take();
        if let Some(procs) = procs {
            procs.kill();
        }

    }
}