use std::path::Path;
use crate::common::error::Error;
use crate::common::pantsu_tag::PantsuTag;
use crate::sauce::{sauce_finder, SauceMatch, tag_finder};

pub mod sauce;
pub mod common;
pub mod db;
pub mod file_handler;

const SIMILARITY_GOOD: f32 = 80.0;
const SIMILARITY_UNSURE: f32 = 50.0;

pub enum SauceQuality {
    Found,
    Unsure,
    NotFound,
}

pub fn add_image(image_path: &Path) -> Result<(SauceQuality, Vec<SauceMatch>), Error>{
    // opt: check whether file is image

    // file_handler: get file name
    let image_name = "x";

    // file_handler: check whether file already exists

    let sauce_matches = sauce_finder::find_sauce(image_path)?;
    //sauce_matches.sort();
    let best_match = &sauce_matches[0];
    return if best_match.similarity >= SIMILARITY_GOOD {
        let tags = tag_finder::find_tags_gelbooru(&best_match.link)?;
        // create img in db

        add_tags_to_image(image_name, &tags)?;
        Ok((SauceQuality::Found, sauce_matches))
    } else if best_match.similarity >= SIMILARITY_UNSURE {
        Ok((SauceQuality::Unsure, sauce_matches))
    } else {
        Ok((SauceQuality::NotFound, sauce_matches))
    }
}

pub fn add_tags_to_image(image_name: &str, tags: &Vec<PantsuTag>) -> Result<(), Error> {
    Ok(())
}
