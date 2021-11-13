pub mod tag_finder;
pub mod sauce_finder;

pub struct SauceMatch {
    pub link: String,
    pub similarity: f32,
}