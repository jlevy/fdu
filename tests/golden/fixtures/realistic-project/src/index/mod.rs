mod snapshot;
mod tree;

use std::path::{Path, PathBuf};

pub use snapshot::{Header, load, save};
pub use tree::{Entry, EntryId, RollUp};

pub struct Index {
    root: PathBuf,
    entries: Vec<Entry>,
}

impl Index {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into(), entries: Vec::new() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
}
