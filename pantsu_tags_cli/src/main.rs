use structopt::StructOpt;
use crate::cli::Args;
use crate::common::AppError;
mod common;
mod cli;
mod get;
mod import;
mod feh;


fn main() -> Result<(), AppError> {
    let args = Args::from_args();
    println!("Got arguments {:?}", args);
    let res = match args {
        Args::Import{no_auto_sources, images} => {
            import::import(no_auto_sources, images)
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