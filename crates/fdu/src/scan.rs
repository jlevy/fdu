//! The scan layer: walking a tree, producing observations, and applying reconciliation.
//!
//! A cold scan is just a large batch of upserts, and a revalidation sweep is the diff
//! between what the index believes and what the filesystem says. Both speak the same
//! [`Observation`] vocabulary as the watch layer, which is what lets the index be ignorant of
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

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::ApplyStats;
use crate::index::{Index, IndexHandle};
use crate::types::{
    AppliedDelta, Attrs, EntryKind, Error, Observation, ObservationOp, Op, Result, ScanScope,
};

/// How many ops accumulate before an observation is handed to the sink.
///
/// Batching matters for more than syscall economy: consumers coalesce per path within a
/// batch and stat once per batch, and a live UI wants partial results while a large tree
/// is still being walked rather than one delta at the end.
const DEFAULT_BATCH_SIZE: usize = 1024;

/// Identity of the current built-in ignore policy. No ignore rules exist yet.
const IGNORE_RULES_FINGERPRINT: u64 = 0;

/// Identity of the current compound-extension classifier.
const TYPE_RULES_FINGERPRINT: u64 = 1;

/// Identity of the fixed stat-tier reducer set.
const REDUCERS_FINGERPRINT: u64 = 1;

/// Knobs for a scan.
#[derive(Clone, Debug)]
pub struct ScanConfig {
    /// Maximum directory depth to descend. `None` means unlimited.
    pub max_depth: Option<usize>,
    /// Ops per emitted observation.
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

impl ScanConfig {
    /// Semantic cache identity, excluding operational batching choices.
    pub const fn scope(&self) -> ScanScope {
        ScanScope {
            max_depth: self.max_depth,
            follow_symlinks: self.follow_symlinks,
            one_filesystem: self.one_filesystem,
            ignore_rules_fingerprint: IGNORE_RULES_FINGERPRINT,
            type_rules_fingerprint: TYPE_RULES_FINGERPRINT,
            reducers_fingerprint: REDUCERS_FINGERPRINT,
        }
    }

    fn validate(&self) -> Result<()> {
        if self.follow_symlinks {
            return Err(Error::UnsupportedScanConfig(
                "follow_symlinks requires cycle, root-boundary, and filesystem-boundary semantics",
            ));
        }
        Ok(())
    }
}

impl Default for ScanScope {
    fn default() -> Self {
        ScanConfig::default().scope()
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

/// Filesystem and index effects from an applying reconciliation pass.
#[derive(Debug, Default)]
pub struct ReconcileReport {
    pub scan: ScanReport,
    pub apply: ApplyStats,
}

enum ReconcileTarget<'a> {
    Direct(&'a mut Index),
    Shared(&'a IndexHandle),
}

impl ReconcileTarget<'_> {
    fn root_path(&self) -> Result<PathBuf> {
        match self {
            Self::Direct(index) => Ok(index.root_path().to_path_buf()),
            Self::Shared(handle) => Ok(handle.read()?.root_path().to_path_buf()),
        }
    }

    fn path_state(&self, path: &Path) -> Result<crate::PathState> {
        match self {
            Self::Direct(index) => Ok(index.path_state(path)),
            Self::Shared(handle) => Ok(handle.read()?.path_state(path)),
        }
    }

    fn child_states(&self, path: &Path) -> Result<BTreeMap<OsString, crate::PathState>> {
        match self {
            Self::Direct(index) => Ok(collect_child_states(index, path)),
            Self::Shared(handle) => {
                let index = handle.read()?;
                Ok(collect_child_states(&index, path))
            }
        }
    }

    fn apply(&mut self, observation: &Observation) -> Result<crate::ApplyOutcome> {
        match self {
            Self::Direct(index) => Ok(index.apply(observation)),
            Self::Shared(handle) => handle.apply(observation),
        }
    }

    fn take_pending_invalidations(&mut self) -> Result<Vec<(PathBuf, crate::InvalidateReason)>> {
        match self {
            Self::Direct(index) => Ok(index.take_pending_invalidations()),
            Self::Shared(handle) => Ok(handle.write()?.take_pending_invalidations()),
        }
    }

    fn begin_reconcile(&mut self, path: &Path) -> Result<u64> {
        match self {
            Self::Direct(index) => Ok(index.begin_reconcile(path)),
            Self::Shared(handle) => Ok(handle.write()?.begin_reconcile(path)),
        }
    }

    fn finish_reconcile(&mut self, path: &Path, started_at: u64, complete: bool) -> Result<()> {
        match self {
            Self::Direct(index) => index.finish_reconcile(path, started_at, complete),
            Self::Shared(handle) => {
                handle.write()?.finish_reconcile(path, started_at, complete);
            }
        }
        Ok(())
    }
}

fn collect_child_states(index: &Index, path: &Path) -> BTreeMap<OsString, crate::PathState> {
    index
        .children(path)
        .unwrap_or_default()
        .into_iter()
        .map(|(name, _)| {
            let child_path = path.join(name);
            (name.to_os_string(), index.path_state(&child_path))
        })
        .collect()
}

/// Walk `root` and emit observations describing everything found.
pub fn scan(
    root: &Path,
    config: &ScanConfig,
    sink: &mut dyn FnMut(Observation),
) -> Result<ScanReport> {
    config.validate()?;
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
    // Depth-first keeps the portable reference implementation small and locality-friendly.
    // Traversal order becomes a measured choice once the parallel syscall layer lands.
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
                sink(Observation::new(std::mem::take(&mut batch)));
                batch.reserve(config.batch_size);
            }

            if should_descend(kind, attrs, depth, root_dev, config) {
                queue.push((rel_path, depth + 1));
            }
        }
    }

