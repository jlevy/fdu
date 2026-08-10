//! Persisting an index to disk and reading it back.
//!
//! # Status: format v2 is a bounded bootstrap format
//!
//! This module implements a flat, uncompressed writer and a bounded streaming reader.
//! It exists so the cache *lifecycle* — semantic-scope invalidation, atomic replacement,
//! complete-only persistence, resource limits, and corrupt-equals-empty — is nailed down
//! before the optimized wire format is designed. The loader checks file size, trailer,
//! and payload checksum before parsing, then checks the header, record count, and path
//! lengths before allocating record data and rebuilds one entry at a time.
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

use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::index::{EntryId, Index, IndexHandle};
use crate::types::{Attrs, EntryKind, Error, Freshness, Observation, Op, Result, ScanScope};

/// Leading magic. Distinguishes an fdu snapshot from any other file that lands here.
const MAGIC: &[u8; 8] = b"FDUSNAP\x00";

/// Trailing magic. Present only once the whole snapshot has been written, so a truncated
/// file is detected on load rather than parsed into a plausible-looking partial tree.
const TRAILER: &[u8; 8] = b"FDUEND\x00\x00";

/// CRC-32C of every byte before the footer. This detects plausible payload corruption
/// that remains structurally valid; it is an integrity check, not authentication.
const CHECKSUM_BYTES: usize = std::mem::size_of::<u32>();

/// Reversed Castagnoli polynomial used by CRC-32C.
const CRC32C_POLYNOMIAL: u32 = 0x82f6_3b78;

/// One table lookup per byte keeps integrity validation from dominating snapshot load.
const CRC32C_TABLE: [u32; 256] = make_crc32c_table();

/// On-disk format version. Bump on any layout change; old snapshots are then discarded
/// rather than misread.
const FORMAT_VERSION: u32 = 2;

/// Snapshot path encoding used by Unix targets.
#[cfg(unix)]
const PATH_ENCODING_UNIX_BYTES: u8 = 1;

/// Snapshot path encoding used by Windows targets.
#[cfg(windows)]
const PATH_ENCODING_WINDOWS_WIDE: u8 = 2;

/// Portable UTF-8 fallback for targets outside Unix and Windows.
#[cfg(not(any(unix, windows)))]
const PATH_ENCODING_UTF8: u8 = 3;

/// Marks the root's absent parent.
const NO_PARENT: u32 = u32::MAX;

/// Sentinel for an unlimited scan depth in the snapshot header.
const UNLIMITED_DEPTH: u64 = u64::MAX;

/// Scope flag for symlink-following traversal.
const SCOPE_FOLLOW_SYMLINKS: u8 = 1 << 0;

/// Scope flag for staying on the root filesystem.
const SCOPE_ONE_FILESYSTEM: u8 = 1 << 1;

/// All scope bits understood by this format version.
const SCOPE_KNOWN_FLAGS: u8 = SCOPE_FOLLOW_SYMLINKS | SCOPE_ONE_FILESYSTEM;

/// Encoded byte width of the fixed scan-scope header.
#[cfg(test)]
const SERIALIZED_SCOPE_BYTES: usize = 8 + 1 + 8 * 3;

/// Smallest possible on-disk record: parent slot, kind, name length, and six 8-byte
/// attribute fields, with a zero-length name. Used to sanity-check a declared entry
/// count against the bytes actually present.
const MIN_RECORD_BYTES: usize = 4 + 1 + 4 + 8 * 6;

/// Binary size unit used by the snapshot resource limits.
const GIBIBYTE: u64 = 1024 * 1024 * 1024;

/// Largest snapshot image accepted by the bootstrap streaming reader.
const MAX_SNAPSHOT_BYTES: u64 = 64 * GIBIBYTE;

/// Process-local discriminator for exclusive sibling temporary files.
static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

/// Stale files can survive a killed writer. Try enough unique sequence values to step
/// over them without turning an attacker-controlled directory into an unbounded loop.
const MAX_TEMP_CREATE_ATTEMPTS: usize = 1024;

/// Largest encoded root or entry name accepted from a snapshot.
const MAX_PATH_BYTES: u32 = 1024 * 1024;

