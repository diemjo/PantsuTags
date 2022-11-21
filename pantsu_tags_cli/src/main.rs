use ::log::{error, info, LevelFilter};
use clap::Parser;
use lazy_static::lazy_static;

use pantsu_tags::db::AspectRatio;

use crate::cli::{Args};
use crate::common::AppError;
use crate::config::AppConfig;

mod common;
mod cli;
mod feh;
mod cmds;
mod config;
mod log;

lazy_static! {
    pub static ref CONFIGURATION: AppConfig = AppConfig::load_config();
}

fn main() -> Result<(), AppError> {
    log4rs::init_config(log::log_config(LevelFilter::Info)).unwrap();
    let args = Args::parse();
    //println!("Got arguments {:?}", args);
    let res: Result<(), AppError> = match args {
        Args::ImportImages(args) => {
            info!("Running command 'import-images'");
            cmds::import_images(args.no_feh, args.images, args.always_copy_images)
        },
        Args::RemoveImages(args) => {
            info!("Running command 'remove-images'");
            cmds::remove_images(args.images)
        },
        Args::AddTags(args) => {
            info!("Running command 'add-tags'");
            cmds::add_tags(args.tags, args.images)
        },
        Args::RemoveTags(args) => {
            info!("Running command 'remove-tags'");
            cmds::remove_tags(args.tags, args.images)
        },
        Args::ListTags(args) => {
            info!("Running command 'list-tags'");
            cmds::list_tags(args.images, args.tag_types, args.print_tagnames)
        },
        Args::ImageInfos(args) => {
            info!("Running command 'image-infos'");
            cmds::image_infos(args.images)
        },
        Args::ListImages(args) => {
            info!("Running command 'list-images'");
            cmds::list_images(&args.include_tags, &args.exclude_tags, match (args.aspect_ratio_min, args.aspect_ratio_max) {
                (Some(min), Some(max)) => AspectRatio::Range(min, max),
                (Some(min), None) => AspectRatio::Min(min),
                (None, Some(max)) => AspectRatio::Max(max),
                (None, None) => AspectRatio::Any,
            }, args.print_filenames, args.sauce_existing, args.sauce_not_existing, args.sauce_not_checked, None)
        },
        Args::AutoLookupTags(args) => {
            info!("Running command 'auto-lookup-tags'");
            cmds::auto_lookup_tags(args.images, args.sauce_existing, args.sauce_not_existing, args.sauce_not_checked, args.no_feh)
        },
        Args::ImportTags(args) => {
            info!("Running command 'import-tags'");
            cmds::import_tags(&args.file)
        },
        Args::ExportTags(args) => {
            info!("Running command 'export-tags'");
            cmds::export_tags(&args.file)
        }
    };

    match res {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            error!("Error: {}", e);
            Err(e)
        },
    }
}