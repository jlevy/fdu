//! **fdu** — a fast, incremental file roll-up engine.
//!
//! fdu answers, for any directory in a tree: how big is it, how many files does it hold,
//! what changed most recently, and what kinds of files live in it — hierarchically, for
//! every directory at once, from a single walk.
//!
//! # The shape: three artifacts, one contract
//!
//! 1. **The index** ([`index::Index`]) — the in-memory hierarchical structure: entry
//!    records plus per-directory roll-up state.
//! 2. **The snapshot** ([`snapshot`]) — that index, serialized.
//! 3. **The change contract** ([`types::Observation`] and [`types::AppliedDelta`]) —
//!    producers submit verified observations; the index commits clocked effective
//!    changes.
//!
//! Everything else is a producer of observations or a consumer of applied deltas. The
//! walker establishes a baseline from upsert observations; the reconciler submits the
//! conditional diff between indexed state and reality; the watch layer submits verified,
//! coalesced observations. The index arbitrates them and re-rolls its reducers; a change
//! feed consumes the effective committed deltas.
//!
//! A deliberate consequence: **watching is not tied to the roll-up logic.** The index
//! knows `apply(Observation)` and nothing about filesystem events, so a batch scan, a test
//! feeding synthetic observations, and a live watcher are indistinguishable to it.
//!
//! # Freshness is a ladder, not a set of alternatives
//!
//! [`open`] is the conservative, blocking entry point: it loads a compatible snapshot,
//! reconciles the configured filesystem scope, and only then returns. It does not serve
//! the loaded baseline concurrently. Applications that want that model can own an
//! [`IndexHandle`], call the applying reconciliation APIs, and inspect [`Freshness`]
//! while readers continue between short write batches. With the `watch` feature,
//! [`watch::Watcher::apply_next`] verifies event hints and closes invalidations through
//! subtree reconciliation; neither `open` nor the Python binding starts it implicitly.
//!
//! ```no_run
//! use fdu::{OpenConfig, open};
//! use std::path::Path;
//!
//! let (index, report) = open(Path::new("."), &OpenConfig::default())?;
//! let total = index.total();
//! println!("{} files, {} bytes ({:?})", total.files, total.bytes, report.path_taken);
//! # Ok::<(), fdu::Error>(())
//! ```
//!
//! # Feature flags
//!
//! - `cli` *(default)* — the `fdu` binary and its dependencies. Library consumers should
//!   take `default-features = false`.
//! - `watch` — the OS-native watch layer. Strictly additive: without it everything else
//!   works, just without live updates.

pub mod classify;
pub mod index;
pub mod scan;
pub mod snapshot;
pub mod types;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "watch")]
pub mod watch;

pub use crate::index::{
    ApplyOutcome, ApplyStats, EntryId, ExtTally, Index, IndexHandle, RollUp, Since,
};
pub use crate::scan::{ReconcileReport, ScanConfig, ScanReport};
pub use crate::types::{
    AppliedDelta, Attrs, Clock, EntryKind, Error, Expectation, Fingerprint, Freshness,
    InvalidateReason, Observation, ObservationOp, Op, PathExpectation, PathState, Result,
    ScanScope,
};

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// How to open a tree.
#[derive(Clone, Debug, Default)]
pub struct OpenConfig {
    /// Walk settings.
    pub scan: ScanConfig,
    /// Where the snapshot for this root lives. `None` disables the cache entirely, which
    /// makes every open a cold scan.
    pub cache_path: Option<PathBuf>,
    /// Write the index back to `cache_path` once it is current.
    pub save_on_open: bool,
}

/// Which tier of the freshness ladder an [`open`] actually used.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OpenPath {
    /// No usable snapshot: the tree was walked from scratch.
    ColdScan,
    /// A snapshot was loaded and reconciled against the filesystem.
    WarmRevalidate,
}

/// What [`open`] did.
#[derive(Debug)]
pub struct OpenReport {
    pub path_taken: OpenPath,
    pub scan: ScanReport,
}

impl OpenReport {
    /// Whether every path in the requested scan scope was read successfully.
    pub fn is_complete(&self) -> bool {
        self.scan.is_complete()
    }

    /// Per-path errors that make this result partial.
    pub fn errors(&self) -> &[Error] {
        &self.scan.errors
    }
}

