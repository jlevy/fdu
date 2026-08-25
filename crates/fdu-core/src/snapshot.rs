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
use std::io::{self, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::engine_contract::{
    Attrs, EntryKind, Error, Freshness, Observation, Op, Result, ScanScope, Source,
};
use crate::index::{EntryId, Index, IndexHandle};

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
const CRC32C_TABLES: [[u32; 256]; 8] = make_crc32c_tables();

/// On-disk format version. Bump on any layout change; old snapshots are then discarded
/// rather than misread.
///
/// 2: the layout this file's sections describe. 3: [`ScanScope`] carries the hidden-path
/// admission fingerprint, and a section records the directories where the walk saw a
/// governing control file it did not retain.
///
/// The scope record is positional and unlength-prefixed, so a field added to it moves every
/// byte after -- there is no reading a v2 file as a v3 one, and no wanting to. Unlike the
/// leaf-count change, which recompute-at-load absorbed, an admission rule cannot be
/// re-derived from a recording made without it: the entries it would have kept are absent
/// from the file, not from the tree.
const FORMAT_VERSION: u32 = 4;

/// Version of the rules that decide which bucket an entry's bytes are tallied under.
///
/// Separate from [`FORMAT_VERSION`] because the two answer different questions: the
/// layout can be unchanged while the meaning of what was written moves. A snapshot stores
/// the bucket an entry was assigned, not the name it was assigned from, so a rule change
/// cannot be re-derived on load — the entries have to be discarded and re-walked.
///
/// 1: initial rules. 2: files with no extension are tallied under `(none)` instead of
/// being left out of the extension roll-up entirely.
const CLASSIFICATION_VERSION: u32 = 2;

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

/// Sentinel for an absent file budget, matching how depth spells "no bound".
///
/// `u64::MAX` rather than a presence flag beside the value, and for the same reason depth
/// uses it: a cap of `u64::MAX` and no cap admit exactly the same walks, so collapsing
/// them costs nothing anyone can observe -- while a separate flag would create a second
/// spelling for one state, and two spellings for one state is a bug waiting for a reader
/// who checks only one of them.
const UNLIMITED_FILES: u64 = u64::MAX;

/// Scope flag for symlink-following traversal.
const SCOPE_FOLLOW_SYMLINKS: u8 = 1 << 0;

/// Scope flag for staying on the root filesystem.
const SCOPE_ONE_FILESYSTEM: u8 = 1 << 1;

/// All scope bits understood by this format version.
/// The walk excluded entries that are neither files, directories nor symlinks.
///
/// A flag rather than a new field, which the byte had room for. The bit clear means
/// "kept", which is what every snapshot written before this existed meant, so the encoding
/// stayed compatible without a version move.
const SCOPE_EXCLUDE_SPECIAL: u8 = 1 << 2;

const SCOPE_KNOWN_FLAGS: u8 = SCOPE_FOLLOW_SYMLINKS | SCOPE_ONE_FILESYSTEM | SCOPE_EXCLUDE_SPECIAL;

/// Encoded byte width of the fixed scan-scope header.
///
/// Depth, the flag byte, and four fingerprints: tag rules, type rules, reducers, and the
/// hidden-path admission rule.
#[cfg(test)]
const SERIALIZED_SCOPE_BYTES: usize = 8 + 1 + 8 * 4 + 8;

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

/// Per-process entropy mixed into temporary file names.
///
/// Correctness never depended on this: `O_EXCL` is what guarantees one creator wins,
/// and the counter above is what keeps two threads of this process from even trying the
/// same name. Randomness closes two gaps the counter cannot.
///
/// A killed writer leaves its temporary behind and nothing reaps it. With a
/// pid-and-counter name, a later process that recycles that pid restarts its counter at
/// zero and collides with the corpse on every run — correct, because it retries, but it
/// accumulates litter and in the pathological case walks the whole retry budget. And
/// the names are otherwise entirely predictable, which matters because
/// [`MAX_TEMP_CREATE_ATTEMPTS`] exists specifically to survive a hostile directory: an
/// attacker who can guess the names can pre-create the whole budget and deny every save.
///
/// Seeded from `RandomState`, whose keys the standard library takes from the operating
/// system, so this needs no dependency and no clock.
static TEMP_FILE_ENTROPY: std::sync::LazyLock<u64> = std::sync::LazyLock::new(|| {
    use std::hash::{BuildHasher, Hasher};
    std::collections::hash_map::RandomState::new().build_hasher().finish()
});

/// Stale files can survive a killed writer. Try enough unique sequence values to step
/// over them without turning an attacker-controlled directory into an unbounded loop.
const MAX_TEMP_CREATE_ATTEMPTS: usize = 1024;

/// How old an abandoned temporary must be before a later writer removes it.
///
/// Per-process entropy in the name means a corpse can never collide with a future
/// writer — which also means no future writer will ever reuse or overwrite it. Unique
/// names turn an occasional collision into permanent litter, and each corpse is a whole
/// snapshot image, so something has to collect them.
///
/// A day is far longer than any real save and far shorter than "never". The threshold
/// is what makes this safe without any liveness check: a temporary this old cannot
/// belong to a writer that is still running, and pid-based liveness tests are both
/// unportable and wrong under pid reuse.
const STALE_TEMP_AGE: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);

/// Largest encoded root or entry name accepted from a snapshot.
const MAX_PATH_BYTES: u32 = 1024 * 1024;

/// Upper bound on records accepted even when a sparse file could physically hold more.
const MAX_SNAPSHOT_ENTRIES: u64 = 100_000_000;

/// Upper bound on recorded pruned-control-file directories.
///
/// One per directory holding a `.gitignore` that a hidden-path rule pruned, so a real tree
/// produces tens to hundreds. The bound is here because this count is read before anything
/// is allocated against it, and a hostile file should not be able to ask for a gigabyte of
/// paths before a single one is validated.
const MAX_CONTROL_DIRS: u64 = 1_000_000;