/// Upper bound on records accepted even when a sparse file could physically hold more.
const MAX_SNAPSHOT_ENTRIES: u64 = 100_000_000;

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
    if index.freshness() != Freshness::Fresh {
        return Err(Error::Snapshot(
            "refusing to persist an index that is stale, reconciling, or partial".into(),
        ));
    }
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(MAGIC);
    buf.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    buf.extend_from_slice(&engine_fingerprint().to_le_bytes());
    buf.push(path_encoding());
    put_scope(&mut buf, index.scope())?;

    put_os_str(&mut buf, index.root_path().as_os_str())?;

    // Pre-order, so a parent's record always precedes its children's and the loader can
    // rebuild the tree in one forward pass with no fixups.
    let mut records: Vec<(u32, EntryId)> = Vec::new();
    let mut stack: Vec<(u32, EntryId)> = vec![(NO_PARENT, EntryId::ROOT)];
    while let Some((parent_slot, id)) = stack.pop() {
        let slot = u32::try_from(records.len())
            .map_err(|_| Error::Snapshot("snapshot exceeds u32 entry capacity".into()))?;
        records.push((parent_slot, id));
        let children = index
            .children_of(id)
            .ok_or_else(|| Error::Snapshot("stale entry handle while saving".into()))?;
        for (_, child) in children.rev() {
            stack.push((slot, child));
        }
    }

    let count = u64::try_from(records.len())
        .map_err(|_| Error::Snapshot("snapshot entry count overflow".into()))?;
    buf.extend_from_slice(&count.to_le_bytes());

    for (parent_slot, id) in records {
        buf.extend_from_slice(&parent_slot.to_le_bytes());
        let kind = index
            .kind_of(id)
            .ok_or_else(|| Error::Snapshot("stale entry handle while saving".into()))?;
        buf.push(kind as u8);
        let name = index
            .name_of(id)
            .ok_or_else(|| Error::Snapshot("stale entry handle while saving".into()))?;
        put_os_str(&mut buf, name)?;
        let attrs = index
            .attrs_of(id)
            .ok_or_else(|| Error::Snapshot("stale entry handle while saving".into()))?;
        buf.extend_from_slice(&attrs.size.to_le_bytes());
        buf.extend_from_slice(&attrs.allocated.to_le_bytes());
        buf.extend_from_slice(&attrs.mtime_ns.to_le_bytes());
        buf.extend_from_slice(&attrs.ctime_ns.to_le_bytes());
        buf.extend_from_slice(&attrs.inode.to_le_bytes());
        buf.extend_from_slice(&attrs.dev.to_le_bytes());
    }
    let checksum = crc32c(&buf);
    buf.extend_from_slice(&checksum.to_le_bytes());
    buf.extend_from_slice(TRAILER);

    write_atomically(path, &buf)
}

/// Capture and persist a coherent shared-index image without holding its lock during
/// serialization or filesystem I/O.
pub fn save_handle(index: &IndexHandle, path: &Path) -> Result<()> {
    let snapshot = index.snapshot()?;
    save(&snapshot, path)
}

/// Load a snapshot, or return `None` when there is nothing usable at `path`.
///
/// Returns `None` — not an error — for a missing file, a foreign file, a version or
/// engine-fingerprint mismatch, and a truncated or corrupt file. Every one of those
/// means the same thing to a caller: there is no warm cache, so scan.
pub fn load(path: &Path) -> Result<Option<Index>> {
    load_with_size_limit(path, MAX_SNAPSHOT_BYTES)
}

