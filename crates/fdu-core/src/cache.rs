//! Inspecting and clearing the snapshot cache.
//!
//! Snapshot files are named by a hash of their root, which keeps two trees from
//! colliding but leaves a directory of opaque names with no way to answer "which tree is
//! this?". These functions read each file's bounded header to recover that mapping, so
//! the cache can be inspected and cleared without guesswork.
//!
//! A file is one of fdu's snapshots only when its contents begin with the snapshot magic;
//! a name alone proves nothing. A snapshot this build cannot serve is still fdu's: an
//! older or newer format, another engine fingerprint, or a header this build cannot read
//! is reported as stale and cleared like a current one. The engine fingerprint mixes in
//! the crate version, so without that every release would strand every snapshot the one
//! before it wrote.
//!
//! Two more kinds of file here are fdu's without being a snapshot in place: a staging file
//! a killed writer never renamed, and a content sidecar whose snapshot is gone. Both are
//! reported as leftovers, and clearing the directory reclaims them under rules that keep a
//! clear from racing a live writer or discarding a sidecar a snapshot still wants.
//! Anything else is reported as unrecognized and never deleted.

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::engine_contract::{Error, Result, ScanScope};
use crate::snapshot::{self, Identity};
use crate::stored_state::{ContentTierIdentity, SnapshotIdentity};

/// Hex digits in a snapshot's file name, one per nibble of the 64-bit root hash.
const SNAPSHOT_NAME_HEX_DIGITS: usize = 16;

/// The suffix every snapshot name in the cache directory ends with.
const SNAPSHOT_NAME_SUFFIX: &str = ".fdu";

/// What a content sidecar's name adds to its snapshot's, per `content_cache_path`.
const CONTENT_NAME_SUFFIX: &str = ".content";

/// What a staging name inserts between its target's name and the discriminator, per
/// `snapshot::temp_name`.
const TEMP_NAME_INFIX: &str = ".tmp.";

/// What is known about one file in the cache directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheStatus {
    /// Where the file lives.
    pub path: PathBuf,
    /// Size on disk, in bytes.
    pub bytes: u64,
    /// The content sidecar fdu wrote beside this snapshot, current or stale, when there is
    /// one.
    pub content: Option<ContentStatus>,
    /// What the file is, as far as this build can tell.
    pub state: CacheState,
}

impl CacheStatus {
    /// The header of a snapshot this build can serve.
    pub fn snapshot(&self) -> Option<&SnapshotInfo> {
        match &self.state {
            CacheState::Current(info) => Some(info),
            CacheState::Stale(_)
            | CacheState::Leftover(_)
            | CacheState::Unrecognized
            | CacheState::Absent => None,
        }
    }

    /// Whether this file is one of fdu's snapshots, current or stale, which is exactly
    /// what clearing removes.
    pub fn is_fdu_snapshot(&self) -> bool {
        matches!(self.state, CacheState::Current(_) | CacheState::Stale(_))
    }

    /// Size of the content sidecar beside this snapshot, current or stale.
    pub fn content_bytes(&self) -> Option<u64> {
        self.content.as_ref().map(|content| content.bytes)
    }
}

/// What is known about the content sidecar beside a snapshot.
///
/// Whether a sidecar is there is decided by its magic, as for a snapshot, so a sidecar this
/// build cannot serve is still reported and cleared with its snapshot. Whether it is
/// current is decided by its own header: its format version and engine fingerprint.
/// Whether it pairs with the snapshot beside it is a question of identity equality, which
/// loading asks and status reports by carrying both identities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentStatus {
    /// Size on disk, in bytes.
    pub bytes: u64,
    /// What the sidecar is, as far as this build can tell.
    pub state: ContentState,
}

/// What a content sidecar beside a snapshot holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContentState {
    /// A sidecar this build reads.
    Current(ContentInfo),
    /// One of fdu's sidecars that this build cannot serve: another format or engine, or a
    /// header this build cannot read.
    Stale(StaleReason),
}

impl ContentState {
    /// Every label a content sidecar's state can carry, in declaration order.
    ///
    /// The two a sidecar can be, which is narrower than [`CacheState::LABELS`]: a sidecar
    /// is never leftover, unrecognized, or absent under its own status, because it is
    /// reported only where one was found beside a snapshot.
    pub const LABELS: [&'static str; 2] = ["current", "stale"];

    /// The label machine output carries, one of [`Self::LABELS`].
    pub fn label(&self) -> &'static str {
        match self {
            Self::Current(_) => "current",
            Self::Stale(_) => "stale",
        }
    }
}

/// The header facts that identify a content sidecar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentInfo {
    /// The identity of the content tier it holds.
    pub identity: ContentTierIdentity,
    /// How many file records it holds.
    pub records: u64,
}

