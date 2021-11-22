use std::path::PathBuf;
use structopt::StructOpt;

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
    #[structopt(about="List tags in database, filter by tag type if provided")]
    ListTags {
        #[structopt(short, long="type")]
        tag_type: Vec<String>
    }
}