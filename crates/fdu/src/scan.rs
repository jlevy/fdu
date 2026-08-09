//! The scan layer: walking a tree and producing deltas.
//!
//! A cold scan is just a large batch of upserts, and a revalidation sweep is the diff
//! between what the index believes and what the filesystem says. Both speak the same
//! [`Delta`] vocabulary as the watch layer, which is what lets the index be ignorant of
//! where its changes came from.
//!
//! # Status
//!
//! This is the portable `read_dir` + `symlink_metadata` implementation. It is correct
//! and it is the reference the fast path must match, but it is **not** the walker the
//! performance goal calls for: raw `getdents64` into a large reused per-thread buffer,
//! dirfd-relative `statx` with a narrow field mask, `d_type` stat-avoidance, and a
//! work-stealing pool. Until that layer lands and the benchmark gate passes, no
//! performance claim should be made for this crate.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::index::Index;
use crate::types::{Attrs, Delta, EntryKind, Error, Op, Result};

/// How many ops accumulate before a delta is handed to the sink.
///
/// Batching matters for more than syscall economy: consumers coalesce per path within a
/// batch and stat once per batch, and a live UI wants partial results while a large tree
/// is still being walked rather than one delta at the end.
const DEFAULT_BATCH_SIZE: usize = 1024;

/// Knobs for a scan.
#[derive(Clone, Debug)]
pub struct ScanConfig {
    /// Maximum directory depth to descend. `None` means unlimited.
    pub max_depth: Option<usize>,
    /// Ops per emitted delta.
    pub batch_size: usize,
    /// Follow symlinks to directories. Off by default: following them turns a tree walk
    /// into a graph walk with cycles, and every surveyed tool defaults to off.
    pub follow_symlinks: bool,
    /// Stay on the filesystem the root lives on.
    pub one_filesystem: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            max_depth: None,
            batch_size: DEFAULT_BATCH_SIZE,
            follow_symlinks: false,
            one_filesystem: false,
        }
    }
}

/// What a scan did, including the errors it walked past.
///
/// Unreadable directories are skipped rather than aborting the scan — a permission-denied
/// subdirectory should not cost you the other 499,000 files — but they are reported
/// rather than swallowed, so a caller can tell a complete answer from a partial one.
#[derive(Debug, Default)]
pub struct ScanReport {
    /// Directories successfully listed.
    pub dirs_read: u64,
    /// Entries observed, directories included.
    pub entries: u64,
    /// Paths that could not be read, with the reason.
    pub errors: Vec<Error>,
}