    if !batch.is_empty() {
        sink(Observation::new(batch));
    }
    Ok(report)
}

/// Walk `root` and return a fully populated index.
pub fn scan_into_index(root: &Path, config: &ScanConfig) -> Result<(Index, ScanReport)> {
    config.validate()?;
    let root = root.canonicalize().map_err(|error| Error::io(root, error))?;
    let mut index = Index::new_with_scope(&root, config.scope());
    let report = scan(&root, config, &mut |observation| {
        index.apply_baseline(&observation);
    })?;
    index.set_initial_freshness(report.is_complete());
    Ok((index, report))
}

/// Diff the filesystem against an existing index and emit conditional observations.
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
    sink: &mut dyn FnMut(Observation),
) -> Result<ScanReport> {
    config.validate()?;
    let root = index.root_path().to_path_buf();
    let root_meta = fs::symlink_metadata(&root).map_err(|error| Error::io(&root, error))?;
    if !root_meta.is_dir() {
        return Err(Error::io(
            &root,
            std::io::Error::new(
                std::io::ErrorKind::NotADirectory,
                "revalidation root is not a directory",
            ),
        ));
    }
    let root_dev = attrs_from(&root_meta).dev;
    let mut report = ScanReport::default();
    let mut batch: Vec<ObservationOp> = Vec::with_capacity(config.batch_size);
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

        let mut seen: BTreeSet<OsString> = BTreeSet::new();
        let mut listing_complete = true;
        for item in listing {
            let item = match item {
                Ok(item) => item,
                Err(e) => {
                    listing_complete = false;
                    report.errors.push(Error::io(&abs_dir, e));
                    continue;
                }
            };
            let name = item.file_name();
            seen.insert(name.clone());
            let rel_path = rel_dir.join(&name);
            let baseline = index.path_state(&rel_path);
            let meta = match fs::symlink_metadata(item.path()) {
                Ok(meta) => meta,
                Err(e) => {
                    report.errors.push(Error::io(item.path(), e));
                    continue;
                }
            };

            let kind = kind_from(&meta);
            let attrs = attrs_from(&meta);
            report.entries += 1;
            batch.push(ObservationOp::if_state(
                Op::Upsert { path: rel_path.clone(), kind, attrs },
                baseline,
            ));
            if batch.len() >= config.batch_size {
                sink(Observation::from_ops(std::mem::take(&mut batch)));
                batch.reserve(config.batch_size);
            }

            if should_descend(kind, attrs, depth, root_dev, config) {
                queue.push((rel_path, depth + 1));
            }
        }

        // Anything the index still lists here but the filesystem did not return is gone.
        if listing_complete {
            if let Some(known) = index.children(&rel_dir) {
                for (name, _) in known {
                    if !seen.contains(name) {
                        let path = rel_dir.join(name);
                        batch.push(ObservationOp::if_state(
                            Op::Remove { path: path.clone() },
                            index.path_state(&path),
                        ));
                    }
                }
            }
        }
    }

    if !batch.is_empty() {
        sink(Observation::from_ops(batch));
    }
    Ok(report)
}

