use std::collections::HashSet;
use crate::common::error::Result;
use crate::get_similar_images;
use super::SimilarImagesGroup;

pub fn group_similar_images<'a>(new_images: &Vec<&'a str>, images_in_db: &Vec<&'a str>) -> Result<Vec<SimilarImagesGroup<'a>>> {
    let mut image_groups: Vec<SimilarImagesGroup> = Vec::new();
    for &image in new_images {
        let group = match try_add_to_existing_image_group(image, &mut image_groups)? {
            Some(group) => group,
            None => add_to_new_image_group(image, &mut image_groups),
        };

        add_similar_old_images(image, group, images_in_db)?;
    }

    Ok(image_groups)
}

/// Try to fit image into a group with at least one similar new image
fn try_add_to_existing_image_group<'a, 'b>(image: &'a str, image_groups: &'b mut Vec<SimilarImagesGroup<'a>>) -> Result<Option<&'b mut SimilarImagesGroup<'a>>> {
    for group in image_groups.iter_mut() {
        let similars = get_similar_images(image, &group.new_images.iter().map(|&img| img).collect())?;
        if !similars.is_empty() {
            // found a group with similar new images
            group.new_images.insert(image);
            return Ok(Some(group));
        }
    }

    return Ok(None);
}

fn add_to_new_image_group<'a, 'b>(image: &'a str, image_groups: &'b mut Vec<SimilarImagesGroup<'a>>) -> &'b mut SimilarImagesGroup<'a> {
    let mut new_group = SimilarImagesGroup::new();
    new_group.new_images.insert(image);

    image_groups.push(new_group);
    image_groups.last_mut().unwrap()
}

/// add all old images that are similar to image to image_group
fn add_similar_old_images<'a>(image: &str, image_group: &mut SimilarImagesGroup<'a>, old_images: &Vec<&'a str>) -> Result<()>{
    let similar_old_images = get_similar_images(image, old_images)?;
    image_group.old_images.extend(similar_old_images);
    Ok(())
}