/// What a path in the cache holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CacheState {
    /// A snapshot this build reads and can serve.
    Current(SnapshotInfo),
    /// One of fdu's snapshots that this build cannot serve. Clearing removes it.
    Stale(StaleReason),
    /// A file fdu wrote that is not a snapshot in place: a staging file a killed writer
    /// never renamed, or a content sidecar whose snapshot is gone.
    ///
    /// Clearing the whole directory reclaims it, under the rules on [`LeftoverKind`].
    Leftover(LeftoverKind),
    /// A file fdu cannot identify as one of its own, which clearing never removes.
    ///
    /// Its contents lack the magic its name implies, or it is not a regular file (a
    /// symbolic link is never followed, and a directory is never descended into), or it
    /// was found by listing a directory under a name the cache gives nothing.
    Unrecognized,
    /// Nothing is there. Only a status for one path can be absent; a listing reports the
    /// files it found.
    Absent,
}

impl CacheState {
    /// Every label [`CacheState::label`] returns, in declaration order.
    pub const LABELS: [&'static str; 5] =
        ["current", "stale", "leftover", "unrecognized", "absent"];

    /// The label machine output carries.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Current(_) => "current",
            Self::Stale(_) => "stale",
            Self::Leftover(_) => "leftover",
            Self::Unrecognized => "unrecognized",
            Self::Absent => "absent",
        }
    }
}

/// Which of fdu's own files a leftover is.
///
/// Both are named by fdu and carry one of fdu's magics, and both are reclaimed only by
/// clearing the whole directory: a root's clear reaches the one path that root's snapshot
/// occupies, and neither of these is at that path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeftoverKind {
    /// The staging file a killed writer left beside its target, never renamed into place.
    ///
    /// Removed only once it is older than the age at which a later writer would reap it,
    /// so clearing can never take a file a running writer still holds.
    StagingTemporary,
    /// A content sidecar whose snapshot is gone.
    ///
    /// Removed only when no snapshot for it exists after the clear, so a sidecar a later
    /// analyzed scan could still reuse stays with the snapshot it belongs to. Orphaned is
    /// what a listing shows: a sidecar whose snapshot is present is grouped with that
    /// snapshot and never listed on its own.
    OrphanedContent,
}

impl LeftoverKind {
    /// Every label [`LeftoverKind::label`] returns, in declaration order.
    pub const LABELS: [&'static str; 2] = ["staging_temporary", "orphaned_content"];

    /// The label machine output carries.
    pub fn label(self) -> &'static str {
        match self {
            Self::StagingTemporary => "staging_temporary",
            Self::OrphanedContent => "orphaned_content",
        }
    }
}

/// What one clear removed.
///
/// Two counts rather than one total: "cleared 4 snapshots" and "reclaimed 2 files fdu left
/// behind" are different facts, and a destructive command that reports one number for both
/// would understate what it did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClearSummary {
    /// Snapshots removed, current or stale, each with its content sidecar.
    pub snapshots: usize,
    /// Leftover files reclaimed.
    pub leftovers: usize,
}

