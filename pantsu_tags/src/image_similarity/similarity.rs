use crate::common::error::Result;
use crate::{ImageInfo};
use crate::file_handler::hash;
use crate::image_similarity::{ImageToImport, NamedImage};
use super::SimilarImagesGroup;

pub fn group_similar_images<'a>(new_images: &'a Vec<ImageToImport>, images_in_db: &'a Vec<ImageInfo>) -> Result<Vec<SimilarImagesGroup<'a>>> {
    let mut image_groups: Vec<SimilarImagesGroup> = Vec::new();
    for image in new_images {
        let group = match try_add_to_existing_image_group(image, &mut image_groups)? {
            Some(group) => group,
            None => add_to_new_image_group(image, &mut image_groups),
        };

        add_similar_old_images(image, group, images_in_db)?;
    }

    Ok(image_groups)
}

/// Try to fit image into a group with at least one similar new image
fn try_add_to_existing_image_group<'a, 'b>(image: &'a ImageToImport, image_groups: &'b mut Vec<SimilarImagesGroup<'a>>) -> Result<Option<&'b mut SimilarImagesGroup<'a>>> {
    for group in image_groups.iter_mut() {
        let similars = get_similar_images(image, group.new_images.iter().map(|&img| img))?; // todo: copies references, that's a bit sad...
        if !similars.is_empty() {
            // found a group with similar new images
            group.new_images.insert(image);  // todo: report lost new images
            return Ok(Some(group));
        }
    }

    return Ok(None);
}

fn add_to_new_image_group<'a, 'b>(image: &'a ImageToImport, image_groups: &'b mut Vec<SimilarImagesGroup<'a>>) -> &'b mut SimilarImagesGroup<'a> {
    let mut new_group = SimilarImagesGroup::new();
    new_group.new_images.insert(image);

    image_groups.push(new_group);
    image_groups.last_mut().unwrap()
}

/// add all old images that are similar to image to image_group
fn add_similar_old_images<'a>(image: &ImageToImport, image_group: &mut SimilarImagesGroup<'a>, old_images: &'a Vec<ImageInfo>) -> Result<()>{
    let similar_old_images = get_similar_images(image,old_images.iter().map(|i| i.get_image()))?;
    image_group.old_images.extend(similar_old_images);
    Ok(())
}


pub fn get_similar_images<'a, T, O, I>(image: &T, other_images: I) -> Result<Vec<&'a O>>
where T: NamedImage, O: NamedImage, I: Iterator<Item = &'a O>
{
    get_images_with_similarity_distance(image, other_images, 10) // todo: hardcoded value
}

pub fn get_images_with_similarity_distance<'a, T, O, I>(image: &T, other_images: I, max_dist: u32) -> Result<Vec<&'a O>>
where T: NamedImage, O: NamedImage, I: Iterator<Item = &'a O>
{
    let image_hash = hash::extract_hash(image.get_name())?; // todo: unwrap?
    Ok(other_images.filter(|&other| {
            let p_hash = hash::extract_hash(other.get_name()).unwrap();
            let dist = image_hash.distance(&p_hash);
            dist < max_dist
        }).collect())
}
