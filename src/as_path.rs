use std::path::Path;

pub trait AsPath {
    fn as_path(&self) -> &Path;
}

impl<T: AsRef<str>> AsPath for T {
    fn as_path(&self) -> &Path {
        Path::new(self.as_ref())
    }
}