fn load_with_size_limit(path: &Path, max_snapshot_bytes: u64) -> Result<Option<Index>> {
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(Error::io(path, e)),
    };
    let file_len = file.metadata().map_err(|e| Error::io(path, e))?.len();
    let footer_bytes = CHECKSUM_BYTES
        .checked_add(TRAILER.len())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(|| Error::Snapshot("snapshot footer size overflow".into()))?;
    if file_len > max_snapshot_bytes || file_len < footer_bytes {
        return Ok(None);
    }

    let footer_offset = i64::try_from(footer_bytes)
        .map_err(|_| Error::Snapshot("snapshot footer size overflow".into()))?;
    file.seek(SeekFrom::End(-footer_offset)).map_err(|e| Error::io(path, e))?;
    let expected_checksum = match read_footer_checksum(&mut file) {
        Ok(checksum) => checksum,
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(Error::io(path, error)),
    };
    let mut trailer = [0u8; TRAILER.len()];
    match file.read_exact(&mut trailer) {
        Ok(()) if &trailer == TRAILER => {}
        Ok(()) => return Ok(None),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(Error::io(path, e)),
    }
    let payload_len = file_len
        .checked_sub(footer_bytes)
        .ok_or_else(|| Error::Snapshot("snapshot length underflow".into()))?;
    file.seek(SeekFrom::Start(0)).map_err(|e| Error::io(path, e))?;
    let actual_checksum = checksum_reader((&mut file).take(payload_len), payload_len)
        .map_err(|error| Error::io(path, error))?;
    if actual_checksum != Some(expected_checksum) {
        return Ok(None);
    }
    file.seek(SeekFrom::Start(0)).map_err(|e| Error::io(path, e))?;
    let mut reader = BufReader::new(file.take(payload_len));
    match parse_stream(&mut reader, payload_len) {
        Ok(index) => Ok(Some(index)),
        Err(ParseError::Invalid) => Ok(None),
        Err(ParseError::Io(source)) => Err(Error::io(path, source)),
    }
}

fn read_footer_checksum(reader: &mut impl Read) -> std::io::Result<u32> {
    let mut bytes = [0u8; CHECKSUM_BYTES];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn checksum_reader(mut reader: impl Read, payload_len: u64) -> std::io::Result<Option<u32>> {
    let mut state = u32::MAX;
    let mut remaining = payload_len;
    let mut buffer = [0u8; 8 * 1024];
    while remaining > 0 {
        let wanted = usize::try_from(remaining).unwrap_or(usize::MAX).min(buffer.len());
        let read = reader.read(&mut buffer[..wanted])?;
        if read == 0 {
            return Ok(None);
        }
        state = crc32c_update(state, &buffer[..read]);
        let read = u64::try_from(read)
            .map_err(|_| std::io::Error::other("checksum read length exceeds u64"))?;
        remaining = remaining
            .checked_sub(read)
            .ok_or_else(|| std::io::Error::other("checksum reader exceeded payload bound"))?;
    }
    Ok(Some(!state))
}

fn crc32c(bytes: &[u8]) -> u32 {
    !crc32c_update(u32::MAX, bytes)
}

fn crc32c_update(mut state: u32, bytes: &[u8]) -> u32 {
    for byte in bytes {
        let index = usize::from(state.to_le_bytes()[0] ^ *byte);
        state = CRC32C_TABLE[index] ^ (state >> 8);
    }
    state
}

const fn make_crc32c_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut index = 0usize;
    let mut value = 0u32;
    while index < table.len() {
        let mut crc = value;
        let mut bit = 0;
        while bit < u8::BITS {
            crc = (crc >> 1) ^ (CRC32C_POLYNOMIAL & 0u32.wrapping_sub(crc & 1));
            bit += 1;
        }
        table[index] = crc;
        index += 1;
        value += 1;
    }
    table
}

