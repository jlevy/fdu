//! Inspecting and clearing the snapshot cache.
//!
//! Snapshot files are named by a hash of their root, which keeps two trees from
//! colliding but leaves a directory of opaque names with no way to answer "which tree is
//! this?". These functions read each file's bounded header to recover that mapping, so
//! the cache can be inspected and cleared without guesswork.
//!
//! Every read here is header-only and bounded: a corrupt or foreign file is reported as
//! unrecognized rather than parsed, and is never deleted.

use std::fs;
use std::path::{Path, PathBuf};

use crate::snapshot;
use crate::types::{Error, Result, ScanScope};

/// What is known about one file in the cache directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheStatus {
    /// Where the file lives.
    pub path: PathBuf,
    /// Size on disk, in bytes.
    pub bytes: u64,
    /// What the header says, when the file is a snapshot this build can read.
    pub snapshot: Option<SnapshotInfo>,
}

impl CacheStatus {
    /// Whether this file is a snapshot this build understands.
    pub fn is_recognized(&self) -> bool {
        self.snapshot.is_some()
    }
}

/// The header facts that identify a snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotInfo {
    /// Absolute path of the tree this snapshot describes.
    pub root: PathBuf,
    /// The scan scope it was captured under.
    pub scope: ScanScope,
    /// How many entries it holds.
    pub entries: u64,
}

/// Read one cache file's status without materializing its index.
pub fn cache_status(path: &Path) -> Result<CacheStatus> {
    let bytes = fs::metadata(path).map_or(0, |meta| meta.len());
    Ok(CacheStatus { path: path.to_path_buf(), bytes, snapshot: snapshot::read_header(path)? })
}

/// Enumerate every file in the cache directory holding fdu snapshots.
///
/// Unrecognized files are listed with `snapshot: None` rather than hidden: a directory
/// this code will not delete from is a directory the caller should still be able to see.
pub fn list_caches(cache_dir: &Path) -> Result<Vec<CacheStatus>> {
    let entries = match fs::read_dir(cache_dir) {
        Ok(entries) => entries,
        // An absent cache directory is an empty cache, not an error.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(Error::io(cache_dir, error)),
    };

    let mut found = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| Error::io(cache_dir, error))?;
        let path = entry.path();
        if path.is_file() {
            found.push(cache_status(&path)?);
        }
    }
    // Deterministic order, so two runs of a status command agree.
    found.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(found)
}

/// Remove one snapshot, if it is one.
///
/// Returns whether a file was removed. Idempotent: clearing an absent cache succeeds.
pub fn clear_cache(path: &Path) -> Result<bool> {
    if snapshot::read_header(path)?.is_none() {
        // Refusing to delete what this code cannot identify is the whole safety property:
        // the cache directory may hold files this build did not write.
        return Ok(false);
    }
    match fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(Error::io(path, error)),
    }
}

/// Remove every recognized snapshot in a cache directory, leaving anything else alone.
///
/// Returns how many files were removed.
pub fn clear_all_caches(cache_dir: &Path) -> Result<usize> {
    let mut removed = 0;
    for status in list_caches(cache_dir)? {
        if status.is_recognized() && clear_cache(&status.path)? {
            removed += 1;
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CachePolicy, OpenConfig, open};

    fn seed(tree: &Path, snapshot_path: &Path) {
        std::fs::write(tree.join("a.txt"), b"hello").expect("write");
        let config = OpenConfig {
            cache_path: Some(snapshot_path.to_path_buf()),
            policy: CachePolicy::Auto,
            ..OpenConfig::default()
        };
        open(tree, &config).expect("seed");
    }

    #[test]
    fn a_snapshot_reports_the_root_it_describes() {
        // The property that makes the cache inspectable: a hash-named file can say which
        // tree it belongs to.
        let tree = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache");
        let path = cache.path().join("snap.fdu");
        seed(tree.path(), &path);

        let status = cache_status(&path).expect("status");
        assert!(status.is_recognized());
        assert!(status.bytes > 0);
        let info = status.snapshot.expect("header");
        assert_eq!(info.root, tree.path().canonicalize().expect("canonical"));
        assert_eq!(info.entries, 2, "the root plus one file");
    }

    #[test]
    fn unrecognized_files_are_listed_but_never_removed() {
        let tree = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache");
        seed(tree.path(), &cache.path().join("snap.fdu"));
        let foreign = cache.path().join("notes.txt");
        std::fs::write(&foreign, b"not a snapshot").expect("write");

        let listed = list_caches(cache.path()).expect("list");
        assert_eq!(listed.len(), 2);
        assert_eq!(listed.iter().filter(|status| status.is_recognized()).count(), 1);

        assert_eq!(clear_all_caches(cache.path()).expect("clear"), 1);
        assert!(foreign.exists(), "a file this code cannot identify must survive");
        assert_eq!(list_caches(cache.path()).expect("list").len(), 1);
    }

    #[test]
    fn a_truncated_snapshot_is_unrecognized_rather_than_an_error() {
        // Corrupt equals absent, everywhere: a half-written file must not become an
        // error that stops a status command from reporting the rest.
        let tree = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache");
        let path = cache.path().join("snap.fdu");
        seed(tree.path(), &path);

        let full = std::fs::read(&path).expect("read");
        std::fs::write(&path, &full[..full.len() / 2]).expect("truncate");

        let status = cache_status(&path).expect("status");
        assert!(!status.is_recognized());
        assert!(!clear_cache(&path).expect("clear"), "an unreadable file is not ours to delete");
        assert!(path.exists());
    }

    #[test]
    fn clearing_is_idempotent_and_an_absent_directory_is_empty() {
        let cache = tempfile::tempdir().expect("cache");
        let missing = cache.path().join("does-not-exist");
        assert!(list_caches(&missing).expect("list").is_empty());
        assert_eq!(clear_all_caches(&missing).expect("clear"), 0);
        assert!(!clear_cache(&missing.join("snap.fdu")).expect("clear"));
    }
}
