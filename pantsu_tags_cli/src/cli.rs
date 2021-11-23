use std::path::PathBuf;
use structopt::StructOpt;
use pantsu_tags::{PantsuTag, PantsuTagType};

#[derive(Debug, StructOpt)]
#[structopt(name = "PantsuTags", about = "PantsuTags CLI")]
pub enum Args {
    Image(ImageArgs),
    List(ListArgs)
}

#[derive(Debug, StructOpt)]
pub enum ImageArgs {
    Import {
        #[structopt(long)]
        no_auto_sources: bool,

        #[structopt(long)]
        no_feh: bool,

        #[structopt(parse(from_os_str))]
        images: Vec<PathBuf>,
    },
    #[structopt(about="Add tags to an image")]
    AddTags {
        #[structopt(about = "Tags to add to the image")]
        #[structopt(parse(try_from_str))]
        tags: Vec<PantsuTag>,

        #[structopt(about = "The image to add tags to")]
        #[structopt(short, long, parse(from_str))]
        image: String
    },
    #[structopt(about="Remove tags from an image")]
    RmTags {
        #[structopt(about = "Tags to remove from the image")]
        tags: Vec<String>,

        #[structopt(about = "The image to remove tags from")]
        #[structopt(short, long, parse(from_str))]
        image: String
    },
    GetTags {
        #[structopt(about = "The image to retrieve tags for")]
        #[structopt(parse(from_str))]
        images: Vec<String>
    }
}

#[derive(Debug, StructOpt)]
pub enum ListArgs {
    #[structopt(about = "List tags in database")]
    Tags {
        #[structopt(about = "Filter tags to list by tags type")]
        #[structopt(short, long="types", parse(try_from_str))]
        tag_types: Vec<PantsuTagType>,
    },

    #[structopt(about="List files, filtered by tags if provided")]
    Images {
        #[structopt(short, long, parse(from_os_str))]
        temp_dir: Option<PathBuf>,
        #[structopt(short, long)]
        include_tags: Vec<String>,
        #[structopt(short, long)]
        exclude_tags: Vec<String>,
    },
}