/// Reconcile the full index and publish each effective committed delta as it lands.
pub fn reconcile(
    index: &mut Index,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    reconcile_subtree(index, Path::new(""), config, sink)
}

/// Reconcile one relative subtree, applying effective changes during the walk.
pub fn reconcile_subtree(
    index: &mut Index,
    subtree: &Path,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    reconcile_target(&mut ReconcileTarget::Direct(index), subtree, config, sink)
}

/// Reconcile a shared index while allowing readers between applied batches.
pub fn reconcile_handle(
    handle: &IndexHandle,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    reconcile_subtree_handle(handle, Path::new(""), config, sink)
}

/// Reconcile one subtree of a shared index.
pub fn reconcile_subtree_handle(
    handle: &IndexHandle,
    subtree: &Path,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    reconcile_target(&mut ReconcileTarget::Shared(handle), subtree, config, sink)
}

fn reconcile_target(
    target: &mut ReconcileTarget<'_>,
    subtree: &Path,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    config.validate()?;
    let subtree = normalize_subtree(subtree)?;
    let started_at = target.begin_reconcile(&subtree)?;
    match reconcile_target_inner(target, &subtree, config, sink) {
        Ok(report) => {
            target.finish_reconcile(&subtree, started_at, report.scan.is_complete())?;
            Ok(report)
        }
        Err(error) => {
            target.finish_reconcile(&subtree, started_at, false)?;
            Err(error)
        }
    }
}