impl ScanReport {
    /// True when every directory in scope was read successfully.
    pub fn is_complete(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Walk `root` and emit deltas describing everything found.
pub fn scan(root: &Path, config: &ScanConfig, sink: &mut dyn FnMut(Delta)) -> Result<ScanReport> {
    let root_meta = fs::symlink_metadata(root).map_err(|e| Error::io(root, e))?;
    if !root_meta.is_dir() {
        return Err(Error::io(
            root,
            std::io::Error::new(std::io::ErrorKind::NotADirectory, "scan root is not a directory"),
        ));
    }
    let root_dev = attrs_from(&root_meta).dev;

    let mut report = ScanReport::default();
    let mut batch: Vec<Op> = Vec::with_capacity(config.batch_size);
    // Breadth-first: it keeps more requests in flight on a cold cache, which is the
    // case that dominates a first run. Depth-first has better locality once everything
    // is in page cache; making traversal order a tunable is deliberate future work.
    let mut queue: Vec<(PathBuf, usize)> = vec![(PathBuf::new(), 0)];

    while let Some((rel_dir, depth)) = queue.pop() {
        let abs_dir = root.join(&rel_dir);
        let listing = match fs::read_dir(&abs_dir) {
            Ok(listing) => listing,
            Err(e) => {
                report.errors.push(Error::io(abs_dir, e));
                continue;
            }
        };
        report.dirs_read += 1;

        for item in listing {
            let item = match item {
                Ok(item) => item,
                Err(e) => {
                    report.errors.push(Error::io(&abs_dir, e));
                    continue;
                }
            };
            let name = item.file_name();
            let rel_path = rel_dir.join(&name);
            let meta = match fs::symlink_metadata(item.path()) {
                Ok(meta) => meta,
                Err(e) => {
                    report.errors.push(Error::io(item.path(), e));
                    continue;
                }
            };

            let attrs = attrs_from(&meta);
            let kind = kind_from(&meta);
            report.entries += 1;
            batch.push(Op::Upsert { path: rel_path.clone(), kind, attrs });
            if batch.len() >= config.batch_size {
                sink(Delta::new(std::mem::take(&mut batch)));
                batch.reserve(config.batch_size);
            }

            let descend = kind.is_dir()
                || (config.follow_symlinks && meta.is_symlink() && item.path().is_dir());
            let within_depth = config.max_depth.is_none_or(|max| depth + 1 < max);
            let same_fs = !config.one_filesystem || attrs.dev == root_dev || attrs.dev == 0;
            if descend && within_depth && same_fs {
                queue.push((rel_path, depth + 1));
            }
        }
    }

    if !batch.is_empty() {
        sink(Delta::new(batch));
    }
    Ok(report)
}

/// Walk `root` and return a fully populated index.
pub fn scan_into_index(root: &Path, config: &ScanConfig) -> Result<(Index, ScanReport)> {
    let mut index = Index::new(root);
    let mut deltas = Vec::new();
    let report = scan(root, config, &mut |delta| deltas.push(delta))?;
    for delta in &deltas {
        index.apply(delta);
    }
    Ok((index, report))
}

/// Diff the filesystem against an existing index and emit deltas for the difference.
///
/// This is cache tier 2: after a snapshot is loaded, a sweep like this is what makes the
/// answer trustworthy rather than merely fast. Unchanged entries produce upserts whose
/// fingerprints already match, which the index discards as no-ops, so the caller can
/// apply the whole stream without filtering it first.
///
/// Entries the index holds but the filesystem no longer has become [`Op::Remove`],
/// detected per directory rather than by accumulating every visited path in memory.
pub fn revalidate(
    index: &Index,
    config: &ScanConfig,
    sink: &mut dyn FnMut(Delta),
) -> Result<ScanReport> {
    let root = index.root_path().to_path_buf();
    let mut report = ScanReport::default();
    let mut batch: Vec<Op> = Vec::with_capacity(config.batch_size);
    let mut queue: Vec<(PathBuf, usize)> = vec![(PathBuf::new(), 0)];

    while let Some((rel_dir, depth)) = queue.pop() {
        let abs_dir = root.join(&rel_dir);
        let listing = match fs::read_dir(&abs_dir) {
            Ok(listing) => listing,
            Err(e) => {
                report.errors.push(Error::io(abs_dir, e));
                continue;
            }
        };
        report.dirs_read += 1;

        let mut seen: BTreeSet<String> = BTreeSet::new();
        for item in listing {
            let item = match item {
                Ok(item) => item,
                Err(e) => {
                    report.errors.push(Error::io(&abs_dir, e));
                    continue;
                }
            };
            let name = item.file_name();
            seen.insert(name.to_string_lossy().into_owned());
            let rel_path = rel_dir.join(&name);
            let meta = match fs::symlink_metadata(item.path()) {
                Ok(meta) => meta,
                Err(e) => {
                    report.errors.push(Error::io(item.path(), e));
                    continue;
                }
            };

            let kind = kind_from(&meta);
            report.entries += 1;
            batch.push(Op::Upsert { path: rel_path.clone(), kind, attrs: attrs_from(&meta) });
            if batch.len() >= config.batch_size {
                sink(Delta::new(std::mem::take(&mut batch)));
                batch.reserve(config.batch_size);
            }

            if kind.is_dir() && config.max_depth.is_none_or(|max| depth + 1 < max) {
                queue.push((rel_path, depth + 1));
            }
        }

        // Anything the index still lists here but the filesystem did not return is gone.
        if let Some(known) = index.children(&rel_dir) {
            for (name, _) in known {
                if !seen.contains(name) {
                    batch.push(Op::Remove { path: rel_dir.join(name) });
                }
            }
        }
    }

    if !batch.is_empty() {
        sink(Delta::new(batch));
    }
    Ok(report)
}

/// Read an entry's kind and roll-up attributes out of its metadata.
///
/// Exposed so the watch layer verifies entries exactly the way the walker records them —
/// two stat interpretations that could drift would show up as an index that disagrees
/// with itself depending on which producer last touched a path.
pub fn observe(meta: &fs::Metadata) -> (EntryKind, Attrs) {
    (kind_from(meta), attrs_from(meta))
}

fn kind_from(meta: &fs::Metadata) -> EntryKind {
    let file_type = meta.file_type();
    if file_type.is_symlink() {
        EntryKind::Symlink
    } else if file_type.is_dir() {
        EntryKind::Dir
    } else if file_type.is_file() {
        EntryKind::File
    } else {
        EntryKind::Other
    }
}

#[cfg(unix)]
fn attrs_from(meta: &fs::Metadata) -> Attrs {
    use std::os::unix::fs::MetadataExt;
    Attrs {
        size: meta.size(),
        // st_blocks is in 512-byte units by POSIX convention regardless of the
        // filesystem's own block size.
        allocated: meta.blocks().saturating_mul(512),
        mtime_ns: compose_ns(meta.mtime(), meta.mtime_nsec()),
        ctime_ns: compose_ns(meta.ctime(), meta.ctime_nsec()),
        inode: meta.ino(),
        dev: meta.dev(),
    }
}

#[cfg(unix)]
fn compose_ns(secs: i64, nanos: i64) -> i64 {
    secs.saturating_mul(1_000_000_000).saturating_add(nanos)
}

#[cfg(not(unix))]
fn attrs_from(meta: &fs::Metadata) -> Attrs {
    let mtime_ns = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .and_then(|d| i64::try_from(d.as_nanos()).ok())
        .unwrap_or(0);
    Attrs {
        size: meta.len(),
        // No allocated size without platform-specific calls; apparent size is the
        // honest fallback rather than a guess at block rounding.
        allocated: meta.len(),
        mtime_ns,
        // Windows has no ctime in the Unix sense. Leaving it zero means the fingerprint
        // degrades to size + mtime there, which is what every portable tool does.
        ctime_ns: 0,
        inode: 0,
        dev: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn write_file(path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        let mut f = File::create(path).expect("create file");
        f.write_all(contents).expect("write");
    }

    fn sample_tree() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(&dir.path().join("a.txt"), b"hello");
        write_file(&dir.path().join("src/main.rs"), b"fn main() {}");
        write_file(&dir.path().join("src/deep/nested.rs"), b"// nested");
        dir
    }

    #[test]
    fn scan_populates_an_index_end_to_end() {
        let dir = sample_tree();
        let (index, report) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");

        assert!(report.is_complete(), "unexpected errors: {:?}", report.errors);
        let total = index.total();
        assert_eq!(total.files, 3);
        assert_eq!(total.dirs, 2);
        assert_eq!(total.bytes, 5 + 12 + 9);
        assert_eq!(total.by_ext[".rs"].files, 2);
        assert_eq!(total.by_ext[".txt"].files, 1);

        let src = index.rollup(Path::new("src")).expect("src");
        assert_eq!(src.files, 2);
        assert_eq!(src.dirs, 1);
    }

    #[test]
    fn max_depth_stops_descent() {
        let dir = sample_tree();
        let config = ScanConfig { max_depth: Some(1), ..ScanConfig::default() };
        let (index, _) = scan_into_index(dir.path(), &config).expect("scan");

        assert!(index.lookup(Path::new("src")).is_some());
        assert!(index.lookup(Path::new("src/main.rs")).is_none());
    }

    #[test]
    fn scanning_a_file_is_an_error_not_a_panic() {
        let dir = sample_tree();
        let err = scan_into_index(&dir.path().join("a.txt"), &ScanConfig::default());
        assert!(err.is_err());
    }

    #[test]
    fn deltas_arrive_in_batches_of_the_configured_size() {
        let dir = tempfile::tempdir().expect("tempdir");
        for i in 0..25 {
            write_file(&dir.path().join(format!("f{i}.txt")), b"x");
        }
        let config = ScanConfig { batch_size: 10, ..ScanConfig::default() };
        let mut sizes = Vec::new();
        scan(dir.path(), &config, &mut |d| sizes.push(d.len())).expect("scan");

        assert!(sizes.len() >= 3, "expected several batches, got {sizes:?}");
        assert!(sizes.iter().all(|&n| n <= 10));
        assert_eq!(sizes.iter().sum::<usize>(), 25);
    }

    #[test]
    fn revalidate_is_a_no_op_against_an_unchanged_tree() {
        let dir = sample_tree();
        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        let before = index.total().clone();

        let mut deltas = Vec::new();
        revalidate(&index, &ScanConfig::default(), &mut |d| deltas.push(d)).expect("revalidate");
        let mut unchanged = 0;
        for delta in &deltas {
            unchanged += index.apply(delta).unchanged;
        }

        assert_eq!(unchanged, 5, "3 files + 2 dirs all already known");
        assert_eq!(index.total(), &before);
    }

    #[test]
    fn revalidate_detects_additions_edits_and_deletions() {
        let dir = sample_tree();
        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");

        fs::remove_file(dir.path().join("a.txt")).expect("remove");
        write_file(&dir.path().join("src/main.rs"), b"fn main() { longer }");
        write_file(&dir.path().join("added.md"), b"new");

        let mut deltas = Vec::new();
        revalidate(&index, &ScanConfig::default(), &mut |d| deltas.push(d)).expect("revalidate");
        let mut stats = crate::index::ApplyStats::default();
        for delta in &deltas {
            let s = index.apply(delta);
            stats.inserted += s.inserted;
            stats.updated += s.updated;
            stats.removed += s.removed;
        }

        assert_eq!(stats.inserted, 1, "added.md");
        assert_eq!(stats.updated, 1, "main.rs grew");
        assert_eq!(stats.removed, 1, "a.txt is gone");

        let total = index.total();
        assert_eq!(total.files, 3);
        assert_eq!(total.bytes, 20 + 9 + 3);
        assert!(!total.by_ext.contains_key(".txt"));
        assert_eq!(total.by_ext[".md"].files, 1);
    }

    #[test]
    fn revalidate_removes_a_whole_vanished_directory() {
        let dir = sample_tree();
        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        fs::remove_dir_all(dir.path().join("src")).expect("remove dir");

        let mut deltas = Vec::new();
        revalidate(&index, &ScanConfig::default(), &mut |d| deltas.push(d)).expect("revalidate");
        for delta in &deltas {
            index.apply(delta);
        }

        let total = index.total();
        assert_eq!(total.files, 1);
        assert_eq!(total.dirs, 0);
        assert!(index.lookup(Path::new("src")).is_none());
    }
}
