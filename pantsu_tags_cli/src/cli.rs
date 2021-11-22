use std::path::PathBuf;
use structopt::StructOpt;
use pantsu_tags::{PantsuTag, PantsuTagType};

#[derive(Debug, StructOpt)]
#[structopt(name = "PantsuTags", about = "PantsuTags CLI")]
pub enum Args {
    Import {
        #[structopt(short="s", long)]
        no_auto_sources: bool,

        #[structopt(parse(from_os_str))]
        images: Vec<PathBuf>,
    },
    #[structopt(about="List files, filtered by tags if provided")]
    Get {
        #[structopt(short, long, parse(from_os_str))]
        temp_dir: Option<PathBuf>,
        #[structopt(short, long)]
        include_tags: Vec<String>,
        #[structopt(short, long)]
        exclude_tags: Vec<String>,
    },
    #[structopt(about="Add and remove tags from images or list tags in database")]
    Tag(TagArgs),
}

#[derive(Debug, StructOpt)]
pub enum TagArgs {
    #[structopt(about = "List tags in database")]
    List {
        #[structopt(about = "Filter tags to list by tags type")]
        #[structopt(short, long="types", parse(try_from_str))]
        tag_types: Vec<PantsuTagType>,
    },

    #[structopt(about="Add tags to an image")]
    Add {
        #[structopt(about = "Tags to add to the image")]
        #[structopt(parse(try_from_str))]
        tags: Vec<PantsuTag>,

        #[structopt(about = "The image to add tags to")]
        #[structopt(short, long, parse(from_str))]
        image: String
    },
    #[structopt(about="Remove tags from an image")]
    Rm {
        #[structopt(about = "Tags to remove from the image")]
        tags: Vec<String>,

        #[structopt(about = "The image to remove tags from")]
        #[structopt(short, long, parse(from_str))]
        image: String
    }
}