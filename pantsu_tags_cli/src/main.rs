use structopt::StructOpt;
use crate::cli::{Args, ListArgs, ImageArgs};
use crate::common::AppError;

mod common;
mod cli;
mod get;
mod import;
mod tag;
mod feh;
mod autotags;


fn main() -> Result<(), AppError> {
    let args = Args::from_args();
    //println!("Got arguments {:?}", args);
    let res = match args {
        Args::Image(args) => {
            match args {
                ImageArgs::Import { no_auto_sources, no_feh, images } => {
                    import::import(no_auto_sources, no_feh, images)
                },
                ImageArgs::AddTags { tags, image } => {
                    tag::tag_add(tags, image)
                },
                ImageArgs::RmTags { tags, image } => {
                    tag::tag_rm(tags, image)
                },
                ImageArgs::GetTags { images } => {
                    tag::tag_get(images)
                }
            }
        }
        Args::List(args) => {
            match args {
                ListArgs::Images { temp_dir, include_tags, exclude_tags } => {
                    get::get(include_tags, exclude_tags, temp_dir)
                },
                ListArgs::Tags { tag_types } => {
                    tag::tag_list(tag_types)
                }
            }
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