fn reconcile_target_inner(
    target: &mut ReconcileTarget<'_>,
    subtree: &Path,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    let root = target.root_path()?;
    let root_meta = fs::symlink_metadata(&root).map_err(|error| Error::io(&root, error))?;
    if !root_meta.is_dir() {
        return Err(Error::io(
            &root,
            std::io::Error::new(
                std::io::ErrorKind::NotADirectory,
                "reconciliation root is not a directory",
            ),
        ));
    }
    let root_dev = attrs_from(&root_meta).dev;
    let start_depth = subtree.components().count();
    let mut report = ReconcileReport::default();
    let mut batch: Vec<ObservationOp> = Vec::with_capacity(config.batch_size.max(1));

    if !subtree.as_os_str().is_empty() {
        let baseline = target.path_state(subtree)?;
        let absolute = root.join(subtree);
        let meta = match fs::symlink_metadata(&absolute) {
            Ok(meta) => meta,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                batch.push(ObservationOp::if_state(
                    Op::Remove { path: subtree.to_path_buf() },
                    baseline,
                ));
                flush_reconcile_batch(target, &mut batch, sink, &mut report.apply)?;
                return Ok(report);
            }
            Err(error) => {
                report.scan.errors.push(Error::io(absolute, error));
                return Ok(report);
            }
        };
        let kind = kind_from(&meta);
        let attrs = attrs_from(&meta);
        report.scan.entries += 1;
        batch.push(ObservationOp::if_state(
            Op::Upsert { path: subtree.to_path_buf(), kind, attrs },
            baseline,
        ));
        flush_reconcile_batch(target, &mut batch, sink, &mut report.apply)?;
        if !should_descend(kind, attrs, start_depth.saturating_sub(1), root_dev, config) {
            return Ok(report);
        }
    }

    let mut queue = vec![(subtree.to_path_buf(), start_depth)];
    while let Some((rel_dir, depth)) = queue.pop() {
        let mut known = target.child_states(&rel_dir)?;
        let abs_dir = root.join(&rel_dir);
        let listing = match fs::read_dir(&abs_dir) {
            Ok(listing) => listing,
            Err(error) => {
                report.scan.errors.push(Error::io(abs_dir, error));
                continue;
            }
        };
        report.scan.dirs_read += 1;
        let mut listing_complete = true;

        for item in listing {
            let item = match item {
                Ok(item) => item,
                Err(error) => {
                    listing_complete = false;
                    report.scan.errors.push(Error::io(&abs_dir, error));
                    continue;
                }
            };
            let name = item.file_name();
            let rel_path = rel_dir.join(&name);
            let baseline = known.remove(&name).unwrap_or(crate::PathState::Absent);
            let meta = match fs::symlink_metadata(item.path()) {
                Ok(meta) => meta,
                Err(error) => {
                    report.scan.errors.push(Error::io(item.path(), error));
                    continue;
                }
            };
            let kind = kind_from(&meta);
            let attrs = attrs_from(&meta);
            report.scan.entries += 1;
            batch.push(ObservationOp::if_state(
                Op::Upsert { path: rel_path.clone(), kind, attrs },
                baseline,
            ));
            if batch.len() >= config.batch_size.max(1) {
                flush_reconcile_batch(target, &mut batch, sink, &mut report.apply)?;
            }

            if should_descend(kind, attrs, depth, root_dev, config) {
                queue.push((rel_path, depth + 1));
            }
        }

        if listing_complete {
            for (name, baseline) in known {
                batch.push(ObservationOp::if_state(
                    Op::Remove { path: rel_dir.join(name) },
                    baseline,
                ));
                if batch.len() >= config.batch_size.max(1) {
                    flush_reconcile_batch(target, &mut batch, sink, &mut report.apply)?;
                }
            }
        }
    }

    flush_reconcile_batch(target, &mut batch, sink, &mut report.apply)?;
    Ok(report)
}

/// Drain and reconcile every pending invalidation, collapsing nested requests.
pub fn reconcile_pending(
    index: &mut Index,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    let mut target = ReconcileTarget::Direct(index);
    let roots = take_invalidation_roots(&mut target)?;
    let mut combined = ReconcileReport::default();
    for root in roots {
        let report = reconcile_target(&mut target, &root, config, sink)?;
        merge_reconcile_report(&mut combined, report);
    }
    Ok(combined)
}

/// Drain and reconcile invalidations on a shared index.
pub fn reconcile_pending_handle(
    handle: &IndexHandle,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    let mut target = ReconcileTarget::Shared(handle);
    let roots = take_invalidation_roots(&mut target)?;
    let mut combined = ReconcileReport::default();
    for root in roots {
        let report = reconcile_target(&mut target, &root, config, sink)?;
        merge_reconcile_report(&mut combined, report);
    }
    Ok(combined)
}

fn take_invalidation_roots(target: &mut ReconcileTarget<'_>) -> Result<Vec<PathBuf>> {
    let mut pending: Vec<PathBuf> =
        target.take_pending_invalidations()?.into_iter().map(|(path, _)| path).collect();
    pending.sort_by(|left, right| {
        left.components().count().cmp(&right.components().count()).then_with(|| left.cmp(right))
    });
    let mut roots: Vec<PathBuf> = Vec::new();
    for path in pending {
        if roots.iter().any(|root| path.starts_with(root)) {
            continue;
        }
        roots.push(path);
    }

    Ok(roots)
}

fn flush_reconcile_batch(
    target: &mut ReconcileTarget<'_>,
    batch: &mut Vec<ObservationOp>,
    sink: &mut dyn FnMut(&AppliedDelta),
    stats: &mut ApplyStats,
) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }
    let outcome = target.apply(&Observation::from_ops(std::mem::take(batch)))?;
    merge_apply_stats(stats, outcome.stats);
    if let Some(applied) = &outcome.applied {
        sink(applied);
    }
    Ok(())
}