impl ClearSummary {
    /// Whether this clear removed nothing at all.
    pub fn is_empty(self) -> bool {
        self.snapshots == 0 && self.leftovers == 0
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
    /// The identity of every tier it holds.
    pub identity: SnapshotIdentity,
    /// How many entries it holds.
    pub entries: u64,
}

impl SnapshotInfo {
    /// The scan scope an index loaded from it records.
    pub fn scope(&self) -> ScanScope {
        self.identity.scan_scope()
    }
}

/// The name the cache gives the snapshot of a root with this hash.
pub(crate) fn snapshot_file_name(root_hash: u64) -> String {
    format!("{root_hash:0SNAPSHOT_NAME_HEX_DIGITS$x}{SNAPSHOT_NAME_SUFFIX}")
}

/// Whether a name is one [`snapshot_file_name`] produces.
///
/// On bytes rather than on an [`OsStr`], because a sidecar's and a staging file's names
/// each *contain* one, and rebuilding an `OsStr` from a slice of one needs `unsafe`.
/// [`name_shape`] is the entry point; this is what all four of its answers are built from.
fn is_snapshot_name_bytes(bytes: &[u8]) -> bool {
    bytes.len() == SNAPSHOT_NAME_HEX_DIGITS + SNAPSHOT_NAME_SUFFIX.len()
        && bytes.ends_with(SNAPSHOT_NAME_SUFFIX.as_bytes())
        && bytes[..SNAPSHOT_NAME_HEX_DIGITS]
            .iter()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

/// Whether a name is the content sidecar of a snapshot name.
fn is_sidecar_name_bytes(bytes: &[u8]) -> bool {
    bytes.strip_suffix(CONTENT_NAME_SUFFIX.as_bytes()).is_some_and(is_snapshot_name_bytes)
}

/// Which of fdu's names a file in the cache directory is shaped like.
///
/// Shape only: the magic decides whether the file really is fdu's, and every caller checks
/// it. Keeping the two apart is what stops a name from being evidence, which is the same
/// rule a snapshot follows.
fn name_shape(name: &OsStr) -> NameShape {
    let bytes = name.as_encoded_bytes();
    if is_snapshot_name_bytes(bytes) {
        return NameShape::Snapshot;
    }
    if is_sidecar_name_bytes(bytes) {
        return NameShape::Sidecar;
    }
    // `.{target}.tmp.{pid}.{entropy}.{sequence}`, the staging name the snapshot writer
    // gives every file it publishes by rename, for either target.
    let Some(rest) = bytes.strip_prefix(b".") else { return NameShape::Other };
    let Some(at) =
        rest.windows(TEMP_NAME_INFIX.len()).position(|w| w == TEMP_NAME_INFIX.as_bytes())
    else {
        return NameShape::Other;
    };
    let (target, suffix) = rest.split_at(at);
    if suffix.len() <= TEMP_NAME_INFIX.len() {
        // No discriminator after the infix: not a name the writer produces.
        return NameShape::Other;
    }
    if is_snapshot_name_bytes(target) {
        NameShape::SnapshotTemporary
    } else if is_sidecar_name_bytes(target) {
        NameShape::SidecarTemporary
    } else {
        NameShape::Other
    }
}

/// What a file name in the cache directory is shaped like.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NameShape {
    /// `{16 hex}.fdu`: the snapshot of some root.
    Snapshot,
    /// `{16 hex}.fdu.content`: the content sidecar of that snapshot.
    Sidecar,
    /// `.{16 hex}.fdu.tmp.*`: a snapshot being staged.
    SnapshotTemporary,
    /// `.{16 hex}.fdu.content.tmp.*`: a sidecar being staged.
    SidecarTemporary,
    /// A name the cache gives nothing.
    Other,
}

/// Read one cache file's status without materializing its index.
///
/// The file is identified by its contents, because the caller named it: only a name fdu
/// gives its own staging files and sidecars is read as one, and even then the magic
/// decides. A symbolic link at `path` is reported unrecognized rather than followed.
///
/// One path is all this sees, so a sidecar is judged by its own name and magic:
/// [`LeftoverKind::OrphanedContent`] here says the file is a sidecar, not that no snapshot
/// claims it. Whether one does is a fact about the directory, which [`list_caches`]
/// answers and a clear asks again at the moment of removal.
pub fn cache_status(path: &Path) -> Result<CacheStatus> {
    Ok(status_at(path)?.unwrap_or_else(|| CacheStatus {
        path: path.to_path_buf(),
        bytes: 0,
        content: None,
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
        return Ok(Some(unrecognized(path, reportable_bytes(&metadata))));
    }
    // A staging file holds a complete image the moment before its rename, so identifying
    // it by contents alone would call it a current snapshot. The name is what says it was
    // never published, and only fdu's own staging names carry that meaning.
    //
    // A leftover's name is answered here and nowhere below: the name has already said
    // which magic to expect, so contents that are not it leave the file unrecognized.
    // Falling through to the snapshot identification would read a snapshot image under a
    // sidecar's name as a snapshot, and then clear it under a name no snapshot is given
    // and without the age rule that name carries.
    let leftover = match path.file_name().map_or(NameShape::Other, name_shape) {
        NameShape::SnapshotTemporary => Some(leftover_or_unrecognized(
            path,
            metadata.len(),
            LeftoverKind::StagingTemporary,
            is_snapshot_image(path)?,
        )),
        NameShape::SidecarTemporary => Some(leftover_or_unrecognized(
            path,
            metadata.len(),
            LeftoverKind::StagingTemporary,
            is_sidecar_image(path)?,
        )),
        NameShape::Sidecar => Some(leftover_or_unrecognized(
            path,
            metadata.len(),
            LeftoverKind::OrphanedContent,
            is_sidecar_image(path)?,
        )),
        NameShape::Snapshot | NameShape::Other => None,
    };
    if let Some(status) = leftover {
        return Ok(Some(status));
    }
    let state = match snapshot::identify(path)? {
        None => return Ok(None),
        Some(Identity::Foreign) => return Ok(Some(unrecognized(path, metadata.len()))),
        Some(Identity::Current(info)) => CacheState::Current(info),
        Some(Identity::Stale(reason)) => CacheState::Stale(reason),
    };
    let content = crate::content::identify_sidecar(&crate::content::content_cache_path(path))?;
    Ok(Some(CacheStatus { path: path.to_path_buf(), bytes: metadata.len(), content, state }))
}

/// Whether a regular file at `path` begins with the snapshot magic.
///
/// Only a regular file is opened: a symbolic link must not be followed out of the cache
/// directory, and opening a directory succeeds on some platforms and fails on the first
/// read, which would turn a question into an error.
fn is_snapshot_image(path: &Path) -> Result<bool> {
    let Some(metadata) = present(fs::symlink_metadata(path), path)? else { return Ok(false) };
    if !metadata.file_type().is_file() {
        return Ok(false);
    }
    Ok(matches!(snapshot::identify(path)?, Some(Identity::Current(_) | Identity::Stale(_))))
}

/// Whether the file at `path` begins with the content-sidecar magic.
fn is_sidecar_image(path: &Path) -> Result<bool> {
    Ok(crate::content::content_sidecar_bytes(path)?.is_some())
}

/// The snapshot a content sidecar belongs to: the inverse of `content_cache_path`.
///
/// `with_extension("")` drops the *final* extension, which is `.content` for every name
/// this is called with — [`NameShape::Sidecar`] is what proves that.
fn sidecar_snapshot_path(path: &Path) -> PathBuf {
    path.with_extension("")
}

fn unrecognized(path: &Path, bytes: u64) -> CacheStatus {
    CacheStatus { path: path.to_path_buf(), bytes, content: None, state: CacheState::Unrecognized }
}

/// A file under one of fdu's own leftover names: the leftover when its contents are the
/// magic that name implies, and unrecognized when they are not.
fn leftover_or_unrecognized(
    path: &Path,
    bytes: u64,
    kind: LeftoverKind,
    has_magic: bool,
) -> CacheStatus {
    if has_magic {
        CacheStatus {
            path: path.to_path_buf(),
            bytes,
            content: None,
            state: CacheState::Leftover(kind),
        }
    } else {
        unrecognized(path, bytes)
    }
}

/// The byte count to report for a cache entry.
///
/// Only a regular file has a size a report is about. A directory's `st_size` is that
/// directory entry's own accounting, which differs per filesystem and is zero on Windows,
/// and a symbolic link's is the length of the path it holds. Either would be read as a
/// file's size and summed into the unrecognized total, so a non-regular entry reports no
/// bytes.
fn reportable_bytes(metadata: &fs::Metadata) -> u64 {
    if metadata.file_type().is_file() { metadata.len() } else { 0 }
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

/// Enumerate every entry in the cache directory.
///
/// Unrecognized entries are listed rather than hidden: a directory this code will not
/// delete from is a directory the caller should still be able to see, and an entry left
/// out of the listing is one nothing can report. A file counts as a snapshot only under a
/// name the cache gives snapshots and with the snapshot magic, so neither a stray copy
/// under another name nor another program's file under a snapshot's name is mistaken for
/// one. Symbolic links and directories are listed from their own metadata, never followed
/// or descended into, and report no bytes, because what the filesystem calls their size is
/// its own accounting rather than anything a clear could reclaim.
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
        let status = if file_type.is_file() && name_shape(&entry.file_name()) != NameShape::Other {
            status_at(&path)?
        } else {
            present(fs::symlink_metadata(&path), &path)?
                .map(|metadata| unrecognized(&path, reportable_bytes(&metadata)))
        };
        found.extend(status);
    }
    let paired_sidecars = found
        .iter()
        .filter(|status| status.is_fdu_snapshot() && status.content.is_some())
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

/// Remove every fdu snapshot in a cache directory, current or stale, and reclaim the files
/// fdu left behind, leaving anything else alone.
///
/// Each file is identified again immediately before it is removed, so one replaced since
/// the listing by something that is not fdu's survives.
///
/// Snapshots go first and leftovers second, so "no snapshot for this sidecar" is decided
/// against the directory this call leaves rather than the one it found.
pub fn clear_all_caches(cache_dir: &Path) -> Result<ClearSummary> {
    let listed = list_caches(cache_dir)?;
    let mut summary = ClearSummary::default();
    for status in &listed {
        if status.is_fdu_snapshot() && clear_cache(&status.path)? {
            summary.snapshots += 1;
        }
    }
    for status in &listed {
        if let CacheState::Leftover(kind) = status.state {
            if clear_leftover(&status.path, kind)? {
                summary.leftovers += 1;
            }
        }
    }
    Ok(summary)
}

/// Remove one file fdu left behind, if the rules for its kind allow it now.
///
/// Returns whether this call removed it. The status is read again here rather than trusted
/// from a listing: both rules are about the state of the directory at the moment of
/// removal, and a leftover is the one thing in the cache that another process may still be
/// writing.
fn clear_leftover(path: &Path, kind: LeftoverKind) -> Result<bool> {
    let Some(status) = status_at(path)? else { return Ok(false) };
    if status.state != CacheState::Leftover(kind) {
        return Ok(false);
    }
    let removable = match kind {
        // The reaper's threshold, not a second one: a temporary this old cannot belong to
        // a writer that is still running, which is what makes removing it safe without a
        // liveness check no platform can give.
        LeftoverKind::StagingTemporary => {
            let Some(metadata) = present(fs::symlink_metadata(path), path)? else {
                return Ok(false);
            };
            let Ok(modified) = metadata.modified() else { return Ok(false) };
            std::time::SystemTime::now()
                .duration_since(modified)
                .is_ok_and(|age| age >= snapshot::STALE_TEMP_AGE)
        }
        // Kept only while a snapshot image is at its snapshot path: one this clear left,
        // or one a writer published since the listing, either of which a later analyzed
        // scan can still use it for. Anything else there is not a snapshot and cannot
        // want this sidecar, so the sidecar goes and the scan that writes a snapshot at
        // that path writes a fresh one.
        LeftoverKind::OrphanedContent => !is_snapshot_image(&sidecar_snapshot_path(path))?,
    };
    if !removable {
        return Ok(false);
    }
    remove_present(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CachePolicy, OpenFixture, open};

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
        let config = OpenFixture {
            cache_path: Some(snapshot_path.to_path_buf()),
            policy: CachePolicy::Auto,
            ..OpenFixture::default()
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
        let config = OpenFixture {
            cache_path: Some(path.clone()),
            policy: CachePolicy::Auto,
            analysis: crate::content::AnalysisRequest {
                profile: crate::content::AnalysisSet::NONE.with_lines(),
                ..crate::content::AnalysisRequest::default()
            },
            ..OpenFixture::default()
        };
        let (index, _) = open(tree.path(), &config).expect("seed analyzed cache");

        let listed = list_caches(cache.path()).expect("list");
        assert_eq!(listed.len(), 1, "a sidecar is grouped with its snapshot");
        let Some(CacheState::Current(snapshot)) = listed.first().map(|status| &status.state) else {
            panic!("a current snapshot: {listed:?}");
        };
        assert_eq!(snapshot.identity, index.snapshot_identity());
        let content = listed[0].content.as_ref().expect("its sidecar");
        assert_eq!(
            content.state,
            ContentState::Current(ContentInfo {
                identity: index.content_identity(config.analysis.profile),
                records: 1,
            }),
            "status reports the identity the sidecar serves"
        );
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
        assert_eq!(clear_all_caches(cache.path()).expect("clear").snapshots, 5);
        assert!(list_caches(cache.path()).expect("list").is_empty());
    }

    #[test]
    fn a_stale_snapshot_takes_its_content_sidecar_with_it() {
        let tree = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache");
        let path = cache.path().join(layout_name(1));
        std::fs::write(tree.path().join("notes.md"), b"one two\n").expect("write");
        let config = OpenFixture {
            cache_path: Some(path.clone()),
            policy: CachePolicy::Auto,
            analysis: crate::content::AnalysisRequest {
                profile: crate::content::AnalysisSet::NONE.with_lines(),
                ..crate::content::AnalysisRequest::default()
            },
            ..OpenFixture::default()
        };
        open(tree.path(), &config).expect("seed analyzed cache");
        with_format_version(&path, &path, |version| version - 1);

        let listed = list_caches(cache.path()).expect("list");
        assert_eq!(listed.len(), 1, "a stale snapshot's sidecar is grouped with it");
        assert!(matches!(listed[0].state, CacheState::Stale(StaleReason::OlderFormat { .. })));
        assert!(listed[0].content.is_some());
        assert_eq!(clear_all_caches(cache.path()).expect("clear").snapshots, 1);
        assert!(!crate::content::content_cache_path(&path).exists());
    }

    /// A sidecar this build cannot serve beside a snapshot it can is still fdu's: status
    /// labels it stale with the reason, whatever its contents after the magic, and
    /// clearing the snapshot takes it too, because pairing and clearing decide by magic.
    #[test]
    fn a_stale_sidecar_beside_a_current_snapshot_is_labelled_and_cleared() {
        let tree = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache");
        let path = cache.path().join(layout_name(1));
        std::fs::write(tree.path().join("notes.md"), b"one two\n").expect("write");
        let config = OpenFixture {
            cache_path: Some(path.clone()),
            policy: CachePolicy::Auto,
            analysis: crate::content::AnalysisRequest {
                profile: crate::content::AnalysisSet::NONE.with_lines(),
                ..crate::content::AnalysisRequest::default()
            },
            ..OpenFixture::default()
        };
        open(tree.path(), &config).expect("seed analyzed cache");
        let sidecar = crate::content::content_cache_path(&path);
        let written = std::fs::read(&sidecar).expect("a sidecar");

        let mut older = written.clone();
        older[VERSION_OFFSET..FINGERPRINT_OFFSET].copy_from_slice(&4_u32.to_le_bytes());
        let mut newer = written.clone();
        newer[VERSION_OFFSET..FINGERPRINT_OFFSET].copy_from_slice(&99_u32.to_le_bytes());
        let mut other_engine = written.clone();
        other_engine[FINGERPRINT_OFFSET] ^= 0xff;
        let truncated = written[..written.len() - 1].to_vec();
        let mut unreadable_header = written.clone();
        let path_encoding_at = FINGERPRINT_OFFSET + 8;
        unreadable_header[path_encoding_at] ^= 0xff;
        for (image, reason) in [
            (older, StaleReason::OlderFormat { version: 4 }),
            (newer, StaleReason::NewerFormat { version: 99 }),
            (other_engine, StaleReason::OtherEngine),
            (truncated, StaleReason::Unreadable),
            (unreadable_header, StaleReason::Unreadable),
        ] {
            std::fs::write(&sidecar, &image).expect("rewrite the sidecar");
            let listed = list_caches(cache.path()).expect("list");
            assert_eq!(listed.len(), 1, "a stale sidecar is still grouped with its snapshot");
            assert!(matches!(listed[0].state, CacheState::Current(_)), "{listed:?}");
            let content = listed[0].content.as_ref().expect("its sidecar");
            assert_eq!(content.bytes, u64::try_from(image.len()).expect("small"));
            assert_eq!(content.state, ContentState::Stale(reason), "{reason:?}");
            assert_eq!(cache_status(&path).expect("status").content, listed[0].content);
        }

        assert!(clear_cache(&path).expect("clear"));
        assert!(!path.exists());
        assert!(!sidecar.exists(), "the stale sidecar goes with its snapshot");
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
        assert_eq!(clear_all_caches(cache.path()).expect("clear").snapshots, 1);
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
        // A link's `st_size` is the length of the path it holds, which is the tempdir's,
        // so reporting it would put a machine-specific number in a byte total.
        assert_eq!(listed[0].bytes, 0, "a link has no size to report");
        assert_eq!(cache_status(&link).expect("status").state, CacheState::Unrecognized);

        assert!(clear_all_caches(cache.path()).expect("clear").is_empty());
        assert!(!clear_cache(&link).expect("clear"));
        assert!(std::fs::symlink_metadata(&link).is_ok(), "the link stays");
        assert!(
            cache_status(&target).expect("status").snapshot().is_some(),
            "and so does its target"
        );
    }

    #[test]
    fn a_directory_in_the_cache_is_listed_and_never_removed() {
        // Skipping directories hid them from every report: `--cache-status=all` showed
        // nothing and `--cache-clear=all` counted nothing, while the docs promised that
        // anything which is not a regular file is listed as unrecognized.
        let tree = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache");
        let snapshot = cache.path().join(layout_name(1));
        seed(tree.path(), &snapshot);
        // Under a snapshot's own name, so the name cannot be what saves it.
        let nested = cache.path().join(layout_name(2));
        std::fs::create_dir(&nested).expect("create dir");

        let listed = list_caches(cache.path()).expect("list");
        assert_eq!(
            listed
                .iter()
                .map(|status| (status.path.clone(), status.state.label()))
                .collect::<Vec<_>>(),
            [(snapshot, "current"), (nested.clone(), "unrecognized")]
        );
        // No byte count: `st_size` for a directory is 4096 on one filesystem, 0 on
        // Windows, and something else on the next, and it would be summed into the bytes
        // a status attributes to unrecognized files.
        assert_eq!(listed[1].bytes, 0, "a directory has no size to report");
        let single = cache_status(&nested).expect("status");
        assert_eq!(single.state, CacheState::Unrecognized);
        assert_eq!(single.bytes, 0, "and the single-path route agrees");

        assert_eq!(clear_all_caches(cache.path()).expect("clear").snapshots, 1);
        assert!(!clear_cache(&nested).expect("clear"));
        assert!(nested.is_dir(), "a directory is never removed");
    }

    /// Move a file's modification time, so an age rule is tested by stating an age rather
    /// than by waiting for one.
    fn set_modified(path: &Path, at: std::time::SystemTime) {
        std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .expect("open")
            .set_modified(at)
            .expect("set modified");
    }

    /// A moment old enough that no writer can still hold what was written then.
    fn beyond_the_reaper() -> std::time::SystemTime {
        std::time::SystemTime::now() - snapshot::STALE_TEMP_AGE - std::time::Duration::from_secs(1)
    }

    /// The staging name the writer gives `target`, with an arbitrary discriminator.
    fn staging_name(target: &str) -> String {
        format!(".{target}.tmp.7.0123456789abcdef.0")
    }

    /// A status as one string, so a listing's states and leftover kinds compare at once.
    fn describe(status: &CacheStatus) -> String {
        match &status.state {
            CacheState::Leftover(kind) => format!("leftover/{}", kind.label()),
            other => other.label().to_string(),
        }
    }

    #[test]
    fn fdus_own_leftovers_are_named_as_fdus_and_reclaimed_only_under_their_rules() {
        // Both files begin with an fdu magic and neither is a snapshot in place, so
        // calling them "not an fdu snapshot" told the user to leave fdu's own debris
        // alone, and nothing collected it.
        let tree = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache");
        let current = cache.path().join(layout_name(1));
        seed(tree.path(), &current);
        let image = std::fs::read(&current).expect("read snapshot");

        // A writer killed between staging and rename, long enough ago that it cannot be
        // running.
        let abandoned = cache.path().join(staging_name(&layout_name(2)));
        std::fs::write(&abandoned, &image).expect("write");
        set_modified(&abandoned, beyond_the_reaper());
        // The same shape, written a moment ago: this one may be a live writer's.
        let in_flight = cache.path().join(staging_name(&layout_name(3)));
        std::fs::write(&in_flight, &image).expect("write");
        // A staging name over contents that are not fdu's: the name alone proves nothing.
        let impostor = cache.path().join(staging_name(&layout_name(4)));
        std::fs::write(&impostor, b"not a snapshot").expect("write");
        set_modified(&impostor, beyond_the_reaper());
        // A staged sidecar, and a sidecar whose snapshot is gone.
        let staged_sidecar =
            cache.path().join(staging_name(&format!("{}.content", layout_name(5))));
        std::fs::write(&staged_sidecar, b"FDUCTNT\0payload").expect("write");
        set_modified(&staged_sidecar, beyond_the_reaper());
        let orphan = cache.path().join(format!("{}.content", layout_name(6)));
        std::fs::write(&orphan, b"FDUCTNT\0payload").expect("write");
        // A sidecar name over contents that are not fdu's.
        let foreign_sidecar = cache.path().join(format!("{}.content", layout_name(7)));
        std::fs::write(&foreign_sidecar, b"not a sidecar").expect("write");
        // A snapshot image under a sidecar's name, and under a staged sidecar's name. The
        // name says which magic to expect and the snapshot's is not it, so both are
        // unrecognized: reading them as snapshots would clear them under a name no
        // snapshot is given, and the staged one without the age rule its name carries.
        let snapshot_under_sidecar_name = cache.path().join(format!("{}.content", layout_name(8)));
        std::fs::write(&snapshot_under_sidecar_name, &image).expect("write");
        let snapshot_under_staged_sidecar_name =
            cache.path().join(staging_name(&format!("{}.content", layout_name(9))));
        std::fs::write(&snapshot_under_staged_sidecar_name, &image).expect("write");
        set_modified(&snapshot_under_staged_sidecar_name, beyond_the_reaper());

        let listed = list_caches(cache.path()).expect("list");
        assert_eq!(
            listed.iter().map(|status| (status.path.clone(), describe(status))).collect::<Vec<_>>(),
            [
                (abandoned.clone(), "leftover/staging_temporary".to_string()),
                (in_flight.clone(), "leftover/staging_temporary".to_string()),
                (impostor.clone(), "unrecognized".to_string()),
                (staged_sidecar.clone(), "leftover/staging_temporary".to_string()),
                (snapshot_under_staged_sidecar_name.clone(), "unrecognized".to_string()),
                (current.clone(), "current".to_string()),
                (orphan.clone(), "leftover/orphaned_content".to_string()),
                (foreign_sidecar.clone(), "unrecognized".to_string()),
                (snapshot_under_sidecar_name.clone(), "unrecognized".to_string()),
            ]
        );
        // The single-path route says the same, so neither depends on the listing to be
        // safe.
        for named in [&snapshot_under_sidecar_name, &snapshot_under_staged_sidecar_name] {
            assert_eq!(cache_status(named).expect("status").state, CacheState::Unrecognized);
        }

        let summary = clear_all_caches(cache.path()).expect("clear");
        assert_eq!(summary, ClearSummary { snapshots: 1, leftovers: 3 });
        for gone in [&abandoned, &staged_sidecar, &orphan, &current] {
            assert!(!gone.exists(), "{} should be reclaimed", gone.display());
        }
        for kept in [
            &in_flight,
            &impostor,
            &foreign_sidecar,
            &snapshot_under_sidecar_name,
            &snapshot_under_staged_sidecar_name,
        ] {
            assert!(kept.exists(), "{} must survive", kept.display());
        }
        // Idempotent, and the young temporary is still nobody's business to remove.
        assert_eq!(clear_all_caches(cache.path()).expect("clear"), ClearSummary::default());
    }

    #[test]
    fn a_sidecar_is_kept_while_a_snapshot_still_claims_it() {
        // The other half of the orphan rule: what makes a sidecar collectable is that no
        // snapshot is left to want it, decided at the moment of removal.
        let tree = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache");
        let current = cache.path().join(layout_name(1));
        seed(tree.path(), &current);
        let sidecar = crate::content::content_cache_path(&current);
        std::fs::write(&sidecar, b"FDUCTNT\0payload").expect("write");

        // Grouped with its snapshot rather than listed as debris.
        let listed = list_caches(cache.path()).expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].content_bytes(), Some(15));
        assert!(!clear_leftover(&sidecar, LeftoverKind::OrphanedContent).expect("clear"));
        assert!(sidecar.exists(), "its snapshot is still there");

        // Once the snapshot is gone, the same call collects it.
        assert!(remove_present(&current).expect("remove"));
        assert!(clear_leftover(&sidecar, LeftoverKind::OrphanedContent).expect("clear"));
        assert!(!sidecar.exists());
    }

    #[cfg(unix)]
    #[test]
    fn a_symbolic_link_named_like_a_leftover_is_never_followed_or_removed() {
        let tree = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache");
        let outside = tempfile::tempdir().expect("outside");
        let target = outside.path().join(layout_name(9));
        seed(tree.path(), &target);
        // No mtime is set: opening the link to age it would age its target instead, and
        // the rule that saves it here is that a link is not a regular file at all.
        let link = cache.path().join(staging_name(&layout_name(1)));
        std::os::unix::fs::symlink(&target, &link).expect("symlink");

        let listed = list_caches(cache.path()).expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].state, CacheState::Unrecognized);
        assert!(clear_all_caches(cache.path()).expect("clear").is_empty());
        assert!(!clear_leftover(&link, LeftoverKind::StagingTemporary).expect("clear"));
        assert!(std::fs::symlink_metadata(&link).is_ok(), "the link stays");
        assert!(target.exists(), "and so does its target");
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
        assert!(clear_all_caches(&missing).expect("clear").is_empty());
        let absent = missing.join(layout_name(1));
        assert!(!clear_cache(&absent).expect("clear"));
        assert_eq!(cache_status(&absent).expect("status").state, CacheState::Absent);
    }

    #[test]
    fn the_layout_names_what_default_cache_path_produces() {
        // `expect`, not `if let`: with no cache directory this test would otherwise assert
        // only its negative cases and pass without touching its subject. Every supported
        // platform resolves one, and `XDG_CACHE_HOME` names it everywhere if the platform
        // location does not.
        let tree = tempfile::tempdir().expect("tempdir");
        let path = crate::default_cache_path(tree.path())
            .expect("a user cache directory: set XDG_CACHE_HOME if this platform has none");
        assert_eq!(
            name_shape(path.file_name().expect("name")),
            NameShape::Snapshot,
            "{}",
            path.display()
        );
    }

    #[test]
    fn a_name_is_shaped_like_one_of_fdus_files_or_like_nothing() {
        // The shape is a gate, never evidence: each of these still has to carry the right
        // magic before the state that matches it can be reported.
        for (name, shape) in [
            ("0123456789abcdef.fdu", NameShape::Snapshot),
            ("0123456789abcdef.fdu.content", NameShape::Sidecar),
            (".0123456789abcdef.fdu.tmp.1.0011223344556677.9", NameShape::SnapshotTemporary),
            (".0123456789abcdef.fdu.content.tmp.1.0011223344556677.9", NameShape::SidecarTemporary),
            // Near misses: uppercase hex, a missing discriminator, a target that is not a
            // name the cache gives, and the sidecar of something that is not a snapshot.
            ("0123456789ABCDEF.fdu", NameShape::Other),
            (".0123456789abcdef.fdu.tmp.", NameShape::Other),
            (".notes.txt.tmp.1.0011223344556677.9", NameShape::Other),
            ("notes.txt.content", NameShape::Other),
            ("0123456789abcdef.fdu.content.content", NameShape::Other),
            (".fdu.tmp.1", NameShape::Other),
        ] {
            assert_eq!(name_shape(OsStr::new(name)), shape, "{name}");
        }
    }

    #[test]
    fn labels_list_every_variant_in_order() {
        let states = [
            CacheState::Current(SnapshotInfo {
                root: PathBuf::new(),
                identity: crate::ScanConfig::default().snapshot_identity(),
                entries: 1,
            }),
            CacheState::Stale(StaleReason::Unreadable),
            CacheState::Leftover(LeftoverKind::StagingTemporary),
            CacheState::Unrecognized,
            CacheState::Absent,
        ];
        assert_eq!(states.iter().map(CacheState::label).collect::<Vec<_>>(), CacheState::LABELS);
        assert_eq!(
            [LeftoverKind::StagingTemporary, LeftoverKind::OrphanedContent]
                .map(LeftoverKind::label),
            LeftoverKind::LABELS
        );
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
