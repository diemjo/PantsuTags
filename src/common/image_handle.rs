
pub struct ImageHandle {
    filename: String
}

impl ImageHandle {
    pub fn get_filename(&self) -> &str {
        &self.filename
    }
}

impl ImageHandle {
    pub(in crate) fn new(filename: String) -> Self {
        ImageHandle {
            filename,
        }
    }
}