/// A fingerprint of everything that would change how the engine interprets a tree.
///
/// When this does not match, the whole snapshot is discarded. That is the cheap,
/// wholesale answer to "the rules changed, so every derived verdict in the cache might
/// be wrong" — far simpler than trying to work out which entries a rule change affected.
///
/// Currently derived from the crate version, the format version, and the version of the
/// classification rules. When compiled type-recognition rules and reducer registrations
/// arrive, their hashes belong here too.
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
    mix(&CLASSIFICATION_VERSION.to_le_bytes());
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

    // Directories where the walk saw a control file it pruned rather than retained. They
    // are not entries, so nothing else in this file records them, and without them a warm
    // start of a pruned tree binds a `gitignore` rule against nothing and answers "nothing
    // is ignored" -- a wrong answer rather than a missing one. Empty for every index that
    // prunes nothing, which is the default, and then this costs eight bytes.
    let control_dirs = index.pruned_control_dirs();
    let control_count = u64::try_from(control_dirs.len())
        .map_err(|_| Error::Snapshot("control directory count overflow".into()))?;
    buf.extend_from_slice(&control_count.to_le_bytes());
    for directory in control_dirs {
        put_os_str(&mut buf, directory.as_os_str())?;
    }

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
    load_with_size_limit(path, MAX_SNAPSHOT_BYTES, None)
}

/// What a caller requires of a snapshot, and the configuration to rebuild it under.
///
/// Both halves matter, and separating them was the bug. The root and scope are the
/// *guard*: a snapshot describing another tree, or one captured under different semantic
/// rules, is not this caller's answer. The registry and tag rules are the
/// *implementation* of the rules that guard identifies -- what canonical extensions,
/// groups and tag bits are actually derived from. A matching fingerprint proves the
/// caller's rules and the snapshot's agree; it does not classify anything by itself.
pub struct LoadRequest<'a> {
    /// The tree the caller means, as an absolute path.
    pub root: &'a Path,
    /// The semantic scope the caller is asking under.
    pub scope: ScanScope,
    /// The type rules to derive canonical extensions and groups with.
    pub types: std::sync::Arc<crate::classify::TypeRegistry>,
    /// The tag rules to evaluate as each entry materializes.
    pub tags: std::sync::Arc<crate::tags::TagRules>,
}

/// Load a snapshot that answers `request`, building it under the request's own rules.
///
/// Returns `None` for a snapshot that is absent, unreadable, or describes a different
/// root or scope -- the last checked from the header, before a single entry is
/// materialized. Loading a whole tree only to discard it was work the format could
/// already have refused.
pub fn load_for(path: &Path, request: &LoadRequest<'_>) -> Result<Option<Index>> {
    load_with_size_limit(path, MAX_SNAPSHOT_BYTES, Some(request))
}

fn load_with_size_limit(
    path: &Path,
    max_snapshot_bytes: u64,
    expect: Option<&LoadRequest<'_>>,
) -> Result<Option<Index>> {
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

    // One pass, not two: the checksum accumulates as the parser consumes, instead of
    // a separate full read of the image before parsing begins. The verdict still
    // gates the data — the parsed index is returned only after the digest over the
    // complete payload matches, so nothing is ever served from bytes that failed
    // their checksum. What changes is the failure mode for structurally-valid
    // corruption: the parser may do work before the mismatch is known, and the
    // result is then discarded. Structural corruption is caught by the parser's own
    // bounds and consistency checks exactly as before, fail-closed either way.
    // The tree was captured shortly before this file was written, so the file's own
    // mtime is the closest "as of" available without a format change. It slightly
    // overstates freshness — the walk began earlier — which is why format v3 should
    // carry the true capture instant and this should read that instead.
    let captured_at_ns = file
        .metadata()
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .and_then(|since| i64::try_from(since.as_nanos()).ok())
        .unwrap_or(0);
    let mut reader = Crc32cReader::new(BufReader::new(file.take(payload_len)));
    let outcome = parse_stream(&mut reader, payload_len, captured_at_ns, expect);
    match outcome {
        Ok(index) => {
            // A successful parse consumed every payload byte (the trailing-byte check
            // proves it), so the running digest covers the whole image.
            if reader.finish() == expected_checksum { Ok(Some(index)) } else { Ok(None) }
        }
        // A mismatch is not corruption, and both are absence: this file is intact and
        // describes some other question. Treating them alike at the boundary is what lets
        // a caller write `if let Some(index)` and not care which happened.
        Err(ParseError::Invalid | ParseError::Mismatch) => Ok(None),
        Err(ParseError::Io(source)) => Err(Error::io(path, source)),
    }
}

/// A reader that folds CRC-32C over every byte the caller consumes.
struct Crc32cReader<R> {
    inner: R,
    state: u32,
}

impl<R: Read> Crc32cReader<R> {
    fn new(inner: R) -> Self {
        Self { inner, state: u32::MAX }
    }

    fn finish(&self) -> u32 {
        !self.state
    }
}

impl<R: Read> Read for Crc32cReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buf)?;
        self.state = crc32c_update(self.state, &buf[..read]);
        Ok(read)
    }
}

