pub mod error;
pub mod pantsu_tag;
pub mod image_handle;

pub enum ImageRatio {
    Any,
    Max(f32),
    Min(f32),
    Range(f32, f32)
}