/// Open a tree, using the snapshot cache when one is usable.
///
/// On the warm path the snapshot is loaded and then reconciled against the filesystem
/// before being returned. Errors are represented as partial freshness and the previous
/// complete snapshot is left untouched; callers must inspect [`OpenReport::is_complete`]
/// or [`Index::freshness`] before treating totals as complete.
pub fn open(root: &Path, config: &OpenConfig) -> Result<(Index, OpenReport)> {
    let root = root.canonicalize().map_err(|e| Error::io(root, e))?;

    if let Some(cache_path) = &config.cache_path {
        if let Some(mut index) = snapshot::load(cache_path)? {
            if index.root_path() == root && index.scope() == config.scan.scope() {
                let scan_report = scan::reconcile(&mut index, &config.scan, &mut |_| {})?.scan;
                index.establish_baseline();
                if config.save_on_open && scan_report.is_complete() {
                    snapshot::save(&index, cache_path)?;
                }
                return Ok((
                    index,
                    OpenReport { path_taken: OpenPath::WarmRevalidate, scan: scan_report },
                ));
            }
            // The snapshot describes a different root; treat it as absent.
        }
    }

    let (index, scan_report) = scan::scan_into_index(&root, &config.scan)?;
    if let (Some(cache_path), true, true) =
        (&config.cache_path, config.save_on_open, scan_report.is_complete())
    {
        snapshot::save(&index, cache_path)?;
    }
    Ok((index, OpenReport { path_taken: OpenPath::ColdScan, scan: scan_report }))
}

/// The conventional snapshot location for a root.
///
/// Snapshots are keyed by a hash of the canonical root path under the user cache
/// directory, so two roots never collide and a moved tree simply misses rather than
/// reading another tree's data.
pub fn default_cache_path(root: &Path) -> Option<PathBuf> {
    let canonical = root.canonicalize().ok()?;
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in canonical.as_os_str().as_encoded_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    Some(user_cache_dir()?.join("fdu").join(format!("{hash:016x}.fdu")))
}

fn user_cache_dir() -> Option<PathBuf> {
    if let Some(xdg) = nonempty_env("XDG_CACHE_HOME") {
        return Some(PathBuf::from(xdg));
    }
    platform_cache_dir()
}

fn nonempty_env(name: &str) -> Option<OsString> {
    std::env::var_os(name).filter(|value| !value.is_empty())
}

#[cfg(target_os = "windows")]
fn platform_cache_dir() -> Option<PathBuf> {
    windows_cache_dir(
        nonempty_env("LOCALAPPDATA"),
        nonempty_env("USERPROFILE"),
        nonempty_env("HOME"),
    )
}

#[cfg(target_os = "windows")]
fn windows_cache_dir(
    local_app_data: Option<OsString>,
    user_profile: Option<OsString>,
    home: Option<OsString>,
) -> Option<PathBuf> {
    local_app_data
        .map(PathBuf::from)
        .or_else(|| user_profile.map(|path| PathBuf::from(path).join("AppData").join("Local")))
        .or_else(|| home.map(|path| PathBuf::from(path).join(".cache")))
}