fn read_footer_checksum(reader: &mut impl Read) -> std::io::Result<u32> {
    let mut bytes = [0u8; CHECKSUM_BYTES];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

pub(crate) fn crc32c(bytes: &[u8]) -> u32 {
    !crc32c_update(u32::MAX, bytes)
}

/// Slicing-by-8: fold eight input bytes per step through eight derived tables
/// instead of one byte through one. Table 0 is the classic byte table, so the
/// remainder loop and the 8-byte path share one source of truth; the digest is
/// bit-identical to the byte-at-a-time form, which a test asserts over uneven
/// lengths alongside the standard check value.
fn crc32c_update(mut state: u32, bytes: &[u8]) -> u32 {
    let mut chunks = bytes.chunks_exact(8);
    for chunk in &mut chunks {
        let low = (state ^ u32::from_le_bytes(chunk[..4].try_into().expect("chunk holds 8 bytes")))
            .to_le_bytes();
        let high =
            u32::from_le_bytes(chunk[4..].try_into().expect("chunk holds 8 bytes")).to_le_bytes();
        state = CRC32C_TABLES[7][usize::from(low[0])]
            ^ CRC32C_TABLES[6][usize::from(low[1])]
            ^ CRC32C_TABLES[5][usize::from(low[2])]
            ^ CRC32C_TABLES[4][usize::from(low[3])]
            ^ CRC32C_TABLES[3][usize::from(high[0])]
            ^ CRC32C_TABLES[2][usize::from(high[1])]
            ^ CRC32C_TABLES[1][usize::from(high[2])]
            ^ CRC32C_TABLES[0][usize::from(high[3])];
    }
    for byte in chunks.remainder() {
        let index = usize::from(state.to_le_bytes()[0] ^ *byte);
        state = CRC32C_TABLES[0][index] ^ (state >> 8);
    }
    state
}

const fn make_crc32c_tables() -> [[u32; 256]; 8] {
    let mut tables = [[0u32; 256]; 8];
    let mut index = 0usize;
    let mut value = 0u32;
    while index < 256 {
        let mut crc = value;
        let mut bit = 0;
        while bit < u8::BITS {
            crc = (crc >> 1) ^ (CRC32C_POLYNOMIAL & 0u32.wrapping_sub(crc & 1));
            bit += 1;
        }
        tables[0][index] = crc;
        index += 1;
        value += 1;
    }
    // tables[k] advances a byte's contribution k further positions: one more
    // byte-table step applied to the previous table's value.
    let mut table = 1usize;
    while table < tables.len() {
        let mut index = 0usize;
        while index < 256 {
            let previous = tables[table - 1][index];
            tables[table][index] = tables[0][(previous & 0xFF) as usize] ^ (previous >> 8);
            index += 1;
        }
        table += 1;
    }
    tables
}

/// Read only a snapshot's header, without materializing its index.
///
/// Returns `None` for anything this build cannot identify — absent, truncated, foreign,
/// a different format version, or a mismatched engine fingerprint. Corrupt equals
/// absent here exactly as it does on the load path: a caller asking what is in the cache
/// must not be stopped by one unreadable file, and a file this code cannot identify is
/// not a file it should later delete.
pub fn read_header(path: &Path) -> Result<Option<crate::cache::SnapshotInfo>> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(Error::io(path, error)),
    };
    if !has_intact_trailer(&file).map_err(|error| Error::io(path, error))? {
        // Truncation removes the tail and leaves the prologue readable, so a
        // header-only check would call a half-written file a snapshot.
        return Ok(None);
    }
    // The trailer check left the cursor at the end; the header lives at the start.
    let mut file = file;
    file.seek(SeekFrom::Start(0)).map_err(|error| Error::io(path, error))?;

    let mut reader = BufReader::new(file);
    Ok(parse_header(&mut reader).ok())
}

/// Whether a file ends with the snapshot trailer.
///
/// Cheap enough for a status listing — one seek and eight bytes — and it is exactly what
/// a truncated write destroys.
fn has_intact_trailer(file: &fs::File) -> io::Result<bool> {
    let footer_bytes = CHECKSUM_BYTES + TRAILER.len();
    let file_len = file.metadata()?.len();
    if file_len < u64::try_from(footer_bytes).unwrap_or(u64::MAX) {
        return Ok(false);
    }

    let mut handle = file;
    handle.seek(SeekFrom::End(-(i64::try_from(TRAILER.len()).unwrap_or(0))))?;
    let mut trailer = [0u8; TRAILER.len()];
    match handle.read_exact(&mut trailer) {
        Ok(()) => Ok(&trailer == TRAILER),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => Ok(false),
        Err(error) => Err(error),
    }
}

/// Parse the fixed prologue every snapshot begins with.
fn parse_header(reader: &mut impl Read) -> ParseResult<crate::cache::SnapshotInfo> {
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
    let root = PathBuf::from(read_os_string(reader)?);
    // Stepped over rather than kept: this peek answers "which tree, what scope, how big",
    // and none of those depend on where the control files were. It still has to be read,
    // because the entry count is behind it.
    for _ in 0..read_bounded_count(reader, MAX_CONTROL_DIRS)? {
        read_os_string(reader)?;
    }
    let entries = read_u64(reader)?;
    if entries == 0 || entries > MAX_SNAPSHOT_ENTRIES {
        return Err(ParseError::Invalid);
    }
    Ok(crate::cache::SnapshotInfo { root, scope, entries })
}

