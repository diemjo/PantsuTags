use clap::{Parser};
use pantsu_tags::db::AspectRatio;
use crate::cli::{Args};
use crate::common::AppError;

mod common;
mod cli;
mod get;
mod import;
mod tag;
mod feh;
mod autotags;


fn main() -> Result<(), AppError> {
    let args = Args::parse();
    //println!("Got arguments {:?}", args);
    let res: Result<(), AppError> = match args {
        Args::ImportImages(args) => {
            import::import(args.no_feh, args.images)?;
            Ok(())
        },
        Args::RemoveImages(_) => {
            Err(AppError::NoRelevantSauces)
        },
        Args::AddTags(args) => {
            tag::tag_add(args.tags, args.images.into_iter().map(|i|i.to_string_lossy().to_string()).collect())?;
            Ok(())
        },
        Args::RemoveTags(args) => {
            tag::tag_rm(args.tags, args.images.into_iter().map(|i|i.to_string_lossy().to_string()).collect())?;
            Ok(())
        },
        Args::ListTags(args) => {
            if args.images.len()==0 {
                tag::tag_list(args.tag_types)?;
            } else {
                tag::tag_get(args.images.into_iter().map(|i|i.to_string_lossy().to_string()).collect(), args.tag_types)?;
            }
            Ok(())
        },
        Args::ImageInfos(_) => {
            Err(AppError::NoRelevantSauces)
        },
        Args::ListImages(args) => {
            get::get(&args.include_tags, &args.exclude_tags, match (args.aspect_ratio_min, args.aspect_ratio_max) {
                (Some(min), Some(max)) => AspectRatio::Range(min, max),
                (Some(min), None) => AspectRatio::Min(min),
                (None, Some(max)) => AspectRatio::Max(max),
                (None, None) => AspectRatio::Any,
            }, args.sauce_existing, args.sauce_not_existing, args.sauce_not_checked, None)?;
            Ok(())
        },
        Args::AutoLookupTags(args) => {
            autotags::auto_add_tags(args.images, args.no_feh)?;
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