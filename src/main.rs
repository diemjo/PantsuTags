use std::path::{Path, PathBuf};
use structopt::StructOpt;
use pantsu_tags::db::PantsuDB;
use pantsu_tags::SauceQuality;
use pantsu_tags::PantsuTag;

fn main() {
    let args = Args::from_args();
    println!("Got arguments {:?}", args);
    match args {
        Args::Import{no_auto_sources, images} => {
            import(no_auto_sources, images);
        }
        Args::Get{ .. }  => {

        }
    }
}

fn import(no_auto_sources: bool, images: Vec<PathBuf>) {
    let pdb_path = Path::new("./pantsu_tags.db");
    let mut pdb = PantsuDB::new(pdb_path).unwrap();
    for image in images {
        println!("adding image {}", image.to_str().unwrap());
        let image_handle = pantsu_tags::new_image_handle(&pdb, &image, true).unwrap();
        if no_auto_sources {
            let no_tags: Vec<PantsuTag> = Vec::new();
            pantsu_tags::store_image_with_tags(&mut pdb, &image_handle, &no_tags).unwrap();
        }
        else {
            let sauces = pantsu_tags::get_image_sauces(&image_handle).unwrap();
            if let SauceQuality::Found = sauces.0 {
                let tags = pantsu_tags::get_sauce_tags(&sauces.1[0]).unwrap();
                pantsu_tags::store_image_with_tags_from_sauce(&mut pdb, &image_handle, &sauces.1[0], &tags).unwrap();
            }
            else {
                panic!("not yet implemented");
                // todo
            }
        }
        println!("imported image {} into PantsuTags", image.to_str().unwrap())
    }
    println!("imported all images")
}

#[derive(Debug, StructOpt)]
#[structopt(name = "PantsuTags", about = "PantsuTags CLI")]
enum Args {
    Import {
        #[structopt(short="s", long)]
        no_auto_sources: bool,

        #[structopt(parse(from_os_str))]
        images: Vec<PathBuf>,
    },
    Get {
        #[structopt(short, long)]
        dummy: bool,
    }
}