/// Parse a bounded payload. Records are applied one at a time so bootstrap paths are not
/// retained in a second full-tree allocation.
fn parse_stream(
    reader: &mut impl Read,
    payload_len: u64,
    captured_at_ns: i64,
    expect: Option<&LoadRequest<'_>>,
) -> ParseResult<Index> {
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
    let mut control_dirs: Vec<PathBuf> = Vec::new();
    for _ in 0..read_bounded_count(reader, MAX_CONTROL_DIRS)? {
        control_dirs.push(PathBuf::from(read_os_string(reader)?));
    }
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

    // Refuse before materializing, not after. The caller's guard used to run on the
    // finished index, so a snapshot belonging to another tree was fully parsed, fully
    // allocated, and then dropped -- the most expensive possible way to say "not this
    // one". Everything needed to answer is in the header just read.
    if let Some(expect) = expect
        && (root_path != expect.root || scope != expect.scope)
    {
        return Err(ParseError::Mismatch);
    }

    let mut index = Index::new_with_scope(&root_path, scope);
    // Where the walk that wrote this file saw a control file it pruned. The entries are
    // gone -- that is what pruning means -- so this is the only record, and binding a
    // Path-tier rule after a warm start has nothing else to go on.
    index.adopt_pruned_control_dirs(control_dirs);
    // Install the caller's semantics before the first entry exists, so every canonical
    // extension, group and tag bit below is derived under the rules the scope claims.
    // Swapping the pointers afterwards left the derived state built by whatever the
    // default happened to be, while the fingerprint said otherwise -- a warm answer that
    // disagreed with a cold one about the same tree.
    let needs_paths = if let Some(expect) = expect {
        let needs_paths = expect.tags.needs_path();
        index.install_semantics(expect.types.clone(), expect.tags.clone());
        needs_paths
    } else {
        false
    };
    // One `PathBuf` per slot, and only when some enabled rule reads one. Each is a single
    // join off the parent's, which the loader already holds -- the ancestor walk per
    // record that this function's whole shape exists to avoid. Files get the empty
    // `PathBuf`, which does not allocate.
    let mut paths: Vec<PathBuf> = Vec::new();
    if needs_paths {
        paths.reserve(usize::try_from(count).unwrap_or(0));
    }
    // Everything this loader inserts describes the tree as the snapshot found it, not
    // as this process has seen it. Stamping the entries `Cached` is what lets a
    // consumer paint them immediately and label them honestly; without it a loaded
    // index claims to be fresh when nothing has been checked since the file was read.
    index.set_applying_source(Source::Cached, captured_at_ns);
    // The record count is validated against the bytes actually present above, so it is
    // safe to size from: reserving here removes the geometric regrowth of a 450k-element
    // vector without letting a corrupt count drive the allocation.
    let mut ids: Vec<EntryId> = Vec::with_capacity(usize::try_from(count).unwrap_or(0));
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
            if needs_paths {
                paths.push(PathBuf::new());
            }
            continue;
        }

        let parent = *ids
            .get(usize::try_from(parent_slot).map_err(|_| ParseError::Invalid)?)
            .ok_or(ParseError::Invalid)?;
        if !is_snapshot_name(&name) {
            return Err(ParseError::Invalid);
        }
        // The parent's id is already in hand, so the record is inserted straight beneath
        // it. Resolving a path to rediscover that parent, and then searching the parent's
        // children to rediscover the id just created, were both work the format had
        // already answered. A snapshot naming the same path twice, or parenting an entry
        // to a non-directory, is corrupt, and `insert_loaded_child` fails closed on both.
        let relative_path = needs_paths.then(|| {
            crate::counters::bump(|c| c.loader_paths_built += 1);
            let parent_path = paths
                .get(usize::try_from(parent_slot).unwrap_or(usize::MAX))
                .map_or(Path::new(""), PathBuf::as_path);
            parent_path.join(&name)
        });
        let id = index
            .insert_loaded_child(parent, name, kind, attrs, relative_path.as_deref())
            .ok_or(ParseError::Invalid)?;
        ids.push(id);
        if needs_paths {
            // Only a directory is ever asked for its path again, so a file's slot holds
            // the empty one rather than a copy nothing reads.
            paths.push(if kind.is_dir() {
                relative_path.unwrap_or_default()
            } else {
                PathBuf::new()
            });
        }
    }

    let mut extra = [0u8; 1];
    if reader.read(&mut extra).map_err(ParseError::Io)? != 0 {
        return Err(ParseError::Invalid);
    }
    // Once, rather than once per record: the loaded tree is the process baseline, and
    // nothing it inserted was journalled.
    index.establish_baseline();
    // Anything applied after the load is this process checking what the snapshot
    // claimed, which is a revalidation rather than a first sighting.
    index.set_applying_source(Source::Revalidated, 0);
    Ok(index)
}

#[derive(Debug)]
enum ParseError {
    Invalid,
    /// Intact, well-formed, and about a different root or scope.
    ///
    /// Distinct from `Invalid` so the code says which happened even though the boundary
    /// answers `None` to both: calling a perfectly good snapshot corrupt because it
    /// belongs to another tree would mislead the next reader of this file.
    Mismatch,
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

/// Read a count that a caller is about to allocate against, refusing an absurd one.
///
/// Checked before the allocation rather than after, which is the same discipline the entry
/// count follows: a file claiming four billion of anything should cost one comparison to
/// reject, not a reservation.
fn read_bounded_count(reader: &mut impl Read, maximum: u64) -> ParseResult<u64> {
    let count = read_u64(reader)?;
    if count > maximum {
        return Err(ParseError::Invalid);
    }
    Ok(count)
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
    if scope.exclude_special {
        flags |= SCOPE_EXCLUDE_SPECIAL;
    }
    buf.push(flags);
    buf.extend_from_slice(&scope.tag_rules_fingerprint.to_le_bytes());
    buf.extend_from_slice(&scope.type_rules_fingerprint.to_le_bytes());
    buf.extend_from_slice(&scope.reducers_fingerprint.to_le_bytes());
    buf.extend_from_slice(&scope.hidden_fingerprint.to_le_bytes());
    buf.extend_from_slice(&scope.max_files.unwrap_or(UNLIMITED_FILES).to_le_bytes());
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
        exclude_special: flags & SCOPE_EXCLUDE_SPECIAL != 0,
        tag_rules_fingerprint: read_u64(reader)?,
        type_rules_fingerprint: read_u64(reader)?,
        reducers_fingerprint: read_u64(reader)?,
        hidden_fingerprint: read_u64(reader)?,
        max_files: match read_u64(reader)? {
            UNLIMITED_FILES => None,
            // Zero is rejected at config validation, so a zero here is a corrupt or
            // hand-edited file rather than a scope anyone could have asked for.
            0 => return Err(ParseError::Invalid),
            cap => Some(cap),
        },
    })
}

#[cfg(unix)]
pub(crate) fn path_encoding() -> u8 {
    PATH_ENCODING_UNIX_BYTES
}