#[cfg(target_os = "macos")]
fn platform_cache_dir() -> Option<PathBuf> {
    Some(PathBuf::from(nonempty_env("HOME")?).join("Library").join("Caches"))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn platform_cache_dir() -> Option<PathBuf> {
    Some(PathBuf::from(nonempty_env("HOME")?).join(".cache"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_file(path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, contents).expect("write");
    }

    #[test]
    fn open_without_a_cache_always_scans_cold() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(&dir.path().join("a.txt"), b"hello");

        let (index, report) = open(dir.path(), &OpenConfig::default()).expect("open");
        assert_eq!(report.path_taken, OpenPath::ColdScan);
        assert_eq!(index.total().files, 1);
    }

    #[test]
    fn second_open_takes_the_warm_path_and_stays_correct() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        write_file(&dir.path().join("a.txt"), b"hello");
        write_file(&dir.path().join("src/main.rs"), b"fn main() {}");

        let config = OpenConfig {
            cache_path: Some(cache.path().join("snap.fdu")),
            save_on_open: true,
            ..OpenConfig::default()
        };

        let (first, first_report) = open(dir.path(), &config).expect("cold open");
        assert_eq!(first_report.path_taken, OpenPath::ColdScan);
        assert_eq!(first.total().files, 2);

        // Change the tree between opens: the warm path must notice.
        write_file(&dir.path().join("added.md"), b"new");
        fs::remove_file(dir.path().join("a.txt")).expect("remove");

        let (second, second_report) = open(dir.path(), &config).expect("warm open");
        assert_eq!(second_report.path_taken, OpenPath::WarmRevalidate);
        assert_eq!(second.total().files, 2);
        assert!(second.lookup(Path::new("added.md")).is_some());
        assert!(second.lookup(Path::new("a.txt")).is_none());
    }

    #[test]
    fn a_snapshot_from_another_root_is_ignored() {
        let a = tempfile::tempdir().expect("tempdir a");
        let b = tempfile::tempdir().expect("tempdir b");
        let cache = tempfile::tempdir().expect("cache dir");
        write_file(&a.path().join("only-in-a.txt"), b"x");
        write_file(&b.path().join("only-in-b.txt"), b"y");

        let cache_path = cache.path().join("snap.fdu");
        let config = OpenConfig {
            cache_path: Some(cache_path),
            save_on_open: true,
            ..OpenConfig::default()
        };

        open(a.path(), &config).expect("open a");
        let (index, report) = open(b.path(), &config).expect("open b");

        assert_eq!(report.path_taken, OpenPath::ColdScan);
        assert!(index.lookup(Path::new("only-in-b.txt")).is_some());
        assert!(index.lookup(Path::new("only-in-a.txt")).is_none());
    }

    #[test]
    fn snapshot_scope_mismatch_forces_a_cold_scan() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        write_file(&dir.path().join("top.txt"), b"top");
        write_file(&dir.path().join("deep/nested.txt"), b"nested");

        let cache_path = cache.path().join("snap.fdu");
        let full = OpenConfig {
            cache_path: Some(cache_path.clone()),
            save_on_open: true,
            ..OpenConfig::default()
        };
        open(dir.path(), &full).expect("full open");

        let shallow = OpenConfig {
            scan: ScanConfig { max_depth: Some(1), ..ScanConfig::default() },
            cache_path: Some(cache_path),
            save_on_open: false,
        };
        let (index, report) = open(dir.path(), &shallow).expect("shallow open");

        assert_eq!(report.path_taken, OpenPath::ColdScan);
        assert!(index.lookup(Path::new("deep")).is_some());
        assert!(index.lookup(Path::new("deep/nested.txt")).is_none());
    }

    #[test]
    fn operational_batch_size_does_not_invalidate_a_snapshot() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        write_file(&dir.path().join("a.txt"), b"a");
        let cache_path = cache.path().join("snap.fdu");

        let first = OpenConfig {
            scan: ScanConfig { batch_size: 1, ..ScanConfig::default() },
            cache_path: Some(cache_path.clone()),
            save_on_open: true,
        };
        open(dir.path(), &first).expect("first open");

        let second = OpenConfig {
            scan: ScanConfig { batch_size: 17, ..ScanConfig::default() },
            cache_path: Some(cache_path),
            save_on_open: false,
        };
        let (_, report) = open(dir.path(), &second).expect("second open");
        assert_eq!(report.path_taken, OpenPath::WarmRevalidate);
    }

    #[test]
    fn cache_paths_differ_per_root() {
        let a = tempfile::tempdir().expect("tempdir a");
        let b = tempfile::tempdir().expect("tempdir b");
        let (Some(pa), Some(pb)) = (default_cache_path(a.path()), default_cache_path(b.path()))
        else {
            return; // No HOME in this environment; nothing to assert.
        };
        assert_ne!(pa, pb);
        assert_eq!(pa, default_cache_path(a.path()).expect("stable"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_cache_discovery_prefers_native_locations() {
        let local = OsString::from(r"C:\Users\tester\AppData\Local");
        let profile = OsString::from(r"D:\Profile");
        let home = OsString::from(r"E:\Home");

        assert_eq!(
            windows_cache_dir(Some(local.clone()), Some(profile.clone()), Some(home.clone())),
            Some(PathBuf::from(local))
        );
        assert_eq!(
            windows_cache_dir(None, Some(profile.clone()), Some(home.clone())),
            Some(PathBuf::from(profile).join("AppData").join("Local"))
        );
        assert_eq!(
            windows_cache_dir(None, None, Some(home.clone())),
            Some(PathBuf::from(home).join(".cache"))
        );
        assert_eq!(windows_cache_dir(None, None, None), None);
    }
}
