//! Persisting an index to disk and reading it back.
//!
//! # Status: format v0 is a placeholder, and deliberately a boring one
//!
//! This module implements a flat, uncompressed, read-it-all format. It exists so the
//! cache *lifecycle* — engine-fingerprint invalidation, atomic replacement, and
//! corrupt-equals-empty — is nailed down and tested before the wire format is designed,
//! because those invariants are the ones that make a cache trustworthy and they are
//! independent of layout.
//!
//! The target format is different in every other respect: zstd-compressed blocks with an
//! index block at the tail, so opening costs one small read and directory listings
//! decompress on demand into a small LRU cache; item references encoded as
//! `(block << k) | offset` and delta-encoded when they point inside the same block;
//! sibling groups written contiguously so one directory listing costs one block
//! decompression; front-coded names; and pre-computed roll-ups stored per directory so a
//! query never re-aggregates. O(1) open with lazy materialization matters more than raw
//! decode throughput, because it matches how a UI actually navigates: open now, expand
//! later.
//!
//! Two things here are **not** placeholders and must survive the format change:
//!
//! - **A corrupt or unrecognized snapshot is treated as absent, never as an error and
//!   never as data.** A cache that fails closed costs a rescan; a cache that fails open
//!   silently lies.
//! - **Replacement is atomic.** Write a temporary file, then rename over the target, so a
//!   crash mid-write leaves the previous snapshot intact rather than a half-written one.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::index::{EntryId, Index};
use crate::types::{Attrs, Delta, EntryKind, Error, Op, Result};

/// Leading magic. Distinguishes an fdu snapshot from any other file that lands here.
const MAGIC: &[u8; 8] = b"FDUSNAP\x00";

/// Trailing magic. Present only once the whole snapshot has been written, so a truncated
/// file is detected on load rather than parsed into a plausible-looking partial tree.
const TRAILER: &[u8; 8] = b"FDUEND\x00\x00";

/// On-disk format version. Bump on any layout change; old snapshots are then discarded
/// rather than misread.
const FORMAT_VERSION: u32 = 0;

/// Marks the root's absent parent.
const NO_PARENT: u32 = u32::MAX;

/// Smallest possible on-disk record: parent slot, kind, name length, and six 8-byte
/// attribute fields, with a zero-length name. Used to sanity-check a declared entry
/// count against the bytes actually present.
const MIN_RECORD_BYTES: usize = 4 + 1 + 4 + 8 * 6;

/// A fingerprint of everything that would change how the engine interprets a tree.
///
/// When this does not match, the whole snapshot is discarded. That is the cheap,
/// wholesale answer to "the rules changed, so every derived verdict in the cache might
/// be wrong" — far simpler than trying to work out which entries a rule change affected.
///
/// Currently derived from the crate version and the format version. When compiled
/// type-recognition rules and reducer registrations arrive, their hashes belong here too.
pub fn engine_fingerprint() -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut mix = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x1000_0000_01b3);
        }
    };
    mix(env!("CARGO_PKG_VERSION").as_bytes());
    mix(&FORMAT_VERSION.to_le_bytes());
    hash
}

/// Write `index` to `path`, replacing any existing snapshot atomically.
pub fn save(index: &Index, path: &Path) -> Result<()> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(MAGIC);
    buf.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    buf.extend_from_slice(&engine_fingerprint().to_le_bytes());

    let root_path = index.root_path().to_string_lossy().into_owned();
    put_bytes(&mut buf, root_path.as_bytes())?;

    // Pre-order, so a parent's record always precedes its children's and the loader can
    // rebuild the tree in one forward pass with no fixups.
    let mut records: Vec<(u32, EntryId)> = Vec::new();
    let mut stack: Vec<(u32, EntryId)> = vec![(NO_PARENT, EntryId::ROOT)];
    while let Some((parent_slot, id)) = stack.pop() {
        let slot = u32::try_from(records.len())
            .map_err(|_| Error::Snapshot("snapshot exceeds u32 entry capacity".into()))?;
        records.push((parent_slot, id));
        for (_, child) in index.children_of(id).into_iter().rev() {
            stack.push((slot, child));
        }
    }

    let count = u64::try_from(records.len())
        .map_err(|_| Error::Snapshot("snapshot entry count overflow".into()))?;
    buf.extend_from_slice(&count.to_le_bytes());

    for (parent_slot, id) in records {
        buf.extend_from_slice(&parent_slot.to_le_bytes());
        buf.push(index.kind_of(id) as u8);
        put_bytes(&mut buf, index.name_of(id).as_bytes())?;
        let attrs = index.attrs_of(id);
        buf.extend_from_slice(&attrs.size.to_le_bytes());
        buf.extend_from_slice(&attrs.allocated.to_le_bytes());
        buf.extend_from_slice(&attrs.mtime_ns.to_le_bytes());
        buf.extend_from_slice(&attrs.ctime_ns.to_le_bytes());
        buf.extend_from_slice(&attrs.inode.to_le_bytes());
        buf.extend_from_slice(&attrs.dev.to_le_bytes());
    }
    buf.extend_from_slice(TRAILER);

    write_atomically(path, &buf)
}