#[cfg(windows)]
pub(crate) fn path_encoding() -> u8 {
    PATH_ENCODING_WINDOWS_WIDE
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn path_encoding() -> u8 {
    PATH_ENCODING_UTF8
}

#[cfg(unix)]
pub(crate) fn put_os_str(buf: &mut Vec<u8>, value: &OsStr) -> Result<()> {
    use std::os::unix::ffi::OsStrExt;
    put_bytes(buf, value.as_bytes())
}

#[cfg(windows)]
pub(crate) fn put_os_str(buf: &mut Vec<u8>, value: &OsStr) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    let mut bytes = Vec::new();
    for unit in value.encode_wide() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    put_bytes(buf, &bytes)
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn put_os_str(buf: &mut Vec<u8>, value: &OsStr) -> Result<()> {
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
    // Available now the floor is 1.88 — see the note in content_cache.rs.
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
pub(crate) fn write_atomically(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = parent_dir(path);
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
    reap_stale_temporaries(parent, path, STALE_TEMP_AGE);
    Ok(())
}

/// Remove long-abandoned temporaries beside `path`.
///
/// Best effort and deliberately silent: this is housekeeping, and a snapshot that was
/// written correctly must not fail because a directory could not be tidied. Anything
/// that cannot be read or removed is left for the next writer.
fn reap_stale_temporaries(parent: &Path, path: &Path, older_than: std::time::Duration) {
    let Some(prefix) = temp_prefix(path) else { return };
    let Ok(entries) = fs::read_dir(parent) else { return };
    let now = std::time::SystemTime::now();
    for entry in entries.flatten() {
        let name = entry.file_name();
        if !name.as_encoded_bytes().starts_with(prefix.as_encoded_bytes()) {
            continue;
        }
        let stale = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .ok()
            .and_then(|modified| now.duration_since(modified).ok())
            .is_some_and(|age| age >= older_than);
        if stale {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// The shared leading portion of every temporary written for `path`.
///
/// Matching on this rather than on a full parse keeps the reaper from depending on the
/// pid, entropy, and sequence encoding: a temporary written by an older build with a
/// different suffix layout is still recognised and still collected.
fn temp_prefix(path: &Path) -> Option<OsString> {
    let mut prefix = OsString::from(".");
    prefix.push(path.file_name()?);
    prefix.push(".tmp.");
    Some(prefix)
}

/// The directory a target lives in, as a path that can actually be opened.
///
/// A bare relative target such as `snap.fdu` has an *empty* parent rather than none, and
/// the empty path is not the current directory: `create_dir_all` and `rename` tolerate
/// it, but `read_dir` does not — which silently disabled the reaper for exactly those
/// targets while the write itself kept working.
fn parent_dir(path: &Path) -> &Path {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    }
}

/// The temporary name this process uses for `path` at `sequence`.
///
/// Split out so a test can plant a collision at a name the writer will actually try.
/// Guessing the shape does not work — the entropy is per process — and getting that
/// wrong is how the first version of the stale-temporary test came to exercise nothing.
fn temp_name(path: &Path, sequence: u64) -> OsString {
    let mut name = OsString::from(".");
    name.push(path.file_name().unwrap_or_else(|| OsStr::new("snapshot")));
    name.push(format!(".tmp.{}.{:016x}.{}", std::process::id(), *TEMP_FILE_ENTROPY, sequence));
    name
}

fn create_temp_file(path: &Path, parent: &Path) -> Result<(PathBuf, fs::File)> {
    for _ in 0..MAX_TEMP_CREATE_ATTEMPTS {
        let sequence = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
        let tmp = parent.join(temp_name(path, sequence));
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
    use crate::engine_contract::Observation;
    use crate::index::ExtTally;

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
    fn crc32c_slicing_matches_the_byte_reference_on_uneven_lengths() {
        // The 8-byte path and the remainder loop must agree with the classic
        // byte-at-a-time recurrence at every alignment, or an old snapshot's digest
        // stops verifying. The reference below IS that recurrence, against table 0.
        fn reference(bytes: &[u8]) -> u32 {
            let mut state = u32::MAX;
            for byte in bytes {
                let index = usize::from(state.to_le_bytes()[0] ^ *byte);
                state = CRC32C_TABLES[0][index] ^ (state >> 8);
            }
            !state
        }
        let mut data = Vec::new();
        let mut seed = 0x9e37_79b9u32;
        for length in [0usize, 1, 7, 8, 9, 15, 16, 63, 64, 65, 1000] {
            data.clear();
            for _ in 0..length {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                data.push(seed.to_le_bytes()[0]);
            }
            assert_eq!(crc32c(&data), reference(&data), "length {length}");
        }
    }

    #[test]
    fn a_loaded_index_reports_cached_provenance_not_fresh() {
        // The gap that motivated the provenance model. A snapshot is complete when it
        // is written, so a loaded index used to claim `Fresh` — true of when the file
        // was made, and exactly backwards for a consumer painting on load, which needs
        // to know nothing has been checked since.
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snapshot.fdu");
        let original = sample_index();
        save(&original, &path).expect("save");

        let restored = load(&path).expect("load").expect("snapshot present");
        let provenance =
            restored.provenance(Path::new("src/main.rs")).expect("the loaded entry is present");
        assert_eq!(provenance.source, crate::Source::Cached);
        assert!(!provenance.is_verified(), "nothing has been stat'd since the load");
        assert!(
            provenance.observed_at_ns > 0,
            "a cached value must say as of when, or a UI cannot label it"
        );

        // A freshly scanned index is the contrasting case.
        assert_eq!(
            original.provenance(Path::new("src/main.rs")).expect("present").source,
            crate::Source::Scanned
        );
    }

    #[test]
    fn a_loaded_root_reports_cached_provenance_not_fresh() {
        // The root is the one entry the child-only test above cannot cover, and it is
        // the one that matters most: whole-tree totals hang off it, so `fdu ~` reads
        // the root's provenance to label its headline number. `apply_upsert` handles
        // the root in a separate branch, and that branch used to skip the source stamp
        // entirely — leaving a snapshot-loaded root claiming `Scanned` and
        // `is_verified()`, the precise silent lie the model exists to prevent.
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snapshot.fdu");
        save(&sample_index(), &path).expect("save");

        let restored = load(&path).expect("load").expect("snapshot present");
        let provenance = restored.provenance(Path::new("")).expect("the root is always present");
        assert_eq!(provenance.source, crate::Source::Cached, "the root came off disk too");
        assert!(!provenance.is_verified(), "nothing has been stat'd since the load");
        assert!(
            provenance.observed_at_ns > 0,
            "a cached total must say as of when, or a UI cannot label it"
        );
    }

    #[test]
    fn revalidating_a_loaded_index_promotes_entries_out_of_cached() {
        // The failure a reviewer caught on PR #6: entries were stamped only when
        // allocated, so a warm sweep could stat every entry and leave them all
        // reporting `Cached`. A consumer could then never clear a stale-value
        // indicator no matter how much verification ran, which defeats the point.
        let dir = tempfile::tempdir().expect("tempdir");
        let tree = dir.path().join("tree");
        std::fs::create_dir_all(tree.join("sub")).expect("create dirs");
        std::fs::write(tree.join("sub/file.txt"), b"contents").expect("write");
        let snapshot_path = dir.path().join("snapshot.fdu");

        let config = crate::ScanConfig::default();
        let (original, report) = crate::scan::scan_into_index(&tree, &config).expect("scan");
        assert!(report.is_complete());
        save(&original, &snapshot_path).expect("save");

        let mut restored = load(&snapshot_path).expect("load").expect("present");
        let target = Path::new("sub/file.txt");
        assert_eq!(
            restored.provenance(target).expect("present").source,
            crate::Source::Cached,
            "straight off disk, nothing has been checked"
        );

        // A sweep that finds nothing changed still verified every entry it stat'd.
        let reconciled =
            crate::scan::reconcile(&mut restored, &config, &mut |_| {}).expect("reconcile");
        assert!(reconciled.is_complete());
        assert_eq!(reconciled.apply.updated, 0, "the tree did not change");

        let provenance = restored.provenance(target).expect("present");
        assert_eq!(
            provenance.source,
            crate::Source::Revalidated,
            "an unchanged entry that was freshly stat'd has still been verified"
        );
        assert!(provenance.is_verified());
    }

    /// Every entry, not just the root.
    ///
    /// The loader inserts beneath a known parent instead of replaying an observation, so
    /// it no longer shares a code path with the producer that built the original. Root
    /// totals cannot catch a roll-up that is wrong halfway down — the errors would have
    /// to cancel at the root to hide, but a single misplaced subtree does not touch the
    /// root at all. This walks both trees and compares each directory's own roll-up,
    /// each entry's kind, attributes and extension tallies, and the shape of the tree.
    #[test]
    fn round_trip_preserves_every_entry_not_just_the_root() {
        let dir = tempfile::tempdir().expect("tempdir");
        let tree = dir.path().join("tree");
        let snapshot_path = dir.path().join("cache").join("snap.fdu");
        // Depth and fan-out both matter: a parent-relative insert that mis-parents an
        // entry shows up as a wrong roll-up on an interior directory.
        for (relative, contents) in [
            ("a.rs", &b"fn main() {}"[..]),
            ("deep/one/two/three/leaf.txt", b"leaf"),
            ("deep/one/two/sibling.rs", b"sibling"),
            ("deep/one/other.md", b"# other"),
            ("wide/w1.txt", b"1"),
            ("wide/w2.txt", b"22"),
            ("wide/w3.rs", b"333"),
            ("empty/.keep", b""),
        ] {
            let path = tree.join(relative);
            fs::create_dir_all(path.parent().expect("parent")).expect("create dirs");
            fs::write(&path, contents).expect("write");
        }

        let config = crate::ScanConfig::default();
        let (original, report) = crate::scan::scan_into_index(&tree, &config).expect("scan");
        assert!(report.is_complete());
        save(&original, &snapshot_path).expect("save");
        let restored = load(&snapshot_path).expect("load").expect("present");

        assert_eq!(restored.len(), original.len(), "entry count");

        // Walk the original and demand the same entry, in the same place, with the same
        // numbers, in the restored index.
        let mut stack = vec![crate::index::EntryId::ROOT];
        let mut compared = 0_u64;
        while let Some(id) = stack.pop() {
            let path = original.path_of(id).expect("original path");
            let mirrored = restored.lookup(&path).expect("restored entry at the same path");
            assert_eq!(
                original.kind_of(id),
                restored.kind_of(mirrored),
                "kind at {}",
                path.display()
            );
            assert_eq!(
                original.attrs_of(id),
                restored.attrs_of(mirrored),
                "attrs at {}",
                path.display()
            );
            // Roll-ups and children exist only on directories; a file legitimately has
            // neither, and demanding them would fail on the tree rather than the loader.
            if original.kind_of(id) == Some(crate::EntryKind::Dir) {
                let (before, after) = (
                    original.rollup_of(id).expect("original rollup"),
                    restored.rollup_of(mirrored).expect("restored rollup"),
                );
                assert_eq!(
                    (
                        before.files,
                        before.dirs,
                        before.bytes,
                        before.allocated,
                        before.newest_mtime_ns
                    ),
                    (after.files, after.dirs, after.bytes, after.allocated, after.newest_mtime_ns),
                    "rollup at {}",
                    path.display()
                );
                assert_eq!(before.by_ext, after.by_ext, "extension tallies at {}", path.display());
                let names: Vec<_> = original
                    .children_of(id)
                    .expect("original children")
                    .map(|(name, _)| name.to_os_string())
                    .collect();
                let mirrored_names: Vec<_> = restored
                    .children_of(mirrored)
                    .expect("restored children")
                    .map(|(name, _)| name.to_os_string())
                    .collect();
                assert_eq!(names, mirrored_names, "children of {}", path.display());
                stack.extend(original.children_of(id).expect("children").map(|(_, child)| child));
            }
            compared += 1;
        }
        assert_eq!(compared, original.len(), "every entry was compared");

        // The loader must not leave the index looking like it has pending history.
        assert_eq!(restored.clock(), crate::Clock::ZERO);
        assert!(
            restored
                .since(crate::Cursor::start(restored.session()))
                .expect("own session")
                .deltas
                .is_empty()
        );
    }

    /// A warm answer and a cold answer must agree about the same tree, under whatever
    /// taxonomy the caller supplied.
    ///
    /// This is the defect the two-phase load exists to prevent, and it is invisible to
    /// every test that uses the compiled registry: the loader built canonical extensions
    /// and groups with `TypeRegistry::compiled()` and the caller's registry was swapped in
    /// afterwards, so the scope fingerprint said "custom rules" while the derived state
    /// said "the default". The fixture therefore uses a registry that classifies `rs` as
    /// prose, which no default ever would -- if the load path regresses, the warm answer
    /// silently reverts to `rust` and this fails.
    #[test]
    fn a_warm_load_derives_under_the_callers_registry_not_the_compiled_one() {
        let dir = tempfile::tempdir().expect("tempdir");
        let tree = dir.path().join("tree");
        let snapshot_path = dir.path().join("cache").join("snap.fdu");
        for relative in ["main.rs", "src/lib.rs", "notes.md"] {
            let path = tree.join(relative);
            fs::create_dir_all(path.parent().expect("parent")).expect("create dirs");
            fs::write(&path, b"x").expect("write");
        }

        let registry = std::sync::Arc::new(
            crate::classify::TypeRegistry::from_manifest(
                "[[group]]\nid = \"scribbles\"\nlabel = \"Scribbles\"\norder = 1\n\
                 [[kind]]\nid = \"notes\"\nfamily = \"prose\"\ngroup = \"scribbles\"\n\
                 extensions = [\"rs\"]\n",
            )
            .expect("a minimal manifest"),
        );
        let config = crate::ScanConfig { types: Some(registry.clone()), ..Default::default() };

        let (cold, report) = crate::scan::scan_into_index(&tree, &config).expect("scan");
        assert!(report.is_complete());
        save(&cold, &snapshot_path).expect("save");

        let warm = load_for(
            &snapshot_path,
            &LoadRequest {
                root: cold.root_path(),
                scope: config.scope(),
                types: registry.clone(),
                tags: config.tags(),
            },
        )
        .expect("load")
        .expect("the snapshot answers this scope");

        // The group totals are the derived state the loader gets wrong when it classifies
        // under the wrong registry, and they are what a consumer actually reads.
        let cold_groups = cold.rollup_of(EntryId::ROOT).expect("cold root").by_group;
        let warm_groups = warm.rollup_of(EntryId::ROOT).expect("warm root").by_group;
        assert_eq!(warm_groups, cold_groups, "warm and cold must agree on group totals");
        assert!(
            cold_groups.keys().any(|group| group == "scribbles"),
            "the fixture registry must actually be in play: {cold_groups:?}"
        );

        // And the same for the extension buckets, which key off the canonical extension.
        assert_eq!(
            warm.rollup_of(EntryId::ROOT).expect("warm root").by_ext,
            cold.rollup_of(EntryId::ROOT).expect("cold root").by_ext,
        );
    }

    /// A Name-tier warm load builds no paths, and a Path-tier one builds exactly one each.
    ///
    /// The load path's whole advantage over the observation path is that it holds a parent
    /// id and a basename per record and never resolves a path. Tagging threatened that
    /// twice: first by evaluating rules against a path joined from an ancestor walk, then
    /// by re-traversing the finished index to tag it. Both are gone, and this is what says
    /// so in numbers rather than in a comment -- the counter is the claim.
    #[test]
    fn a_warm_load_builds_no_paths_for_name_tier_rules() {
        let _serial = crate::counters::test_serial();
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("cache").join("snap.fdu");
        let original = sample_index();
        save(&original, &path).expect("save");

        let request = |tags: std::sync::Arc<crate::tags::TagRules>| LoadRequest {
            root: Path::new("/some/root"),
            scope: ScanScope { tag_rules_fingerprint: tags.fingerprint(), ..original.scope() },
            types: crate::classify::TypeRegistry::compiled().clone(),
            tags,
        };

        // The scope in the file was written with no tag rules, so only that request
        // matches; the Path-tier arm below is measured through `parse_stream` directly,
        // where the guard is not in the way of the question being asked.
        crate::counters::enable(true);
        crate::counters::reset();
        let loaded = load_for(&path, &request(crate::tags::TagRules::none().clone().into()))
            .expect("load")
            .expect("present");
        let built = crate::counters::snapshot().loader_paths_built;
        crate::counters::enable(false);

        assert_eq!(loaded.len(), original.len(), "the load must actually have happened");
        assert_eq!(built, 0, "no enabled rule reads a path, so the loader must not construct one");
    }

    /// A snapshot for another tree or another scope is refused from its header.
    ///
    /// Not merely refused -- refused *before* materializing. Loading half a million
    /// entries to discover the file answers a different question was the most expensive
    /// possible way to say no, and the header carries everything needed to say it.
    #[test]
    fn a_snapshot_for_another_root_or_scope_is_refused_without_materializing_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("cache").join("snap.fdu");
        let original = sample_index();
        save(&original, &path).expect("save");

        let matching = LoadRequest {
            root: Path::new("/some/root"),
            scope: original.scope(),
            types: crate::classify::TypeRegistry::compiled().clone(),
            tags: crate::tags::TagRules::none().clone().into(),
        };
        assert!(
            load_for(&path, &matching).expect("load").is_some(),
            "the control: this request is the one the snapshot answers"
        );

        let elsewhere = LoadRequest { root: Path::new("/other/root"), ..matching };
        assert!(load_for(&path, &elsewhere).expect("load").is_none(), "a different root");

        let other_scope = ScanScope { max_depth: Some(3), ..original.scope() };
        let narrowed = LoadRequest { scope: other_scope, ..elsewhere };
        let narrowed = LoadRequest { root: Path::new("/some/root"), ..narrowed };
        assert!(load_for(&path, &narrowed).expect("load").is_none(), "a different scope");
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
        assert!(
            restored
                .since(crate::Cursor::start(restored.session()))
                .expect("own session")
                .deltas
                .is_empty()
        );
        assert_eq!(restored.len(), original.len());
        // Public roll-ups resolve assignment-ordered extension ids to stable names.
        let (restored_total, original_total) = (restored.total(), original.total());
        assert_eq!(
            (restored_total.files, restored_total.dirs, restored_total.bytes),
            (original_total.files, original_total.dirs, original_total.bytes)
        );
        assert_eq!(restored_total.allocated, original_total.allocated);
        assert_eq!(restored_total.newest_mtime_ns, original_total.newest_mtime_ns);
        assert_eq!(restored_total.by_ext, original_total.by_ext);
        assert_eq!(restored.total().files, 3);
        assert_eq!(restored.total().dirs, 2);
        assert_eq!(restored.total().bytes, 157);
        assert_eq!(
            restored.total().by_ext[".rs"],
            ExtTally { files: 2, bytes: 150, allocated: 1024 }
        );
        assert_eq!(
            restored.attrs(Path::new("src/deep/nested.rs")),
            original.attrs(Path::new("src/deep/nested.rs"))
        );
    }

    #[test]
    fn round_trip_handles_wide_directory_fanout() {
        // Load resolves each record's id from its parent. Doing that by scanning the
        // parent's children costs O(width^2) per directory, which is invisible on the
        // handful of siblings every other test uses and dominates a real one — a
        // node_modules or a Maildir is thousands wide.
        const CHILDREN: u64 = 4_096;
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("wide.fdu");
        let mut original = Index::new("/some/root");
        let ops = (0..CHILDREN)
            .map(|sequence| Op::Upsert {
                path: PathBuf::from(format!("child-{sequence:04}.dat")),
                kind: EntryKind::File,
                attrs: attrs(sequence + 1, i64::try_from(sequence).expect("fanout fits i64")),
            })
            .collect();
        original.apply_baseline_ok(&Observation::new(ops));

        save(&original, &path).expect("save wide snapshot");
        let restored = load(&path).expect("load wide snapshot").expect("snapshot present");

        assert_eq!(restored.total().files, CHILDREN);
        assert_eq!(restored.len(), original.len());
        // The last child is the one a linear scan reaches last, so it is the one that
        // proves the resolution used the parent's map rather than its sibling order.
        assert_eq!(
            restored.attrs(Path::new("child-4095.dat")),
            original.attrs(Path::new("child-4095.dat"))
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

        assert!(
            load_with_size_limit(&path, file_len - 1, None).expect("load must not error").is_none()
        );
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
    fn an_abandoned_temporary_is_collected_once_it_is_old_enough() {
        // Per-process entropy means no future writer will ever generate a corpse's
        // name again, so nothing would otherwise reclaim it: unique names turn an
        // occasional collision into permanent litter, one whole snapshot image at a
        // time. The reaper closes that, and the age threshold is what makes it safe
        // without a liveness check — pid-based liveness is unportable and wrong under
        // pid reuse.
        //
        // The threshold is a parameter so both sides are testable without setting
        // mtimes, which the standard library cannot do and which is not worth a
        // dependency.
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");
        let prefix = temp_prefix(&path).expect("a named target has a prefix");

        let mut corpse = prefix.clone();
        corpse.push("111.0123456789abcdef.7");
        let unrelated = OsString::from("notes.txt");
        fs::write(dir.path().join(&corpse), b"a writer killed long ago").expect("plant");
        fs::write(dir.path().join(&unrelated), b"not ours").expect("plant");

        // The shipped threshold spares anything that could still be in flight.
        reap_stale_temporaries(dir.path(), &path, STALE_TEMP_AGE);
        assert!(dir.path().join(&corpse).exists(), "a fresh corpse must be left alone");

        // Past the threshold it is collected, and only it.
        reap_stale_temporaries(dir.path(), &path, std::time::Duration::ZERO);
        assert!(!dir.path().join(&corpse).exists(), "an old corpse must be collected");
        assert!(dir.path().join(&unrelated).exists(), "unrelated files are never touched");
    }

    #[test]
    fn the_reaper_only_matches_its_own_targets_temporaries() {
        // The prefix carries the target's file name, so two snapshots sharing a
        // directory cannot collect each other's work in progress.
        let mine = temp_prefix(Path::new("/cache/snap.fdu")).expect("prefix");
        let theirs = temp_prefix(Path::new("/cache/other.fdu")).expect("prefix");
        assert_eq!(mine, OsString::from(".snap.fdu.tmp."));
        assert_ne!(mine, theirs);
        assert!(temp_prefix(Path::new("/")).is_none(), "a rootless path has no name");
    }

    #[test]
    fn a_stale_temporary_does_not_block_a_later_write() {
        // A killed writer leaves its temporary behind and nothing reaps it until it is
        // a day old, so a later write can find a corpse sitting exactly where it wants
        // to go. `O_EXCL` plus the retry loop is what makes that safe: the collision is
        // detected and stepped over, never shared.
        //
        // The corpse is planted at names this process will really try, via `temp_name`.
        // An earlier version of this test guessed the shape and planted
        // `.snap.fdu.tmp.{pid}.0`, which the entropy in the name means the writer can
        // never generate — so it collided with nothing and proved nothing, while
        // reading as though it had.
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.fdu");

        // Block a window of upcoming sequence values. A window rather than one name
        // because the counter is process-global and other tests draw from it too;
        // whichever value this write lands on, it starts inside the blocked range.
        let first = NEXT_TEMP_FILE.load(Ordering::Relaxed);
        let planted: Vec<OsString> =
            (first..first + 32).map(|sequence| temp_name(&path, sequence)).collect();
        for name in &planted {
            fs::write(dir.path().join(name), b"corpse").expect("plant a stale temporary");
        }

        write_atomically(&path, b"payload").expect("write past the stale temporaries");
        assert_eq!(fs::read(&path).expect("read back"), b"payload");

        // Every corpse survives: the writer stepped over them rather than reusing or
        // removing one, and they are far too young for the reaper.
        let mut survivors: Vec<_> = fs::read_dir(dir.path())
            .expect("read_dir")
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.file_name())
            .filter(|name| name != "snap.fdu")
            .collect();
        survivors.sort();
        let mut expected = planted;
        expected.sort();
        assert_eq!(survivors, expected, "the write must step over corpses, not consume them");
    }

    #[test]
    fn a_bare_filename_target_resolves_to_an_openable_directory() {
        // `Path::parent` returns `Some("")` for a bare name, not `None`, so an
        // `unwrap_or(".")` fallback never fires. The write still worked — `rename`
        // accepts the empty path — but `read_dir("")` fails, so the reaper returned
        // immediately and collected nothing for those targets.
        assert_eq!(parent_dir(Path::new("snap.fdu")), Path::new("."));
        assert!(fs::read_dir(parent_dir(Path::new("snap.fdu"))).is_ok(), "must be openable");
        assert_eq!(parent_dir(Path::new("/cache/snap.fdu")), Path::new("/cache"));
        assert_eq!(parent_dir(Path::new("cache/snap.fdu")), Path::new("cache"));
    }

    #[test]
    fn a_temporary_name_is_unique_per_sequence_and_carries_process_entropy() {
        // The two components that make a corpse unreachable by a future process, and a
        // writer unable to collide with itself.
        let path = Path::new("/cache/snap.fdu");
        assert_ne!(temp_name(path, 0), temp_name(path, 1), "the counter separates files");
        let name = temp_name(path, 0).to_string_lossy().into_owned();
        assert!(name.starts_with(".snap.fdu.tmp."), "reaper prefix must match: {name}");
        assert!(
            name.contains(&format!("{:016x}", *TEMP_FILE_ENTROPY)),
            "entropy must be in the name, or a recycled pid regenerates a corpse: {name}"
        );
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
        index.set_initial_coverage(crate::Status::Partial(crate::CoverageReason::Inaccessible));

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
            tag_rules_fingerprint: 11,
            type_rules_fingerprint: 22,
            reducers_fingerprint: 33,
            hidden_fingerprint: 44,
            max_files: Some(55),
            exclude_special: true,
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