fn merge_apply_stats(total: &mut ApplyStats, addition: ApplyStats) {
    total.inserted += addition.inserted;
    total.updated += addition.updated;
    total.removed += addition.removed;
    total.unchanged += addition.unchanged;
    total.invalidated += addition.invalidated;
    total.stale += addition.stale;
}

fn merge_reconcile_report(total: &mut ReconcileReport, addition: ReconcileReport) {
    total.scan.dirs_read += addition.scan.dirs_read;
    total.scan.entries += addition.scan.entries;
    total.scan.errors.extend(addition.scan.errors);
    merge_apply_stats(&mut total.apply, addition.apply);
}

fn should_descend(
    kind: EntryKind,
    attrs: Attrs,
    parent_depth: usize,
    root_dev: u64,
    config: &ScanConfig,
) -> bool {
    let within_depth = config.max_depth.is_none_or(|max| parent_depth + 1 < max);
    let same_filesystem = !config.one_filesystem || attrs.dev == root_dev || attrs.dev == 0;
    kind.is_dir() && within_depth && same_filesystem
}

fn normalize_subtree(path: &Path) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(Error::PathEscapesRoot(path.to_path_buf()));
            }
        }
    }
    Ok(normalized)
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
    fn cold_scan_establishes_a_baseline_without_change_history() {
        let dir = sample_tree();
        let (index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");

        assert_eq!(index.clock(), crate::Clock::ZERO);
        assert!(index.since(crate::Clock::ZERO).deltas.is_empty());
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
    fn direct_scan_records_the_canonical_root() {
        let dir = sample_tree();
        let aliased = dir.path().join(".");
        let (index, _) = scan_into_index(&aliased, &ScanConfig::default()).expect("scan");

        assert_eq!(index.root_path(), dir.path().canonicalize().expect("canonical root"));
    }

    #[test]
    fn unsupported_symlink_following_is_rejected_on_cold_and_warm_paths() {
        let dir = sample_tree();
        let unsupported = ScanConfig { follow_symlinks: true, ..ScanConfig::default() };
        assert!(matches!(
            scan_into_index(dir.path(), &unsupported),
            Err(Error::UnsupportedScanConfig(_))
        ));

        let (index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        assert!(matches!(
            revalidate(&index, &unsupported, &mut |_| {}),
            Err(Error::UnsupportedScanConfig(_))
        ));
    }

    #[test]
    fn revalidation_uses_the_same_depth_boundary_as_cold_scan() {
        let dir = sample_tree();
        let config = ScanConfig { max_depth: Some(1), ..ScanConfig::default() };
        let (mut index, _) = scan_into_index(dir.path(), &config).expect("scan");
        write_file(&dir.path().join("src/added-after-scan.txt"), b"new");

        let mut observations = Vec::new();
        revalidate(&index, &config, &mut |observation| observations.push(observation))
            .expect("revalidate");
        for observation in &observations {
            index.apply(observation);
        }

        assert!(index.lookup(Path::new("src/added-after-scan.txt")).is_none());
    }

    #[test]
    fn filesystem_boundary_is_part_of_the_shared_descent_policy() {
        let config = ScanConfig { one_filesystem: true, ..ScanConfig::default() };
        let attrs = Attrs { dev: 22, ..Attrs::default() };
        assert!(!should_descend(EntryKind::Dir, attrs, 0, 11, &config));
        assert!(should_descend(EntryKind::Dir, Attrs { dev: 11, ..attrs }, 0, 11, &config,));
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

    #[test]
    fn pending_invalidation_reconciles_the_requested_subtree() {
        let dir = sample_tree();
        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        write_file(&dir.path().join("src/added.rs"), b"new");
        index.apply(&Observation::new(vec![Op::InvalidateSubtree {
            path: PathBuf::from("src"),
            reason: crate::InvalidateReason::Requested,
        }]));
        assert_eq!(index.freshness_at(Path::new("src")), crate::Freshness::Stale);

        let mut applied = Vec::new();
        let report = reconcile_pending(&mut index, &ScanConfig::default(), &mut |delta| {
            applied.push(delta.clone());
        })
        .expect("reconcile pending");

        assert!(report.scan.is_complete());
        assert!(index.lookup(Path::new("src/added.rs")).is_some());
        assert_eq!(index.freshness_at(Path::new("src")), crate::Freshness::Fresh);
        assert!(index.take_pending_invalidations().is_empty());
        assert!(
            applied
                .iter()
                .any(|delta| { delta.ops.iter().any(|op| op.path() == Path::new("src/added.rs")) })
        );
    }

    #[test]
    fn handle_reconciliation_publishes_after_each_delta_is_applied() {
        let dir = sample_tree();
        let (index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        let handle = crate::IndexHandle::new(index);
        let reader = handle.clone();
        write_file(&dir.path().join("added.md"), b"new");

        let mut observed_after_apply = false;
        reconcile_handle(&handle, &ScanConfig::default(), &mut |delta| {
            if delta.ops.iter().any(|op| op.path() == Path::new("added.md")) {
                observed_after_apply =
                    reader.read().expect("read index").lookup(Path::new("added.md")).is_some();
            }
        })
        .expect("reconcile handle");

        assert!(observed_after_apply);
    }

    #[test]
    fn reconciliation_does_not_clear_a_newer_invalidation() {
        let dir = sample_tree();
        let (index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        let handle = crate::IndexHandle::new(index);
        write_file(&dir.path().join("added.md"), b"new");

        let invalidator = handle.clone();
        let mut saw_reconciling = false;
        reconcile_handle(&handle, &ScanConfig::default(), &mut |delta| {
            if delta.ops.iter().any(|op| op.path() == Path::new("added.md")) {
                saw_reconciling =
                    invalidator.read().expect("read").freshness() == crate::Freshness::Reconciling;
                invalidator
                    .apply(&Observation::new(vec![Op::InvalidateSubtree {
                        path: PathBuf::new(),
                        reason: crate::InvalidateReason::WatchOverflow,
                    }]))
                    .expect("new invalidation");
            }
        })
        .expect("reconcile handle");

        assert!(saw_reconciling);
        assert_eq!(handle.read().expect("read").freshness(), crate::Freshness::Stale);
    }

    #[test]
    fn failed_reconciliation_marks_the_scope_partial() {
        let dir = sample_tree();
        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        fs::remove_dir_all(dir.path()).expect("remove root");

        assert!(reconcile(&mut index, &ScanConfig::default(), &mut |_| {}).is_err());
        assert_eq!(index.freshness(), crate::Freshness::Partial);
    }

    #[test]
    fn subtree_reconciliation_rejects_paths_outside_the_root() {
        let dir = sample_tree();
        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");

        assert!(matches!(
            reconcile_subtree(
                &mut index,
                Path::new("../outside"),
                &ScanConfig::default(),
                &mut |_| {},
            ),
            Err(Error::PathEscapesRoot(_))
        ));
        assert_eq!(index.freshness(), crate::Freshness::Fresh);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn scan_and_revalidate_keep_non_utf8_names_distinct() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let dir = tempfile::tempdir().expect("tempdir");
        let first = PathBuf::from(OsString::from_vec(vec![b'n', 0x80]));
        let second = PathBuf::from(OsString::from_vec(vec![b'n', 0x81]));
        write_file(&dir.path().join(&first), b"a");
        write_file(&dir.path().join(&second), b"bb");

        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        assert_eq!(index.total().files, 2);
        assert_eq!(index.total().bytes, 3);

        fs::remove_file(dir.path().join(&first)).expect("remove first");
        let mut observations = Vec::new();
        revalidate(&index, &ScanConfig::default(), &mut |observation| {
            observations.push(observation);
        })
        .expect("revalidate");
        for observation in &observations {
            index.apply(observation);
        }
        assert!(index.lookup(&first).is_none());
        assert!(index.lookup(&second).is_some());
        assert_eq!(index.total().bytes, 2);
    }
}