/// Load a snapshot, or return `None` when there is nothing usable at `path`.
///
/// Returns `None` — not an error — for a missing file, a foreign file, a version or
/// engine-fingerprint mismatch, and a truncated or corrupt file. Every one of those
/// means the same thing to a caller: there is no warm cache, so scan.
pub fn load(path: &Path) -> Result<Option<Index>> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(Error::io(path, e)),
    };
    Ok(parse(&bytes))
}

/// Parse a snapshot image, yielding `None` for anything not recognizable as a current,
/// complete one.
fn parse(bytes: &[u8]) -> Option<Index> {
    let mut cur = Cursor::new(bytes);
    if cur.take(MAGIC.len())? != MAGIC {
        return None;
    }
    if cur.u32()? != FORMAT_VERSION {
        return None;
    }
    if cur.u64()? != engine_fingerprint() {
        return None;
    }
    // A truncated snapshot can still parse as a shorter valid one, so the trailer is
    // checked before any of the body is trusted.
    if bytes.len() < TRAILER.len() || &bytes[bytes.len() - TRAILER.len()..] != TRAILER {
        return None;
    }

    let root_path = String::from_utf8(cur.bytes()?.to_vec()).ok()?;

    // The declared count comes from a file that may be corrupt or hostile, so it is
    // checked against what the remaining bytes could physically hold before it is
    // allowed to size an allocation. Trusting it directly turns a corrupt snapshot into
    // an out-of-memory abort — which would be exactly the "fails open" behaviour this
    // module exists to prevent.
    let count = usize::try_from(cur.u64()?).ok()?;
    let remaining = bytes.len().checked_sub(cur.pos)?;
    if count.checked_mul(MIN_RECORD_BYTES)? > remaining {
        return None;
    }

    let mut index = Index::new(&root_path);
    // Slot number -> path, so a child can name its parent by the path the index knows.
    let mut paths: Vec<PathBuf> = Vec::with_capacity(count);
    let mut ops: Vec<Op> = Vec::new();

    for _ in 0..count {
        let parent_slot = cur.u32()?;
        let kind = EntryKind::from_u8(cur.u8()?)?;
        let name = String::from_utf8(cur.bytes()?.to_vec()).ok()?;
        let attrs = Attrs {
            size: cur.u64()?,
            allocated: cur.u64()?,
            mtime_ns: cur.i64()?,
            ctime_ns: cur.i64()?,
            inode: cur.u64()?,
            dev: cur.u64()?,
        };

        if parent_slot == NO_PARENT {
            paths.push(PathBuf::new());
            continue;
        }
        let parent_path = paths.get(usize::try_from(parent_slot).ok()?)?;
        let path = parent_path.join(&name);
        paths.push(path.clone());
        ops.push(Op::Upsert { path, kind, attrs });
    }

    if cur.take(TRAILER.len())? != TRAILER {
        return None;
    }

    // Rebuilding through the delta path means roll-up state can never disagree with the
    // entries it summarizes: there is one code path that computes it.
    index.apply(&Delta::new(ops));
    Some(index)
}

fn put_bytes(buf: &mut Vec<u8>, bytes: &[u8]) -> Result<()> {
    let len = u32::try_from(bytes.len())
        .map_err(|_| Error::Snapshot("string too long for snapshot".into()))?;
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(bytes);
    Ok(())
}

/// Write to a sibling temporary file, then rename over the target.
///
/// Rename is atomic within a filesystem, so a reader either sees the whole old snapshot
/// or the whole new one. The temporary must be a sibling for that to hold — a rename
/// across filesystems is a copy, and copies are not atomic.
fn write_atomically(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;

    let tmp = parent.join(format!(
        ".{}.tmp{}",
        path.file_name().map_or_else(|| "snapshot".into(), |n| n.to_string_lossy()),
        std::process::id()
    ));

    let mut file = fs::File::create(&tmp).map_err(|e| Error::io(&tmp, e))?;
    let write_then_sync = file.write_all(bytes).and_then(|()| file.sync_all());
    if let Err(e) = write_then_sync {
        let _ = fs::remove_file(&tmp);
        return Err(Error::io(&tmp, e));
    }
    drop(file);

    if let Err(e) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(Error::io(path, e));
    }
    Ok(())
}

