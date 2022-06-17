use clap::{Parser, ArgGroup};
use std::path::PathBuf;
use pantsu_tags::{PantsuTag, PantsuTagType};

#[derive(Debug, Parser)]
#[clap(name = "PantsuTags", about = "PantsuTags CLI")]
pub enum Args {
    ImportImages(ImportImagesArgs),
    RemoveImages(RemoveImagesArgs),
    AddTags(AddTagsArgs),
    RemoveTags(RemoveTagsArgs),
    ListTags(ListTagsArgs),
    ImageInfos(ImageInfosArgs),
    ListImages(ListImagesArgs),
    AutoLookupTags(AutoLookupTagsArgs),
}

#[derive(Debug, Parser)]
pub struct ImportImagesArgs {
    #[clap(parse(from_os_str), required=true, min_values=1)]
    pub images: Vec<PathBuf>,
    #[clap(short, long)]
    pub no_feh: bool
}

#[derive(Debug, Parser)]
pub struct RemoveImagesArgs {
    #[clap(parse(from_os_str), required=true, min_values=1)]
    pub images: Vec<PathBuf>,
}

#[derive(Debug, Parser)]
pub struct AddTagsArgs {
    #[clap(short, long, parse(from_os_str), required=true, min_values=1)]
    pub images: Vec<PathBuf>,
    #[clap(short, long, parse(try_from_str), required=true, min_values=1)]
    pub tags: Vec<PantsuTag>,
}

#[derive(Debug, Parser)]
pub struct RemoveTagsArgs {
    #[clap(short, long, parse(from_os_str), required=true, min_values=1)]
    pub images: Vec<PathBuf>,
    #[clap(short, long, required=true, min_values=1)]
    pub tags: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct ListTagsArgs {
    #[clap(short, long, parse(from_os_str))]
    pub images: Vec<PathBuf>,
    #[clap(short, long="types", parse(try_from_str))]
    pub tag_types: Vec<PantsuTagType>,
}

#[derive(Debug, Parser)]
pub struct ImageInfosArgs {
    #[clap(short, long, parse(from_os_str))]
    pub images: Vec<PathBuf>,
}

#[derive(Debug, Parser)]
#[clap(group(ArgGroup::new("sauce").args(&["sauce-existing", "sauce-not-existing", "sauce-not-checked"])))]
pub struct ListImagesArgs {
    #[clap(short, long)]
    pub include_tags: Vec<String>,
    #[clap(short, long)]
    pub exclude_tags: Vec<String>,

    #[clap(short='l', long)]
    pub aspect_ratio_min: Option<f32>,
    #[clap(short='u', long)]
    pub aspect_ratio_max: Option<f32>,

    #[clap(short='s', long)]
    pub sauce_existing: bool,
    #[clap(short='n', long)]
    pub sauce_not_existing: bool,
    #[clap(short='c', long)]
    pub sauce_not_checked: bool,
}

#[derive(Debug, Parser)]
#[clap(group(ArgGroup::new("sauce").args(&["sauce-existing", "sauce-not-existing", "sauce-not-checked"])))]
pub struct AutoLookupTagsArgs {
    #[clap(short, long, parse(from_os_str))]
    pub images: Vec<PathBuf>,
    #[clap(long)]
    pub no_feh: bool,

    #[clap(short='s', long)]
    pub sauce_existing: bool,
    #[clap(short='n', long)]
    pub sauce_not_existing: bool,
    #[clap(short='c', long)]
    pub sauce_not_checked: bool,
}