/// Parse a bounded payload. Records are applied one at a time so bootstrap paths are not
/// retained in a second full-tree allocation.
fn parse_stream(reader: &mut impl Read, payload_len: u64) -> ParseResult<Index> {
    if read_array::<_, 8>(reader)? != *MAGIC {
        return Err(ParseError::Invalid);
    }
    if read_u32(reader)? != FORMAT_VERSION || read_u64(reader)? != engine_fingerprint() {
        return Err(ParseError::Invalid);
    }
    if read_u8(reader)? != path_encoding() {
        return Err(ParseError::Invalid);
    }
    let scope = read_scope(reader)?;
    let root_path = PathBuf::from(read_os_string(reader)?);
    let count = read_u64(reader)?;
    if count == 0 || count > MAX_SNAPSHOT_ENTRIES {
        return Err(ParseError::Invalid);
    }
    let minimum_body = count
        .checked_mul(u64::try_from(MIN_RECORD_BYTES).map_err(|_| ParseError::Invalid)?)
        .ok_or(ParseError::Invalid)?;
    if minimum_body > payload_len {
        return Err(ParseError::Invalid);
    }

    let mut index = Index::new_with_scope(&root_path, scope);
    let mut ids: Vec<EntryId> = Vec::new();
    for slot in 0..count {
        let parent_slot = read_u32(reader)?;
        let kind = EntryKind::from_u8(read_u8(reader)?).ok_or(ParseError::Invalid)?;
        let name = read_os_string(reader)?;
        let attrs = Attrs {
            size: read_u64(reader)?,
            allocated: read_u64(reader)?,
            mtime_ns: read_i64(reader)?,
            ctime_ns: read_i64(reader)?,
            inode: read_u64(reader)?,
            dev: read_u64(reader)?,
        };

        if parent_slot == NO_PARENT {
            if slot != 0 || kind != EntryKind::Dir || !name.is_empty() {
                return Err(ParseError::Invalid);
            }
            index
                .apply_baseline(&Observation::new(vec![Op::Upsert {
                    path: PathBuf::new(),
                    kind,
                    attrs,
                }]))
                .map_err(|_| ParseError::Invalid)?;
            ids.push(EntryId::ROOT);
            continue;
        }

        let parent = *ids
            .get(usize::try_from(parent_slot).map_err(|_| ParseError::Invalid)?)
            .ok_or(ParseError::Invalid)?;
        if index.kind_of(parent) != Some(EntryKind::Dir) || !is_snapshot_name(&name) {
            return Err(ParseError::Invalid);
        }
        let mut path = index.path_of(parent).ok_or(ParseError::Invalid)?;
        path.push(name);
        if index.lookup(&path).is_some() {
            return Err(ParseError::Invalid);
        }
        index
            .apply_baseline(&Observation::new(vec![Op::Upsert { path: path.clone(), kind, attrs }]))
            .map_err(|_| ParseError::Invalid)?;
        let id = index.lookup(&path).ok_or(ParseError::Invalid)?;
        ids.push(id);
    }

    let mut extra = [0u8; 1];
    if reader.read(&mut extra).map_err(ParseError::Io)? != 0 {
        return Err(ParseError::Invalid);
    }
    Ok(index)
}

#[derive(Debug)]
enum ParseError {
    Invalid,
    Io(std::io::Error),
}

type ParseResult<T> = std::result::Result<T, ParseError>;

