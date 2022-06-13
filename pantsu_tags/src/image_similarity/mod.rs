use std::collections::HashSet;

mod similarity;

pub use similarity::group_similar_images;

/// Group of images that are similar to each other
pub struct SimilarImagesGroup<'a> {
    /// Images that will be newly added
    pub new_images: HashSet<&'a str>,
    /// Images that are present in PantsuTags
    pub old_images: HashSet<&'a str>,
}

impl<'a> SimilarImagesGroup<'a> {
    pub(crate) fn new() -> Self {
        Self {
            new_images: HashSet::new(),
            old_images: HashSet::new(),
        }
    }

    pub fn is_single_image(&self) -> bool {
        self.new_images.len() == 1 && self.old_images.is_empty()
    }
}