/// Minimal forward-only reader. Every accessor returns `None` past the end, so a
/// truncated file falls out as "unusable snapshot" rather than a panic.
struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        let slice = self.bytes.get(self.pos..end)?;
        self.pos = end;
        Some(slice)
    }

    fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }

    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }

    fn i64(&mut self) -> Option<i64> {
        Some(i64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }

    fn bytes(&mut self) -> Option<&'a [u8]> {
        let len = self.u32()?;
        self.take(usize::try_from(len).ok()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::ExtTally;
    use crate::types::Delta;

    fn attrs(size: u64, mtime_ns: i64) -> Attrs {
        Attrs {
            size,
            allocated: size.div_ceil(512) * 512,
            mtime_ns,
            ctime_ns: mtime_ns,
            inode: size.wrapping_mul(7).wrapping_add(1),
            dev: 3,
        }
    }

    fn sample_index() -> Index {
        let mut index = Index::new("/some/root");
        index.apply(&Delta::new(vec![
            Op::Upsert { path: PathBuf::from("src"), kind: EntryKind::Dir, attrs: attrs(0, 1) },
            Op::Upsert {
                path: PathBuf::from("src/main.rs"),
                kind: EntryKind::File,
                attrs: attrs(100, 10),
            },
            Op::Upsert {
                path: PathBuf::from("src/deep/nested.rs"),
                kind: EntryKind::File,
                attrs: attrs(50, 20),
            },
            Op::Upsert {
                path: PathBuf::from("notes.md"),
                kind: EntryKind::File,
                attrs: attrs(7, 30),
            },
        ]));
        index
    }

    #[test]
    fn round_trip_preserves_tree_and_rollups() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("cache").join("snap.fdu");
        let original = sample_index();

        save(&original, &path).expect("save");
        let restored = load(&path).expect("load").expect("snapshot present");

        assert_eq!(restored.root_path(), Path::new("/some/root"));
        assert_eq!(restored.len(), original.len());
        assert_eq!(restored.total(), original.total());
        assert_eq!(restored.total().files, 3);
        assert_eq!(restored.total().dirs, 2);
        assert_eq!(restored.total().bytes, 157);
        assert_eq!(restored.total().by_ext[".rs"], ExtTally { files: 2, bytes: 150 });
        assert_eq!(
            restored.attrs(Path::new("src/deep/nested.rs")),
            original.attrs(Path::new("src/deep/nested.rs"))
        );
    }

    #[test]
    fn missing_snapshot_is_absent_not_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let loaded = load(&dir.path().join("nope.fdu")).expect("load must not error");
        assert!(loaded.is_none());
    }

    #[test]
    fn foreign_file_is_treated_as_absent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");
        fs::write(&path, b"this is not a snapshot at all").expect("write");
        assert!(load(&path).expect("load must not error").is_none());
    }

    #[test]
    fn truncated_snapshot_is_treated_as_absent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");
        save(&sample_index(), &path).expect("save");

        let full = fs::read(&path).expect("read");
        for cut in [full.len() - 1, full.len() / 2, MAGIC.len() + 2] {
            fs::write(&path, &full[..cut]).expect("truncate");
            assert!(
                load(&path).expect("load must not error").is_none(),
                "a snapshot truncated to {cut} bytes must not parse"
            );
        }
    }

    #[test]
    fn corrupt_body_with_intact_magic_and_trailer_is_rejected() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");
        save(&sample_index(), &path).expect("save");

        let mut bytes = fs::read(&path).expect("read");
        // Claim far more entries than the body holds.
        let count_at = MAGIC.len() + 4 + 8 + 4 + "/some/root".len();
        bytes[count_at..count_at + 8].copy_from_slice(&u64::MAX.to_le_bytes());
        fs::write(&path, &bytes).expect("write");

        assert!(load(&path).expect("load must not error").is_none());
    }

    #[test]
    fn engine_fingerprint_mismatch_discards_the_snapshot() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");
        save(&sample_index(), &path).expect("save");

        let mut bytes = fs::read(&path).expect("read");
        let fp_at = MAGIC.len() + 4;
        bytes[fp_at] ^= 0xff;
        fs::write(&path, &bytes).expect("write");

        assert!(load(&path).expect("load must not error").is_none());
    }

    #[test]
    fn save_replaces_an_existing_snapshot_and_leaves_no_temp_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");

        save(&sample_index(), &path).expect("first save");
        let mut smaller = Index::new("/some/root");
        smaller.apply(&Delta::new(vec![Op::Upsert {
            path: PathBuf::from("only.txt"),
            kind: EntryKind::File,
            attrs: attrs(1, 1),
        }]));
        save(&smaller, &path).expect("second save");

        let restored = load(&path).expect("load").expect("present");
        assert_eq!(restored.total().files, 1);

        let leftovers: Vec<_> = fs::read_dir(dir.path())
            .expect("read_dir")
            .filter_map(std::result::Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|name| name != "snap.fdu")
            .collect();
        assert!(leftovers.is_empty(), "temp files left behind: {leftovers:?}");
    }

    #[test]
    fn empty_index_round_trips() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");
        let empty = Index::new("/some/root");

        save(&empty, &path).expect("save");
        let restored = load(&path).expect("load").expect("present");
        assert!(restored.is_empty());
        assert_eq!(restored.total(), empty.total());
    }
}
