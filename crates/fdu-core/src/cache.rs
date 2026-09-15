//! Inspecting and clearing the snapshot cache.
//!
//! Snapshot files are named by a hash of their root, which keeps two trees from
//! colliding but leaves a directory of opaque names with no way to answer "which tree is
//! this?". These functions read each file's bounded header to recover that mapping, so
//! the cache can be inspected and cleared without guesswork.
//!
//! A file is one of fdu's snapshots only when its contents begin with the snapshot magic;
//! a name alone proves nothing. A snapshot this build cannot serve is still fdu's: an
//! older or newer format, another engine fingerprint, or a truncated write is reported as
//! stale and cleared like a current one. The engine fingerprint mixes in the crate
//! version, so without that every release would strand every snapshot the one before it
//! wrote. Anything else is reported as unrecognized and never deleted.

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::engine_contract::{Error, Result, ScanScope};
use crate::snapshot::{self, Identity};

/// Hex digits in a snapshot's file name, one per nibble of the 64-bit root hash.
const SNAPSHOT_NAME_HEX_DIGITS: usize = 16;

/// The suffix every snapshot name in the cache directory ends with.
const SNAPSHOT_NAME_SUFFIX: &str = ".fdu";

/// What is known about one file in the cache directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheStatus {
    /// Where the file lives.
    pub path: PathBuf,
    /// Size on disk, in bytes.
    pub bytes: u64,
    /// Size of the content sidecar fdu wrote beside this snapshot, current or stale.
    pub content_bytes: Option<u64>,
    /// What the file is, as far as this build can tell.
    pub state: CacheState,
}

impl CacheStatus {
    /// The header of a snapshot this build can serve.
    pub fn snapshot(&self) -> Option<&SnapshotInfo> {
        match &self.state {
            CacheState::Current(info) => Some(info),
            CacheState::Stale(_) | CacheState::Unrecognized | CacheState::Absent => None,
        }
    }

    /// Whether this file is one of fdu's snapshots, current or stale, which is exactly
    /// what clearing removes.
    pub fn is_fdu_snapshot(&self) -> bool {
        matches!(self.state, CacheState::Current(_) | CacheState::Stale(_))
    }
}

/// What a path in the cache holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CacheState {
    /// A snapshot this build reads and can serve.
    Current(SnapshotInfo),
    /// One of fdu's snapshots that this build cannot serve. Clearing removes it.
    Stale(StaleReason),
    /// A file fdu cannot identify as one of its snapshots, which clearing never removes.
    ///
    /// Its contents lack the snapshot magic, or it is not a regular file (a symbolic link
    /// is never followed), or it was found by listing a directory under a name the cache
    /// never gives a snapshot.
    Unrecognized,
    /// Nothing is there. Only a status for one path can be absent; a listing reports the
    /// files it found.
    Absent,
}

impl CacheState {
    /// Every label [`CacheState::label`] returns, in declaration order.
    pub const LABELS: [&'static str; 4] = ["current", "stale", "unrecognized", "absent"];

    /// The label machine output carries.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Current(_) => "current",
            Self::Stale(_) => "stale",
            Self::Unrecognized => "unrecognized",
            Self::Absent => "absent",
        }
    }
}

/// Why a snapshot fdu wrote cannot be served by this build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaleReason {
    /// Written in an earlier snapshot format.
    OlderFormat {
        /// The format version its header names.
        version: u32,
    },
    /// Written in a later snapshot format, by a newer fdu.
    NewerFormat {
        /// The format version its header names.
        version: u32,
    },
    /// The format matches but the engine fingerprint does not: another fdu version, or
    /// other classification rules.
    OtherEngine,
    /// The format and engine match but the header cannot be read: a truncated or corrupt
    /// file, or one written with another platform's path encoding.
    Unreadable,
}

