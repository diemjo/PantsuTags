use rusqlite::{Transaction};
use crate::common::error::Result;
use crate::{ImageHandle, PantsuTag, PantsuTagType, Sauce};
use crate::common::ImageRatio;

mod insert_transactions;
mod update_transactions;
mod delete_transactions;
mod select_transactions;

pub trait PantsuDBTransaction<T> {
    fn execute(self, transaction: &mut Transaction) -> Result<T>;
}

pub struct AddImageTransaction<'t> {
    image: &'t ImageHandle,
}

pub struct AddTagsTransaction<'t, 'p> {
    image: &'t ImageHandle,
    tags: Vec<&'p PantsuTag>,
    update_source: Option<Sauce>
}

pub struct UpdateImageTransaction {
    image: ImageHandle
}

pub struct RemoveImageTransaction {
    image: ImageHandle
}

pub struct RemoveTagsTransaction<'t, 'p> {
    image: &'t ImageHandle,
    tags: Vec<&'p str>,
    remove_all: bool
}

pub struct GetImagesTransaction<'p> {
    include_tags: Vec<&'p str>,
    exclude_tags: Vec<&'p str>,
    ratio: ImageRatio
}

pub struct GetImageTransaction<'p> {
    image_name: &'p str
}

pub struct GetTagsTransaction<'p> {
    tag_types: Vec<&'p PantsuTagType>,
    image_handle: Option<&'p ImageHandle>
}