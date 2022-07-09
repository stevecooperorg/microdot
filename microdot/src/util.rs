use anyhow::{anyhow, Result};
use std::path::*;
use unfold::Unfold;

pub fn git_root() -> Result<PathBuf> {
    let current_dir: &Path = &std::env::current_dir()?;
    let mut ancestors = Unfold::new(|path| &path.parent().unwrap(), current_dir);

    fn has_git_dir(path: &Path) -> bool {
        std::fs::read_dir(path)
            .unwrap()
            .flatten()
            .any(|p| p.file_name() == ".git")
    }

    let git_root = ancestors
        .find(|&path| has_git_dir(path))
        .ok_or_else(|| anyhow!("could not find the git root"))?;

    Ok(git_root.into())
}
