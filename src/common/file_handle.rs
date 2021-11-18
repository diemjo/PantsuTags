
pub struct FileHandle {
    filename: String
}

impl FileHandle {
    pub fn get_filename(&self) -> &str {
        &self.filename
    }
}

impl FileHandle {
    pub(in crate) fn new(filename: String) -> Self {
        FileHandle {
            filename,
        }
    }
}