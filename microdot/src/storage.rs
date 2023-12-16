use crate::util::write_if_different;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

trait Store {
    fn read<P: AsRef<Path>>(&self, path: P) -> Result<String>;
    fn write<P: AsRef<Path>, S: AsRef<str>>(&self, path: P, content: S) -> Result<()>;
}

struct FileStore {
    root: PathBuf,
}

impl FileStore {
    #[allow(dead_code)]
    fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl Store for FileStore {
    fn read<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        std::fs::read_to_string(path).with_context(|| {
            format!(
                "reading from file store rooted at {}",
                self.root.to_string_lossy()
            )
        })
    }

    fn write<P: AsRef<Path>, S: AsRef<str>>(&self, path: P, content: S) -> Result<()> {
        let bytes = content.as_ref().as_bytes();
        write_if_different(path, bytes).with_context(|| {
            format!(
                "writing to file store rooted at {}",
                self.root.to_string_lossy()
            )
        })
    }
}
