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

lazy_static! {
    pub static ref CONFIGURATION: AppConfig = AppConfig::load_config();
}

fn main() -> Result<(), AppError> {
    let args = Args::parse();
    //println!("Got arguments {:?}", args);
    let res: Result<(), AppError> = match args {
        Args::ImportImages(args) => {
            cmds::import_images(args.no_feh, args.images)
        },
        Args::RemoveImages(args) => {
            cmds::remove_images(args.images)
        },
        Args::AddTags(args) => {
            cmds::add_tags(args.tags, args.images)
        },
        Args::RemoveTags(args) => {
            cmds::remove_tags(args.tags, args.images)
        },
        Args::ListTags(args) => {
            cmds::list_tags(args.images, args.tag_types, args.print_tagnames)
        },
        Args::ImageInfos(args) => {
            cmds::image_infos(args.images)
        },
        Args::ListImages(args) => {
            cmds::list_images(&args.include_tags, &args.exclude_tags, match (args.aspect_ratio_min, args.aspect_ratio_max) {
                (Some(min), Some(max)) => AspectRatio::Range(min, max),
                (Some(min), None) => AspectRatio::Min(min),
                (None, Some(max)) => AspectRatio::Max(max),
                (None, None) => AspectRatio::Any,
            }, args.print_filenames, args.sauce_existing, args.sauce_not_existing, args.sauce_not_checked, None)
        },
        Args::AutoLookupTags(args) => {
            cmds::auto_lookup_tags(args.images, args.sauce_existing, args.sauce_not_existing, args.sauce_not_checked, args.no_feh)
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