impl StaleReason {
    /// Every label [`StaleReason::label`] returns, in declaration order.
    pub const LABELS: [&'static str; 4] =
        ["older_format", "newer_format", "other_engine", "unreadable"];

    /// The label machine output carries.
    pub fn label(self) -> &'static str {
        match self {
            Self::OlderFormat { .. } => "older_format",
            Self::NewerFormat { .. } => "newer_format",
            Self::OtherEngine => "other_engine",
            Self::Unreadable => "unreadable",
        }
    }

    /// The format version the header names, when that version is why it is stale.
    pub fn format_version(self) -> Option<u32> {
        match self {
            Self::OlderFormat { version } | Self::NewerFormat { version } => Some(version),
            Self::OtherEngine | Self::Unreadable => None,
        }
    }
}

/// Which snapshots a cache-lifecycle request covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheScope {
    /// The one file a root's snapshot occupies.
    Root,
    /// Every file in the cache directory.
    All,
}

impl CacheScope {
    /// Every label [`CacheScope::label`] returns, in declaration order.
    pub const LABELS: [&'static str; 2] = ["root", "all"];

    /// The label the command line and the Python package accept.
    pub fn label(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::All => "all",
        }
    }

    /// Parse a label, ignoring case and surrounding whitespace.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "root" => Some(Self::Root),
            "all" => Some(Self::All),
            _ => None,
        }
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

/// The name the cache gives the snapshot of a root with this hash.
pub(crate) fn snapshot_file_name(root_hash: u64) -> String {
    format!("{root_hash:0SNAPSHOT_NAME_HEX_DIGITS$x}{SNAPSHOT_NAME_SUFFIX}")
}

/// Whether a name is one [`snapshot_file_name`] produces.
fn is_snapshot_file_name(name: &OsStr) -> bool {
    let bytes = name.as_encoded_bytes();
    bytes.len() == SNAPSHOT_NAME_HEX_DIGITS + SNAPSHOT_NAME_SUFFIX.len()
        && bytes.ends_with(SNAPSHOT_NAME_SUFFIX.as_bytes())
        && bytes[..SNAPSHOT_NAME_HEX_DIGITS]
            .iter()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

/// Read one cache file's status without materializing its index.
///
/// The file is identified by its contents alone, because the caller named it. A symbolic
/// link at `path` is reported unrecognized rather than followed.
pub fn cache_status(path: &Path) -> Result<CacheStatus> {
    Ok(status_at(path)?.unwrap_or_else(|| CacheStatus {
        path: path.to_path_buf(),
        bytes: 0,
        content_bytes: None,
        state: CacheState::Absent,
    }))
}

/// The status of whatever is at `path`, or `None` when nothing is.
///
/// Only a regular file is opened. A symbolic link, directory, or special file is described
/// from its own metadata, so nothing here follows a link out of the cache directory or
/// blocks on opening a FIFO.
fn status_at(path: &Path) -> Result<Option<CacheStatus>> {
    let Some(metadata) = present(fs::symlink_metadata(path), path)? else {
        return Ok(None);
    };
    if !metadata.file_type().is_file() {
        return Ok(Some(unrecognized(path, metadata.len())));
    }
    let state = match snapshot::identify(path)? {
        None => return Ok(None),
        Some(Identity::Foreign) => return Ok(Some(unrecognized(path, metadata.len()))),
        Some(Identity::Current(info)) => CacheState::Current(info),
        Some(Identity::Stale(reason)) => CacheState::Stale(reason),
    };
    let content_bytes =
        crate::content::content_sidecar_bytes(&crate::content::content_cache_path(path))?;
    Ok(Some(CacheStatus { path: path.to_path_buf(), bytes: metadata.len(), content_bytes, state }))
}

fn unrecognized(path: &Path, bytes: u64) -> CacheStatus {
    CacheStatus {
        path: path.to_path_buf(),
        bytes,
        content_bytes: None,
        state: CacheState::Unrecognized,
    }
}

/// Keep a filesystem answer, reading "not found" as nothing there rather than an error.
///
/// A path can be absent before the call or removed by another process during it, and a
/// status or clear that races a concurrent clear must not fail because of that.
fn present<T>(result: io::Result<T>, path: &Path) -> Result<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Error::io(path, error)),
    }
}

