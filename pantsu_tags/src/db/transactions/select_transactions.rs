use rusqlite::{Transaction};
use crate::db::transactions::{GetImagesTransaction, GetImageTransaction, GetTagsTransaction, PantsuDBTransaction};
use crate::{ImageHandle, ImageRatio, PantsuTag, PantsuTagType};
use crate::common::error::Result;
use crate::db::db_calls;

impl<'p> GetImagesTransaction<'p> {
    pub fn new() -> Self {
        GetImagesTransaction {
            include_tags: Vec::new(),
            exclude_tags: Vec::new(),
            ratio: ImageRatio::Any
        }
    }

    pub fn including_tags(mut self, tags: &'p Vec<String>) -> Self {
        for tag in tags {
            self.include_tags.push(tag);
        }
        self
    }

    pub fn exclude_tags(mut self, tags: &'p Vec<String>) -> Self {
        for tag in tags {
            self.exclude_tags.push(tag);
        }
        self
    }

    pub fn with_ratio(mut self, ratio: ImageRatio) -> Self {
        self.ratio = ratio;
        self
    }
}

impl<'p> PantsuDBTransaction<Vec<ImageHandle>> for GetImagesTransaction<'p> {
    fn execute(self, transaction: &mut Transaction) -> Result<Vec<ImageHandle>> {
        let (ratio_min, ratio_max) = match self.ratio {
            ImageRatio::Any => (0f32, f32::MAX),
            ImageRatio::Min(min) => (f32::max(0f32, min), f32::MAX),
            ImageRatio::Max(max) => (0f32, f32::max(0f32, max)),
            ImageRatio::Range(min, max) => (f32::max(0f32, min), f32::max(f32::max(0f32, min), max))
        };
        let res = if self.include_tags.len()==0 && self.exclude_tags.len()==0 {
            match self.ratio {
                ImageRatio::Any => db_calls::get_all_files(transaction)?,
                _ => db_calls::get_all_files_with_ratio(&transaction, ratio_min, ratio_max)?
            }
        } else {
            db_calls::get_files_with_tags(&transaction, &self.include_tags, &self.exclude_tags, ratio_min, ratio_max)?
        };
        Ok(res)
    }
}

impl<'p> GetImageTransaction<'p> {
    pub fn new(image_name: &'p str) -> Self {
        GetImageTransaction {
            image_name
        }
    }
}

impl<'p> PantsuDBTransaction<Option<ImageHandle>> for GetImageTransaction<'p> {
    fn execute(self, transaction: &mut Transaction) -> Result<Option<ImageHandle>> {
        db_calls::get_file(transaction, self.image_name)
    }
}

impl<'p> GetTagsTransaction<'p> {
    pub fn new() -> Self {
        GetTagsTransaction {
            tag_types: Vec::new(),
            image_handle: None
        }
    }

    pub fn with_types(mut self, tag_types: &'p Vec<PantsuTagType>) -> Self {
        for tag_type in tag_types {
            self.tag_types.push(tag_type)
        }
        self
    }

    pub fn only_for_image(mut self, image: &'p ImageHandle) -> Self {
        self.image_handle = Some(image);
        self
    }
}

impl<'p> PantsuDBTransaction<Vec<PantsuTag>> for GetTagsTransaction<'p> {
    fn execute(self, transaction: &mut Transaction) -> Result<Vec<PantsuTag>> {
        match self.tag_types.len() {
            0 => match self.image_handle {
                Some(image) => db_calls::get_tags_for_file(transaction, image),
                None => db_calls::get_all_tags(transaction)
            }
            _more => match self.image_handle {
                Some(image) => db_calls::get_tags_for_file_with_types(transaction, image, &self.tag_types),
                None => db_calls::get_tags_with_types(transaction, &self.tag_types)
            }
        }
    }
}