fn read_array<R: Read, const N: usize>(reader: &mut R) -> ParseResult<[u8; N]> {
    let mut bytes = [0u8; N];
    match reader.read_exact(&mut bytes) {
        Ok(()) => Ok(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => Err(ParseError::Invalid),
        Err(error) => Err(ParseError::Io(error)),
    }
}

fn read_u8(reader: &mut impl Read) -> ParseResult<u8> {
    Ok(read_array::<_, 1>(reader)?[0])
}

fn read_u32(reader: &mut impl Read) -> ParseResult<u32> {
    Ok(u32::from_le_bytes(read_array(reader)?))
}

fn read_u64(reader: &mut impl Read) -> ParseResult<u64> {
    Ok(u64::from_le_bytes(read_array(reader)?))
}

fn read_i64(reader: &mut impl Read) -> ParseResult<i64> {
    Ok(i64::from_le_bytes(read_array(reader)?))
}

fn read_bytes(reader: &mut impl Read) -> ParseResult<Vec<u8>> {
    let len = read_u32(reader)?;
    if len > MAX_PATH_BYTES {
        return Err(ParseError::Invalid);
    }
    let mut bytes = vec![0u8; usize::try_from(len).map_err(|_| ParseError::Invalid)?];
    match reader.read_exact(&mut bytes) {
        Ok(()) => Ok(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => Err(ParseError::Invalid),
        Err(error) => Err(ParseError::Io(error)),
    }
}

#[cfg(unix)]
fn read_os_string(reader: &mut impl Read) -> ParseResult<OsString> {
    Ok(os_string_from_bytes(&read_bytes(reader)?))
}

#[cfg(not(unix))]
fn read_os_string(reader: &mut impl Read) -> ParseResult<OsString> {
    os_string_from_bytes(&read_bytes(reader)?).ok_or(ParseError::Invalid)
}

fn put_bytes(buf: &mut Vec<u8>, bytes: &[u8]) -> Result<()> {
    let len = u32::try_from(bytes.len())
        .map_err(|_| Error::Snapshot("string too long for snapshot".into()))?;
    if len > MAX_PATH_BYTES {
        return Err(Error::Snapshot("path exceeds snapshot limit".into()));
    }
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(bytes);
    Ok(())
}

fn is_snapshot_name(name: &OsStr) -> bool {
    let mut components = Path::new(name).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

fn put_scope(buf: &mut Vec<u8>, scope: ScanScope) -> Result<()> {
    let max_depth = scope.max_depth.map_or(Ok(UNLIMITED_DEPTH), |depth| {
        u64::try_from(depth).map_err(|_| Error::Snapshot("scan depth overflow".into()))
    })?;
    buf.extend_from_slice(&max_depth.to_le_bytes());
    let mut flags = 0u8;
    if scope.follow_symlinks {
        flags |= SCOPE_FOLLOW_SYMLINKS;
    }
    if scope.one_filesystem {
        flags |= SCOPE_ONE_FILESYSTEM;
    }
    buf.push(flags);
    buf.extend_from_slice(&scope.ignore_rules_fingerprint.to_le_bytes());
    buf.extend_from_slice(&scope.type_rules_fingerprint.to_le_bytes());
    buf.extend_from_slice(&scope.reducers_fingerprint.to_le_bytes());
    Ok(())
}

fn read_scope(reader: &mut impl Read) -> ParseResult<ScanScope> {
    let depth = read_u64(reader)?;
    let max_depth = if depth == UNLIMITED_DEPTH {
        None
    } else {
        Some(usize::try_from(depth).map_err(|_| ParseError::Invalid)?)
    };
    let flags = read_u8(reader)?;
    if flags & !SCOPE_KNOWN_FLAGS != 0 {
        return Err(ParseError::Invalid);
    }
    Ok(ScanScope {
        max_depth,
        follow_symlinks: flags & SCOPE_FOLLOW_SYMLINKS != 0,
        one_filesystem: flags & SCOPE_ONE_FILESYSTEM != 0,
        ignore_rules_fingerprint: read_u64(reader)?,
        type_rules_fingerprint: read_u64(reader)?,
        reducers_fingerprint: read_u64(reader)?,
    })
}

#[cfg(unix)]
fn path_encoding() -> u8 {
    PATH_ENCODING_UNIX_BYTES
}

#[cfg(windows)]
fn path_encoding() -> u8 {
    PATH_ENCODING_WINDOWS_WIDE
}

#[cfg(not(any(unix, windows)))]
fn path_encoding() -> u8 {
    PATH_ENCODING_UTF8
}

#[cfg(unix)]
fn put_os_str(buf: &mut Vec<u8>, value: &OsStr) -> Result<()> {
    use std::os::unix::ffi::OsStrExt;
    put_bytes(buf, value.as_bytes())
}

#[cfg(windows)]
fn put_os_str(buf: &mut Vec<u8>, value: &OsStr) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    let mut bytes = Vec::new();
    for unit in value.encode_wide() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    put_bytes(buf, &bytes)
}

#[cfg(not(any(unix, windows)))]
fn put_os_str(buf: &mut Vec<u8>, value: &OsStr) -> Result<()> {
    let text = value
        .to_str()
        .ok_or_else(|| Error::Snapshot("path is not valid UTF-8 on this platform".into()))?;
    put_bytes(buf, text.as_bytes())
}

#[cfg(unix)]
fn os_string_from_bytes(bytes: &[u8]) -> OsString {
    use std::os::unix::ffi::OsStringExt;
    OsString::from_vec(bytes.to_vec())
}

#[cfg(windows)]
fn os_string_from_bytes(bytes: &[u8]) -> Option<OsString> {
    use std::os::windows::ffi::OsStringExt;
    if !bytes.len().is_multiple_of(std::mem::size_of::<u16>()) {
        return None;
    }
    let units: Vec<u16> = bytes
        .chunks_exact(std::mem::size_of::<u16>())
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    Some(OsString::from_wide(&units))
}

#[cfg(not(any(unix, windows)))]
fn os_string_from_bytes(bytes: &[u8]) -> Option<OsString> {
    Some(OsString::from(String::from_utf8(bytes.to_vec()).ok()?))
}

/// Write to a sibling temporary file, then rename over the target.
///
/// Rename is atomic within a filesystem, so a reader either sees the whole old snapshot
/// or the whole new one. The temporary must be a sibling for that to hold — a rename
/// across filesystems is a copy, and copies are not atomic.
fn write_atomically(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;

    let (tmp, mut file) = create_temp_file(path, parent)?;
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

fn create_temp_file(path: &Path, parent: &Path) -> Result<(PathBuf, fs::File)> {
    for _ in 0..MAX_TEMP_CREATE_ATTEMPTS {
        let sequence = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
        let mut name = OsString::from(".");
        name.push(path.file_name().unwrap_or_else(|| OsStr::new("snapshot")));
        name.push(format!(".tmp.{}.{}", std::process::id(), sequence));
        let tmp = parent.join(name);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&tmp) {
            Ok(file) => return Ok((tmp, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(Error::io(&tmp, error)),
        }
    }
    Err(Error::io(
        parent,
        std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "could not reserve a unique snapshot temporary file",
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::ExtTally;
    use crate::types::Observation;

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
        index.apply_ok(&Observation::new(vec![
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

    fn entry_count_offset(bytes: &[u8]) -> usize {
        let root_len_at = MAGIC.len() + 4 + 8 + 1 + SERIALIZED_SCOPE_BYTES;
        let root_len = u32::from_le_bytes(
            bytes[root_len_at..root_len_at + 4]
                .try_into()
                .expect("saved snapshot has a root length"),
        );
        root_len_at + 4 + usize::try_from(root_len).expect("root length fits usize")
    }

    fn rewrite_checksum(bytes: &mut [u8]) {
        let payload_len = bytes.len() - CHECKSUM_BYTES - TRAILER.len();
        let checksum = crc32c(&bytes[..payload_len]);
        bytes[payload_len..payload_len + CHECKSUM_BYTES].copy_from_slice(&checksum.to_le_bytes());
    }

    #[test]
    fn crc32c_matches_the_standard_check_value() {
        assert_eq!(crc32c(b"123456789"), 0xe306_9283);
    }

    #[test]
    fn round_trip_preserves_tree_and_rollups() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("cache").join("snap.fdu");
        let original = sample_index();

        save(&original, &path).expect("save");
        let restored = load(&path).expect("load").expect("snapshot present");

        assert_eq!(restored.root_path(), Path::new("/some/root"));
        assert_eq!(restored.clock(), crate::Clock::ZERO);
        assert!(restored.since(crate::Clock::ZERO).deltas.is_empty());
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
    fn shared_save_captures_before_filesystem_io() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("shared.fdu");
        let handle = IndexHandle::new(sample_index());

        save_handle(&handle, &path).expect("save shared snapshot");
        handle
            .apply(&Observation::new(vec![Op::Upsert {
                path: PathBuf::from("after.txt"),
                kind: EntryKind::File,
                attrs: attrs(9, 40),
            }]))
            .expect("mutate after capture");

        let restored = load(&path).expect("load").expect("snapshot present");
        assert!(restored.lookup(Path::new("after.txt")).is_none());
        assert!(handle.kind(Path::new("after.txt")).expect("query").is_some());
    }

    #[test]
    fn missing_snapshot_is_absent_not_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let loaded = load(&dir.path().join("nope.fdu")).expect("load must not error");
        assert!(loaded.is_none());
    }

    #[test]
    fn configured_size_limit_rejects_before_body_allocation() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");
        save(&sample_index(), &path).expect("save");
        let file_len = fs::metadata(&path).expect("metadata").len();

        assert!(load_with_size_limit(&path, file_len - 1).expect("load must not error").is_none());
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
        let count_at = entry_count_offset(&bytes);
        bytes[count_at..count_at + 8].copy_from_slice(&u64::MAX.to_le_bytes());
        rewrite_checksum(&mut bytes);
        fs::write(&path, &bytes).expect("write");

        assert!(load(&path).expect("load must not error").is_none());
    }

    #[test]
    fn plausible_attribute_corruption_is_rejected() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");
        save(&sample_index(), &path).expect("save");

        let mut bytes = fs::read(&path).expect("read");
        let records_at = entry_count_offset(&bytes) + 8;
        // The root record has an empty name, so its first attribute starts immediately
        // after parent, kind, and encoded-name length. Changing a low size byte keeps the
        // image structurally valid and would silently alter the restored state without an
        // integrity check.
        let root_size_at = records_at + 4 + 1 + 4;
        bytes[root_size_at] ^= 1;
        fs::write(&path, &bytes).expect("write");

        assert!(load(&path).expect("load must not error").is_none());
    }

    #[test]
    fn entry_names_with_path_components_are_rejected() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");
        save(&sample_index(), &path).expect("save");

        let mut bytes = fs::read(&path).expect("read");
        let count_at = entry_count_offset(&bytes);
        let records_at = count_at + 8;
        let first_child_name_len_at = records_at + MIN_RECORD_BYTES + 4 + 1;
        let old_name_len = u32::from_le_bytes(
            bytes[first_child_name_len_at..first_child_name_len_at + 4]
                .try_into()
                .expect("saved snapshot has a child name length"),
        );
        let old_name_end = first_child_name_len_at
            + 4
            + usize::try_from(old_name_len).expect("name length fits usize");
        let mut invalid_name = Vec::new();
        put_os_str(&mut invalid_name, OsStr::new("../bad")).expect("encode invalid name");
        bytes.splice(first_child_name_len_at..old_name_end, invalid_name);
        rewrite_checksum(&mut bytes);
        fs::write(&path, &bytes).expect("write");

        assert!(load(&path).expect("load must not error").is_none());
    }

    #[test]
    fn oversized_declared_path_is_rejected_before_allocation() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");
        save(&sample_index(), &path).expect("save");

        let mut bytes = fs::read(&path).expect("read");
        let root_len_at = MAGIC.len() + 4 + 8 + 1 + SERIALIZED_SCOPE_BYTES;
        bytes[root_len_at..root_len_at + 4].copy_from_slice(&(MAX_PATH_BYTES + 1).to_le_bytes());
        rewrite_checksum(&mut bytes);
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
        rewrite_checksum(&mut bytes);
        fs::write(&path, &bytes).expect("write");

        assert!(load(&path).expect("load must not error").is_none());
    }

    #[test]
    fn save_replaces_an_existing_snapshot_and_leaves_no_temp_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");

        save(&sample_index(), &path).expect("first save");
        let mut smaller = Index::new("/some/root");
        smaller.apply_ok(&Observation::new(vec![Op::Upsert {
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
    fn concurrent_atomic_writes_do_not_share_a_temporary_file() {
        use std::sync::{Arc, Barrier};

        const WRITERS: usize = 8;
        const PAYLOAD_BYTES: usize = 1024 * 1024;

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");
        let barrier = Arc::new(Barrier::new(WRITERS));
        let results = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..WRITERS)
                .map(|writer| {
                    let barrier = Arc::clone(&barrier);
                    let path = path.clone();
                    scope.spawn(move || {
                        let byte = u8::try_from(writer + 1).expect("writer id fits");
                        let bytes = vec![byte; PAYLOAD_BYTES];
                        barrier.wait();
                        write_atomically(&path, &bytes)
                    })
                })
                .collect();
            handles.into_iter().map(std::thread::ScopedJoinHandle::join).collect::<Vec<_>>()
        });

        for result in results {
            result.expect("writer thread did not panic").expect("concurrent atomic write");
        }
        let final_bytes = fs::read(&path).expect("read final image");
        assert_eq!(final_bytes.len(), PAYLOAD_BYTES);
        assert!(final_bytes.iter().all(|byte| *byte == final_bytes[0]));

        let leftovers: Vec<_> = fs::read_dir(dir.path())
            .expect("read_dir")
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.file_name())
            .filter(|name| name != "snap.fdu")
            .collect();
        assert!(leftovers.is_empty(), "temp files left behind: {leftovers:?}");
    }

    #[test]
    fn concurrent_snapshot_reader_sees_only_a_complete_old_or_new_image() {
        use std::sync::{Arc, Barrier};

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");
        let mut old_index = Index::new("/some/root");
        old_index.apply_ok(&Observation::new(vec![Op::Upsert {
            path: PathBuf::from("old.txt"),
            kind: EntryKind::File,
            attrs: attrs(11, 1),
        }]));
        let mut new_index = Index::new("/some/root");
        new_index.apply_ok(&Observation::new(vec![
            Op::Upsert {
                path: PathBuf::from("new-a.txt"),
                kind: EntryKind::File,
                attrs: attrs(20, 2),
            },
            Op::Upsert {
                path: PathBuf::from("new-b.txt"),
                kind: EntryKind::File,
                attrs: attrs(30, 3),
            },
        ]));
        save(&old_index, &path).expect("save old image");

        let start: Arc<Barrier> = Arc::new(Barrier::new(2));
        let (done_tx, done_rx) = std::sync::mpsc::sync_channel(1);
        std::thread::scope(|scope| {
            let writer_start: Arc<Barrier> = Arc::clone(&start);
            let writer_path: PathBuf = path.clone();
            scope.spawn(move || {
                writer_start.wait();
                done_tx.send(save(&new_index, &writer_path)).expect("report snapshot write");
            });

            let reader_start: Arc<Barrier> = Arc::clone(&start);
            let reader_path: PathBuf = path.clone();
            scope.spawn(move || {
                reader_start.wait();
                let deadline: std::time::Instant =
                    std::time::Instant::now() + std::time::Duration::from_secs(10);
                loop {
                    assert!(
                        std::time::Instant::now() < deadline,
                        "snapshot replacement did not finish before the deadline"
                    );
                    let image: Index =
                        load(&reader_path).expect("load during replacement").expect("image");
                    match image.total().files {
                        1 => {
                            assert_eq!(image.total().bytes, 11);
                            assert!(image.lookup(Path::new("old.txt")).is_some());
                            assert!(image.lookup(Path::new("new-a.txt")).is_none());
                        }
                        2 => {
                            assert_eq!(image.total().bytes, 50);
                            assert!(image.lookup(Path::new("old.txt")).is_none());
                            assert!(image.lookup(Path::new("new-a.txt")).is_some());
                            assert!(image.lookup(Path::new("new-b.txt")).is_some());
                        }
                        partial => panic!("reader observed partial image with {partial} files"),
                    }

                    match done_rx.try_recv() {
                        Ok(result) => {
                            result.expect("replace snapshot");
                            break;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => {}
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            panic!("snapshot writer disconnected")
                        }
                    }
                }
            });
        });

        let final_image: Index = load(&path).expect("load final").expect("final image");
        assert_eq!(final_image.total().files, 2);
        assert_eq!(final_image.total().bytes, 50);
    }

    #[cfg(unix)]
    #[test]
    fn saved_snapshot_is_owner_readable_only() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");
        save(&sample_index(), &path).expect("save");

        let mode = fs::metadata(&path).expect("metadata").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
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

    #[test]
    fn partial_index_is_never_persisted() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("partial.fdu");
        save(&Index::new("/root"), &path).expect("complete baseline");
        let complete_bytes = fs::read(&path).expect("read complete snapshot");

        let mut index = Index::new("/root");
        index.set_initial_freshness(false);

        assert!(matches!(save(&index, &path), Err(Error::Snapshot(_))));
        assert_eq!(fs::read(&path).expect("old snapshot remains"), complete_bytes);
    }

    #[test]
    fn semantic_scan_scope_round_trips() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");
        let scope = ScanScope {
            max_depth: Some(7),
            follow_symlinks: false,
            one_filesystem: true,
            ignore_rules_fingerprint: 11,
            type_rules_fingerprint: 22,
            reducers_fingerprint: 33,
        };
        let index = Index::new_with_scope("/some/root", scope);

        save(&index, &path).expect("save");
        let restored = load(&path).expect("load").expect("present");
        assert_eq!(restored.scope(), scope);
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_names_round_trip_without_aliasing() {
        use std::os::unix::ffi::OsStringExt;

        let first = PathBuf::from(OsString::from_vec(vec![b'n', 0x80]));
        let second = PathBuf::from(OsString::from_vec(vec![b'n', 0x81]));
        let mut root = PathBuf::from("/some");
        root.push(OsString::from_vec(vec![b'r', 0x82]));
        let mut index = Index::new(&root);
        index.apply_baseline_ok(&Observation::new(vec![
            Op::Upsert { path: first.clone(), kind: EntryKind::File, attrs: attrs(10, 1) },
            Op::Upsert { path: second.clone(), kind: EntryKind::File, attrs: attrs(20, 2) },
        ]));

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");
        save(&index, &path).expect("save");
        let restored = load(&path).expect("load").expect("present");

        assert_eq!(restored.root_path(), root);
        assert_eq!(restored.total().files, 2);
        assert_eq!(restored.total().bytes, 30);
        assert!(restored.lookup(&first).is_some());
        assert!(restored.lookup(&second).is_some());
    }
}