/// Enumerate every file in the cache directory holding fdu snapshots.
///
/// Unrecognized files are listed rather than hidden: a directory this code will not
/// delete from is a directory the caller should still be able to see. A file counts as a
/// snapshot only under a name the cache gives snapshots and with the snapshot magic, so
/// neither a stray copy under another name nor another program's file under a snapshot's
/// name is mistaken for one. Symbolic links are listed, never followed.
pub fn list_caches(cache_dir: &Path) -> Result<Vec<CacheStatus>> {
    let Some(entries) = present(fs::read_dir(cache_dir), cache_dir)? else {
        // An absent cache directory is an empty cache, not an error.
        return Ok(Vec::new());
    };

    let mut found = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| Error::io(cache_dir, error))?;
        let path = entry.path();
        // `DirEntry::file_type` describes the entry itself, never a link's target.
        let Some(file_type) = present(entry.file_type(), &path)? else { continue };
        if file_type.is_dir() {
            continue;
        }
        let status = if file_type.is_file() && is_snapshot_file_name(&entry.file_name()) {
            status_at(&path)?
        } else {
            present(fs::symlink_metadata(&path), &path)?
                .map(|metadata| unrecognized(&path, metadata.len()))
        };
        found.extend(status);
    }
    let paired_sidecars = found
        .iter()
        .filter(|status| status.is_fdu_snapshot() && status.content_bytes.is_some())
        .map(|status| crate::content::content_cache_path(&status.path))
        .collect::<std::collections::BTreeSet<_>>();
    found.retain(|status| !paired_sidecars.contains(&status.path));
    // Deterministic order, so two runs of a status command agree.
    found.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(found)
}

/// Remove one snapshot, current or stale, with its content sidecar.
///
/// Returns whether this call removed it. Idempotent: clearing an absent cache succeeds,
/// and so does clearing one another process removed first. A file that is not one of
/// fdu's snapshots is left in place and reported as not removed.
pub fn clear_cache(path: &Path) -> Result<bool> {
    let Some(status) = status_at(path)? else { return Ok(false) };
    if !status.is_fdu_snapshot() {
        // Refusing to delete what this code cannot identify is the whole safety property:
        // the cache directory may hold files fdu did not write.
        return Ok(false);
    }
    // `remove_file` unlinks a symbolic link rather than its target, so a link swapped in
    // after the check above can cost that link and never the file it points at.
    if !remove_present(path)? {
        return Ok(false);
    }
    let content_path = crate::content::content_cache_path(path);
    if crate::content::content_sidecar_bytes(&content_path)?.is_some() {
        remove_present(&content_path)?;
    }
    Ok(true)
}

/// Remove a file, returning whether this call removed it.
fn remove_present(path: &Path) -> Result<bool> {
    Ok(present(fs::remove_file(path), path)?.is_some())
}

/// Remove every fdu snapshot in a cache directory, current or stale, leaving anything
/// else alone.
///
/// Returns how many snapshots were removed. Each file is identified again immediately
/// before it is removed, so one replaced since the listing by something that is not a
/// snapshot survives.
pub fn clear_all_caches(cache_dir: &Path) -> Result<usize> {
    let mut removed = 0;
    for status in list_caches(cache_dir)? {
        if status.is_fdu_snapshot() && clear_cache(&status.path)? {
            removed += 1;
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CachePolicy, OpenConfig, open};

    /// Byte offset of the format version: it follows the eight-byte magic.
    const VERSION_OFFSET: usize = 8;

    /// Byte offset of the engine fingerprint: it follows the four-byte format version.
    const FINGERPRINT_OFFSET: usize = VERSION_OFFSET + 4;

    /// A name the cache gives snapshots, distinct per `seed`.
    fn layout_name(seed: u64) -> String {
        snapshot_file_name(seed)
    }

    fn seed(tree: &Path, snapshot_path: &Path) {
        std::fs::write(tree.join("a.txt"), b"hello").expect("write");
        let config = OpenConfig {
            cache_path: Some(snapshot_path.to_path_buf()),
            policy: CachePolicy::Auto,
            ..OpenConfig::default()
        };
        open(tree, &config).expect("seed");
    }

    /// Copy the snapshot at `from` to `to`, overwriting its format version.
    fn with_format_version(from: &Path, to: &Path, version: impl FnOnce(u32) -> u32) {
        let mut bytes = std::fs::read(from).expect("read snapshot");
        let range = VERSION_OFFSET..FINGERPRINT_OFFSET;
        let current = u32::from_le_bytes(bytes[range.clone()].try_into().expect("four bytes"));
        bytes[range].copy_from_slice(&version(current).to_le_bytes());
        std::fs::write(to, bytes).expect("write stale snapshot");
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
        assert!(status.bytes > 0);
        let info = status.snapshot().expect("header");
        assert_eq!(info.root, tree.path().canonicalize().expect("canonical"));
        assert_eq!(info.entries, 2, "the root plus one file");
    }

    #[test]
    fn content_sidecar_is_reported_and_cleared_with_its_snapshot() {
        let tree = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache");
        let path = cache.path().join(layout_name(1));
        std::fs::write(tree.path().join("notes.md"), b"one two\n").expect("write");
        let config = OpenConfig {
            cache_path: Some(path.clone()),
            policy: CachePolicy::Auto,
            analysis: crate::content::AnalysisRequest {
                profile: crate::content::AnalysisSet::NONE.with_lines(),
                ..crate::content::AnalysisRequest::default()
            },
            ..OpenConfig::default()
        };
        open(tree.path(), &config).expect("seed analyzed cache");

        let listed = list_caches(cache.path()).expect("list");
        assert_eq!(listed.len(), 1, "a sidecar is grouped with its snapshot");
        assert!(listed[0].content_bytes.is_some());
        assert!(clear_cache(&path).expect("clear"));
        assert!(!path.exists());
        assert!(!crate::content::content_cache_path(&path).exists());
    }

    #[test]
    fn stale_snapshots_are_reported_beside_a_current_one_and_all_are_cleared() {
        // Every release changes the engine fingerprint and some change the format, so a
        // cache that only recognised what this build writes would strand everything an
        // earlier build left. Each way of being stale is reported with its reason.
        let tree = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache");
        let current = cache.path().join(layout_name(1));
        seed(tree.path(), &current);

        let older = cache.path().join(layout_name(2));
        with_format_version(&current, &older, |version| version - 1);
        let newer = cache.path().join(layout_name(3));
        with_format_version(&current, &newer, |version| version + 1);
        let other_engine = cache.path().join(layout_name(4));
        let mut bytes = std::fs::read(&current).expect("read");
        for byte in &mut bytes[FINGERPRINT_OFFSET..FINGERPRINT_OFFSET + 8] {
            *byte = !*byte;
        }
        std::fs::write(&other_engine, &bytes).expect("write");
        let truncated = cache.path().join(layout_name(5));
        let full = std::fs::read(&current).expect("read");
        std::fs::write(&truncated, &full[..full.len() - 1]).expect("truncate");
        // Hand-built rather than copied: only the magic and a lower version, the least an
        // earlier format is guaranteed to share with this one.
        let hand_built = cache.path().join(layout_name(6));
        let mut prologue = b"FDUSNAP\x00".to_vec();
        prologue.extend_from_slice(&1_u32.to_le_bytes());
        std::fs::write(&hand_built, &prologue).expect("write");

        let current_version = u32::from_le_bytes(
            full[VERSION_OFFSET..FINGERPRINT_OFFSET].try_into().expect("four bytes"),
        );
        let states = list_caches(cache.path())
            .expect("list")
            .into_iter()
            .map(|status| (status.path, status.state))
            .collect::<Vec<_>>();
        assert_eq!(states.len(), 6);
        assert!(matches!(states[0].1, CacheState::Current(_)));
        assert_eq!(
            states[1..],
            [
                (
                    older,
                    CacheState::Stale(StaleReason::OlderFormat { version: current_version - 1 })
                ),
                (
                    newer,
                    CacheState::Stale(StaleReason::NewerFormat { version: current_version + 1 })
                ),
                (other_engine, CacheState::Stale(StaleReason::OtherEngine)),
                (truncated.clone(), CacheState::Stale(StaleReason::Unreadable)),
                (hand_built, CacheState::Stale(StaleReason::OlderFormat { version: 1 })),
            ]
        );

        assert!(clear_cache(&truncated).expect("clear one"), "a truncated snapshot is still fdu's");
        assert_eq!(clear_all_caches(cache.path()).expect("clear"), 5);
        assert!(list_caches(cache.path()).expect("list").is_empty());
    }

    #[test]
    fn a_stale_snapshot_takes_its_content_sidecar_with_it() {
        let tree = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache");
        let path = cache.path().join(layout_name(1));
        std::fs::write(tree.path().join("notes.md"), b"one two\n").expect("write");
        let config = OpenConfig {
            cache_path: Some(path.clone()),
            policy: CachePolicy::Auto,
            analysis: crate::content::AnalysisRequest {
                profile: crate::content::AnalysisSet::NONE.with_lines(),
                ..crate::content::AnalysisRequest::default()
            },
            ..OpenConfig::default()
        };
        open(tree.path(), &config).expect("seed analyzed cache");
        with_format_version(&path, &path, |version| version - 1);

        let listed = list_caches(cache.path()).expect("list");
        assert_eq!(listed.len(), 1, "a stale snapshot's sidecar is grouped with it");
        assert!(matches!(listed[0].state, CacheState::Stale(StaleReason::OlderFormat { .. })));
        assert!(listed[0].content_bytes.is_some());
        assert_eq!(clear_all_caches(cache.path()).expect("clear"), 1);
        assert!(!crate::content::content_cache_path(&path).exists());
    }

    #[test]
    fn unrecognized_files_are_listed_but_never_removed() {
        let tree = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache");
        let path = cache.path().join(layout_name(1));
        seed(tree.path(), &path);
        let foreign = cache.path().join("notes.txt");
        std::fs::write(&foreign, b"not a snapshot").expect("write");
        // Another program's file under a snapshot's name: the name proves nothing.
        let impostor = cache.path().join(layout_name(2));
        std::fs::write(&impostor, b"FDUSNAQ and more bytes").expect("write");
        // One of fdu's snapshots under a name the cache never gives one, such as a backup
        // someone made: a listing cannot tell it from a user's own file.
        let renamed = cache.path().join("backup.fdu");
        std::fs::copy(&path, &renamed).expect("copy");

        let listed = list_caches(cache.path()).expect("list");
        assert_eq!(
            listed
                .iter()
                .map(|status| (status.path.clone(), status.state.label()))
                .collect::<Vec<_>>(),
            [
                (path.clone(), "current"),
                (impostor.clone(), "unrecognized"),
                (renamed.clone(), "unrecognized"),
                (foreign.clone(), "unrecognized"),
            ]
        );
        assert_eq!(listed[3].bytes, 14, "an unrecognized file still reports its size");

        assert!(!clear_cache(&impostor).expect("clear"));
        assert_eq!(clear_all_caches(cache.path()).expect("clear"), 1);
        for survivor in [&foreign, &impostor, &renamed] {
            assert!(survivor.exists(), "{} must survive", survivor.display());
        }
        assert_eq!(list_caches(cache.path()).expect("list").len(), 3);
    }

    #[cfg(unix)]
    #[test]
    fn a_symbolic_link_in_the_cache_is_never_followed_or_removed() {
        // A link named like a snapshot and pointing at a real snapshot outside the cache
        // directory: following it would report, and then delete, what it points at.
        let tree = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache");
        let outside = tempfile::tempdir().expect("outside");
        let target = outside.path().join(layout_name(9));
        seed(tree.path(), &target);
        let link = cache.path().join(layout_name(1));
        std::os::unix::fs::symlink(&target, &link).expect("symlink");

        let listed = list_caches(cache.path()).expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].state, CacheState::Unrecognized);
        assert_eq!(cache_status(&link).expect("status").state, CacheState::Unrecognized);

        assert_eq!(clear_all_caches(cache.path()).expect("clear"), 0);
        assert!(!clear_cache(&link).expect("clear"));
        assert!(std::fs::symlink_metadata(&link).is_ok(), "the link stays");
        assert!(
            cache_status(&target).expect("status").snapshot().is_some(),
            "and so does its target"
        );
    }

    #[test]
    fn a_file_that_vanishes_before_removal_is_not_an_error() {
        // What a concurrent clear leaves this one to find: the file was identified, then
        // someone else removed it.
        let cache = tempfile::tempdir().expect("cache");
        let gone = cache.path().join(layout_name(1));
        assert!(!remove_present(&gone).expect("remove"));
        assert_eq!(status_at(&gone).expect("status"), None);
    }

    #[test]
    fn clearing_is_idempotent_and_an_absent_directory_is_empty() {
        let cache = tempfile::tempdir().expect("cache");
        let missing = cache.path().join("does-not-exist");
        assert!(list_caches(&missing).expect("list").is_empty());
        assert_eq!(clear_all_caches(&missing).expect("clear"), 0);
        let absent = missing.join(layout_name(1));
        assert!(!clear_cache(&absent).expect("clear"));
        assert_eq!(cache_status(&absent).expect("status").state, CacheState::Absent);
    }

    #[test]
    fn the_layout_names_what_default_cache_path_produces() {
        let tree = tempfile::tempdir().expect("tempdir");
        if let Some(path) = crate::default_cache_path(tree.path()) {
            assert!(is_snapshot_file_name(path.file_name().expect("name")), "{}", path.display());
        }
        assert!(!is_snapshot_file_name(OsStr::new("0123456789ABCDEF.fdu")));
        assert!(!is_snapshot_file_name(OsStr::new("0123456789abcdef.fdu.content")));
        assert!(!is_snapshot_file_name(OsStr::new(".0123456789abcdef.fdu.tmp.1.2.3")));
    }

    #[test]
    fn labels_list_every_variant_in_order() {
        let states = [
            CacheState::Current(SnapshotInfo {
                root: PathBuf::new(),
                scope: crate::test_support::not_observing_controls(),
                entries: 1,
            }),
            CacheState::Stale(StaleReason::Unreadable),
            CacheState::Unrecognized,
            CacheState::Absent,
        ];
        assert_eq!(states.iter().map(CacheState::label).collect::<Vec<_>>(), CacheState::LABELS);
        let reasons = [
            StaleReason::OlderFormat { version: 1 },
            StaleReason::NewerFormat { version: 9 },
            StaleReason::OtherEngine,
            StaleReason::Unreadable,
        ];
        assert_eq!(reasons.map(StaleReason::label), StaleReason::LABELS);
        assert_eq!([CacheScope::Root, CacheScope::All].map(CacheScope::label), CacheScope::LABELS);
        assert_eq!(
            CacheScope::LABELS.map(CacheScope::parse),
            [Some(CacheScope::Root), Some(CacheScope::All)]
        );
    }
}
