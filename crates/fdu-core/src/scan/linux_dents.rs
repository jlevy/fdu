//! A Linux directory reader that lists a directory with raw `getdents64` and stats each
//! entry with `statx` relative to the listing's own descriptor.
//!
//! It replaces what the portable path pays per directory and per entry: glibc's
//! `opendir` (its `fstat` and 32 KiB `DIR` buffer) and the standard library's `CString`,
//! `OsString`, and `Arc` for every listed name. The glibc wrappers for both calls
//! (`statx` in 2.28, `getdents64` in 2.30) are newer than the manylinux2014 wheel's
//! glibc 2.17, so this module calls them through `libc::syscall`, as the standard
//! library's own fallback does, and is therefore the sole FFI boundary. Records are
//! parsed by offset in safe code, and every kernel-provided length is validated before
//! it is used. Any open, enumeration, or malformed-record failure, and a kernel or
//! sandbox without `statx`, returns `None`, which makes the caller reopen the complete
//! directory through the portable `read_dir` reference path.
//!
//! Per-entry stats pass `AT_NO_AUTOMOUNT` (fdu-puk7): an unmounted automount point among
//! the children is reported as the trigger directory rather than mounted by the stat.

use std::ffi::OsStr;
use std::fs;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use super::{Attrs, EntryKind, compose_ns};

/// Bytes asked of each `getdents64` call. Two literals, not a cast: pedantic clippy.
const CHUNK_BYTES: usize = 65_536;
const CHUNK_COUNT: libc::c_uint = 65_536;
/// Record-buffer capacity kept between listings; a larger buffer shrinks back to a chunk.
const RETAINED_BYTES: usize = 1_048_576;
/// Entry capacity kept between listings, bounded for the same reason as the records.
const RETAINED_ENTRIES: usize = 16_384;
/// `linux_dirent64` header: `d_ino` u64 at 0, `d_off` i64 at 8, `d_reclen` u16 at 16,
/// `d_type` u8 at 18, then `d_name` at 19, NUL-terminated. The kernel pads `d_reclen` to
/// 8 bytes; nothing here relies on that.
const RECORD_HEADER: usize = 19;
const RECLEN_OFFSET: usize = 16;
const TYPE_OFFSET: usize = 18;
const STATX_FLAGS: libc::c_int =
    libc::AT_SYMLINK_NOFOLLOW | libc::AT_NO_AUTOMOUNT | libc::AT_STATX_SYNC_AS_STAT;
const STATX_MASK: libc::c_uint = libc::STATX_BASIC_STATS;
// The kernel writes sizeof(struct statx) == 256 bytes; the binding must be that size.
const _: () = assert!(size_of::<libc::statx>() == 256);

/// Set once `statx` reports `ENOSYS` or `EPERM` (a kernel before 4.11, or a seccomp
/// filter). Every later read then declines at once, as the standard library's own
/// `statx` probe does, instead of paying an open and a `getdents64` per directory
/// before declining.
static STATX_UNAVAILABLE: AtomicBool = AtomicBool::new(false);

/// Which listed children need a stat.
#[derive(Clone, Copy, Debug)]
pub(super) struct StatPolicy {
    /// H72: directory and symlink kinds come from `d_type` without a stat.
    pub(super) skip_dir_symlink_stat: bool,
    /// Descent compares `attrs.dev`, so directories are stated even under the skip.
    pub(super) one_filesystem: bool,
}

/// What one listed child's observation produced.
pub(super) enum Outcome {
    Observed {
        kind: EntryKind,
        attrs: Attrs,
    },
    /// The stat failed with something other than `ENOENT`; the caller reports it at
    /// `listing_path.join(name)`. `ENOENT` entries are not yielded at all.
    Failed(std::io::Error),
}

struct Dent {
    name_start: usize,
    name_len: usize,
    outcome: Outcome,
}

/// One listed child, its name borrowed from the reader's buffer.
pub(super) struct Entry<'a> {
    pub(super) name: &'a OsStr,
    pub(super) outcome: Outcome,
}

/// One complete listing, yielded in enumeration order, names borrowed from the reader.
pub(super) struct Listing<'a> {
    names: &'a [u8],
    entries: std::vec::Drain<'a, Dent>,
}

impl<'a> Iterator for Listing<'a> {
    type Item = Entry<'a>;

    fn next(&mut self) -> Option<Entry<'a>> {
        let dent = self.entries.next()?;
        let bytes = &self.names[dent.name_start..dent.name_start + dent.name_len];
        Some(Entry { name: OsStr::from_bytes(bytes), outcome: dent.outcome })
    }
}

#[derive(Default)]
struct Counts {
    entries: u64,
    stats: u64,
    enumeration_calls: u64,
}

/// A walker's reusable native directory reader.
pub(super) struct Reader {
    /// The raw `getdents64` records of the current listing; names are ranges in it.
    names: Vec<u8>,
    entries: Vec<Dent>,
    /// Test seam: runs before each entry's stat with the entry's name, so a test can
    /// delete or replace the entry between `getdents64` and `statx`.
    #[cfg(test)]
    pub(super) before_stat: Option<BeforeStat>,
}

#[cfg(test)]
pub(super) type BeforeStat = Box<dyn FnMut(&OsStr) + Send>;

impl Reader {
    pub(super) fn new() -> Self {
        Self {
            names: Vec::with_capacity(CHUNK_BYTES),
            entries: Vec::new(),
            #[cfg(test)]
            before_stat: None,
        }
    }

    /// Read one complete directory, or decline so the portable backend can retry it.
    ///
    /// Counting happens here, and only on success, for the reason
    /// `macos_bulk::Reader::read` gives: a declined directory is counted by the portable
    /// retry, and counting it here as well would double it.
    pub(super) fn read(&mut self, path: &Path, policy: StatPolicy) -> Option<Listing<'_>> {
        // A test hook injects into per-entry lookups and listings, which only the portable
        // path makes.
        #[cfg(test)]
        if super::walk_hook_covers(path) {
            return None;
        }
        if STATX_UNAVAILABLE.load(Ordering::Relaxed) {
            return None;
        }
        // Clear first: `shrink_to` cannot go below the previous listing's length.
        self.names.clear();
        if self.names.capacity() > RETAINED_BYTES {
            self.names.shrink_to(CHUNK_BYTES);
        }
        self.entries.clear();
        if self.entries.capacity() > RETAINED_ENTRIES {
            self.entries.shrink_to(RETAINED_ENTRIES);
        }
        let Some(counts) = self.read_native(path, policy) else {
            // Leave nothing half-parsed behind for the next listing.
            self.names.clear();
            self.entries.clear();
            return None;
        };
        crate::counters::bump(|c| {
            c.dir_opens += 1;
            c.dir_entries += counts.entries;
            c.stats += counts.stats;
            c.dir_enumeration_calls += counts.enumeration_calls;
        });
        Some(Listing { names: &self.names, entries: self.entries.drain(..) })
    }

    fn read_native(&mut self, path: &Path, policy: StatPolicy) -> Option<Counts> {
        // Any open failure declines, `ELOOP` from `O_NOFOLLOW` included: the portable
        // path reopens the directory and answers exactly as it always has.
        let directory = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
            .open(path)
            .ok()?;
        let fd = directory.as_raw_fd();
        let mut counts = Counts::default();
        loop {
            self.names.reserve(CHUNK_BYTES);
            let start = self.names.len();
            let filled = loop {
                let spare = self.names.spare_capacity_mut();
                debug_assert!(spare.len() >= CHUNK_BYTES);
                // SAFETY: `directory` keeps `fd` open for the call. `spare` is the live,
                // writable, uninitialized tail of `names`, at least CHUNK_BYTES long after
                // `reserve`, and the kernel is asked for exactly CHUNK_COUNT == CHUNK_BYTES
                // bytes, so it cannot write past it. `fd: c_int`, the pointer, and
                // `c_uint` are the register-width argument types std's own `syscall!`
                // fallback passes to `libc::syscall`. The return value is checked before
                // any byte is read, and every kernel-provided length is bounds-checked
                // by `parse_record`.
                let returned = unsafe {
                    libc::syscall(
                        libc::SYS_getdents64,
                        fd,
                        spare.as_mut_ptr().cast::<libc::c_void>(),
                        CHUNK_COUNT,
                    )
                };
                if returned >= 0 {
                    break usize::try_from(returned).ok()?;
                }
                match std::io::Error::last_os_error().raw_os_error() {
                    Some(libc::EINTR) => {}
                    // The directory was removed while it was being read. glibc's
                    // `readdir` ends the listing here without an error, as POSIX calls
                    // that the end of the directory, so the portable answer is the
                    // entries already read, and so is this one.
                    Some(libc::ENOENT) => break 0,
                    _ => return None,
                }
            };
            counts.enumeration_calls += 1;
            if filled == 0 {
                // End of directory: the terminating call.
                return Some(counts);
            }
            if filled > CHUNK_BYTES {
                // Impossible for a conforming kernel; fail closed.
                return None;
            }
            // SAFETY: the kernel initialized exactly `filled` bytes of the spare capacity
            // just passed to it, and `filled <= CHUNK_BYTES <= spare.len()`.
            unsafe { self.names.set_len(start + filled) };
            self.parse_chunk(start, fd, policy, &mut counts)?;
        }
    }

    /// Parse and observe one `getdents64` chunk, `names[start..]`.
    ///
    /// Records never straddle calls, so each chunk is parsed on its own, and name ranges
    /// are indices into `names`, which stay valid across later `reserve` reallocations.
    fn parse_chunk(
        &mut self,
        start: usize,
        fd: RawFd,
        policy: StatPolicy,
        counts: &mut Counts,
    ) -> Option<()> {
        let mut offset = start;
        while offset < self.names.len() {
            let record = parse_record(&self.names, offset)?;
            let name_start = offset + RECORD_HEADER;
            offset += record.reclen;
            let name_with_nul = &self.names[name_start..=name_start + record.name_len];
            let name = &name_with_nul[..record.name_len];
            if name == b"." || name == b".." {
                continue;
            }
            counts.entries += 1;
            #[cfg(test)]
            if let Some(hook) = self.before_stat.as_mut() {
                hook(OsStr::from_bytes(name));
            }
            if let Some(outcome) = observe(fd, name_with_nul, record.d_type, policy, counts).ok()? {
                self.entries.push(Dent { name_start, name_len: record.name_len, outcome });
            }
        }
        Some(())
    }
}

/// One record's shape, or `None` when the bytes are not a valid `linux_dirent64`.
struct ParsedRecord {
    reclen: usize,
    d_type: u8,
    name_len: usize,
}

fn parse_record(chunk: &[u8], offset: usize) -> Option<ParsedRecord> {
    let record = chunk.get(offset..)?;
    let header = record.get(..RECORD_HEADER)?;
    let reclen =
        usize::from(u16::from_ne_bytes([header[RECLEN_OFFSET], header[RECLEN_OFFSET + 1]]));
    // Header plus at least a NUL, and inside this chunk. Alignment is not required:
    // nothing is cast, so a misaligned record is merely read byte-wise.
    if reclen <= RECORD_HEADER || reclen > record.len() {
        return None;
    }
    let name_field = &record[RECORD_HEADER..reclen];
    // No NUL inside the record: decline.
    let name_len = name_field.iter().position(|&byte| byte == 0)?;
    let name = &name_field[..name_len];
    // Never kernel output.
    if name.is_empty() || name.contains(&b'/') {
        return None;
    }
    Some(ParsedRecord { reclen, d_type: header[TYPE_OFFSET], name_len })
}

/// Whether std's `DirEntry::file_type` answers this `d_type` from the listing alone.
///
/// For any other value, `DT_UNKNOWN` above all, std stats the entry itself, and the
/// H72 skip then applies to the kind that stat found.
const fn listing_names_kind(d_type: u8) -> bool {
    matches!(
        d_type,
        libc::DT_CHR
            | libc::DT_FIFO
            | libc::DT_LNK
            | libc::DT_REG
            | libc::DT_SOCK
            | libc::DT_DIR
            | libc::DT_BLK
    )
}

/// `statx` itself is unavailable, so the whole directory is declined.
struct StatxUnavailable;

/// Observe one listed child, or `Ok(None)` when it vanished before its stat.
fn observe(
    fd: RawFd,
    name_with_nul: &[u8],
    d_type: u8,
    policy: StatPolicy,
    counts: &mut Counts,
) -> Result<Option<Outcome>, StatxUnavailable> {
    if policy.skip_dir_symlink_stat {
        match d_type {
            libc::DT_DIR if !policy.one_filesystem => {
                return Ok(Some(Outcome::Observed {
                    kind: EntryKind::Dir,
                    attrs: Attrs::default(),
                }));
            }
            libc::DT_LNK => {
                return Ok(Some(Outcome::Observed {
                    kind: EntryKind::Symlink,
                    attrs: Attrs::default(),
                }));
            }
            _ => {}
        }
    }
    counts.stats += 1;
    let (kind, attrs) = match stat_entry(fd, name_with_nul) {
        Ok(Some(observed)) => observed,
        // ENOENT: the entry vanished between the listing and the stat.
        Ok(None) => return Ok(None),
        Err(error) if matches!(error.raw_os_error(), Some(libc::ENOSYS | libc::EPERM)) => {
            STATX_UNAVAILABLE.store(true, Ordering::Relaxed);
            return Err(StatxUnavailable);
        }
        Err(error) => return Ok(Some(Outcome::Failed(error))),
    };
    // std's `file_type` stats a `DT_UNKNOWN` entry itself, and the skip then applies to
    // what it found (`listed_child_kind_and_attrs`). Reproduce that answer, default attrs
    // included; a known non-directory `d_type` that stats as a directory (a race) is
    // answered with its real attrs, as the portable path answers it.
    let attrs = if policy.skip_dir_symlink_stat
        && !listing_names_kind(d_type)
        && (kind == EntryKind::Symlink || (kind == EntryKind::Dir && !policy.one_filesystem))
    {
        Attrs::default()
    } else {
        attrs
    };
    Ok(Some(Outcome::Observed { kind, attrs }))
}

fn stat_entry(fd: RawFd, name_with_nul: &[u8]) -> std::io::Result<Option<(EntryKind, Attrs)>> {
    debug_assert_eq!(name_with_nul.last(), Some(&0));
    // SAFETY: every field of `libc::statx` is an integer or padding, so all-zero bytes
    // are a valid value (std initializes its buffer the same way).
    let mut buffer: libc::statx = unsafe { std::mem::zeroed() };
    loop {
        // SAFETY: `fd` is held open by the caller's `File` for the call. `name_with_nul`
        // is a NUL-terminated byte string inside the reader's buffer, which is neither
        // written nor reallocated during the call (`parse_chunk` holds it borrowed).
        // `buffer` is a live, writable, 256-byte `libc::statx` (size asserted at compile
        // time), the size the kernel writes. Flags and mask are the documented
        // `c_int`/`c_uint` widths; the argument types match std's `syscall!` fallback.
        let result = unsafe {
            libc::syscall(
                libc::SYS_statx,
                fd,
                name_with_nul.as_ptr().cast::<libc::c_char>(),
                STATX_FLAGS,
                STATX_MASK,
                &raw mut buffer,
            )
        };
        if result == 0 {
            break;
        }
        let error = std::io::Error::last_os_error();
        match error.raw_os_error() {
            Some(libc::EINTR) => {}
            Some(libc::ENOENT) => return Ok(None),
            _ => return Err(error),
        }
    }
    Ok(Some((kind_from_mode(buffer.stx_mode), attrs_from_statx(&buffer))))
}

fn kind_from_mode(mode: u16) -> EntryKind {
    match libc::mode_t::from(mode) & libc::S_IFMT {
        libc::S_IFLNK => EntryKind::Symlink,
        libc::S_IFDIR => EntryKind::Dir,
        libc::S_IFREG => EntryKind::File,
        _ => EntryKind::Other,
    }
}

/// The attributes `attrs_from` derives from std's `from_statx`, field for field.
fn attrs_from_statx(s: &libc::statx) -> Attrs {
    Attrs {
        size: s.stx_size,
        // st_blocks is in 512-byte units by POSIX convention (`attrs_from`).
        allocated: s.stx_blocks.saturating_mul(512),
        mtime_ns: compose_ns(s.stx_mtime.tv_sec, i64::from(s.stx_mtime.tv_nsec)),
        ctime_ns: compose_ns(s.stx_ctime.tv_sec, i64::from(s.stx_ctime.tv_nsec)),
        inode: s.stx_ino,
        // std's `from_statx` sets st_dev = makedev(stx_dev_major, stx_dev_minor), and
        // `MetadataExt::dev` is that value as u64. `dev_t` is u64 on Linux; if libc ever
        // changes it this stops compiling, which is the guard wanted.
        dev: libc::makedev(s.stx_dev_major, s.stx_dev_minor),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashSet};
    use std::ffi::OsString;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::os::unix::net::UnixListener;
    use std::path::PathBuf;

    use super::*;
    use crate::ScanConfig;
    use crate::scan::{
        attrs_from, kind_from, listed_child_kind_and_attrs, metadata_for_fingerprint,
    };

    const INDEX: StatPolicy = StatPolicy { skip_dir_symlink_stat: false, one_filesystem: false };
    const SUMMARY: StatPolicy = StatPolicy { skip_dir_symlink_stat: true, one_filesystem: false };
    /// Every policy a walk can ask for: the retained index and the transient summary,
    /// each free or bound to one filesystem.
    const POLICIES: [StatPolicy; 4] = [
        INDEX,
        StatPolicy { skip_dir_symlink_stat: false, one_filesystem: true },
        SUMMARY,
        StatPolicy { skip_dir_symlink_stat: true, one_filesystem: true },
    ];
    const SEEDS: [u64; 4] = [1, 0x5eed, 169, 20_260_929];

    type Observed = BTreeMap<OsString, (EntryKind, Attrs)>;

    fn observed(listing: Listing<'_>) -> Observed {
        listing
            .map(|entry| match entry.outcome {
                Outcome::Observed { kind, attrs } => (entry.name.to_os_string(), (kind, attrs)),
                Outcome::Failed(error) => panic!("{:?}: {error}", entry.name),
            })
            .collect()
    }

    fn native(directory: &Path, policy: StatPolicy) -> Observed {
        observed(
            Reader::new().read(directory, policy).expect("a readable directory reads natively"),
        )
    }

    fn portable(directory: &Path, policy: StatPolicy) -> Observed {
        fs::read_dir(directory)
            .expect("portable listing")
            .filter_map(|item| {
                let item = item.expect("portable entry");
                listed_child_kind_and_attrs(
                    &item,
                    policy.skip_dir_symlink_stat,
                    policy.one_filesystem,
                )
                .expect("portable observation")
                .map(|found| (item.file_name(), found))
            })
            .collect()
    }

    /// One `linux_dirent64` record as the kernel lays it out, declaring `reclen` and
    /// zero-padded to at least that length.
    fn record(name: &[u8], d_type: u8, reclen: usize) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&42_u64.to_ne_bytes()); // d_ino
        bytes.extend_from_slice(&0_i64.to_ne_bytes()); // d_off
        bytes.extend_from_slice(
            &u16::try_from(reclen).expect("fixture reclen fits u16").to_ne_bytes(),
        );
        bytes.push(d_type);
        bytes.extend_from_slice(name);
        bytes.push(0);
        let padded = bytes.len().max(reclen);
        bytes.resize(padded, 0);
        bytes
    }

    /// The length the kernel gives `name`'s record: header, name, NUL, padded to 8.
    fn kernel_reclen(name: &[u8]) -> usize {
        (RECORD_HEADER + name.len() + 1).next_multiple_of(8)
    }

    fn parse_all(chunk: &[u8]) -> Option<Vec<(Vec<u8>, u8)>> {
        let mut offset = 0;
        let mut parsed = Vec::new();
        while offset < chunk.len() {
            let record = parse_record(chunk, offset)?;
            let start = offset + RECORD_HEADER;
            parsed.push((chunk[start..start + record.name_len].to_vec(), record.d_type));
            offset += record.reclen;
        }
        Some(parsed)
    }

    #[test]
    fn parser_accepts_only_complete_in_bounds_records() {
        assert_eq!(Some(CHUNK_BYTES), usize::try_from(CHUNK_COUNT).ok());

        let expected: Vec<(Vec<u8>, u8)> = vec![
            (b"a".to_vec(), libc::DT_REG),
            (vec![b'n'; 255], libc::DT_DIR),
            ((0x80_u8..=0xff).collect(), libc::DT_LNK),
            (b"invalid-\xc3\x28-utf8".to_vec(), libc::DT_UNKNOWN),
            (b".".to_vec(), libc::DT_DIR),
            (b"..".to_vec(), libc::DT_DIR),
        ];
        let mut chunk = Vec::new();
        let mut boundaries = vec![0];
        for (name, d_type) in &expected {
            chunk.extend(record(name, *d_type, kernel_reclen(name)));
            boundaries.push(chunk.len());
        }
        assert_eq!(parse_all(&chunk).as_ref(), Some(&expected));

        for length in 0..chunk.len() {
            let parsed = parse_all(&chunk[..length]);
            match boundaries.iter().position(|&boundary| boundary == length) {
                Some(records) => assert_eq!(parsed.as_deref(), Some(&expected[..records])),
                None => assert!(parsed.is_none(), "accepted a truncation at byte {length}"),
            }
        }

        for declared in [0, RECORD_HEADER, chunk.len() + 8] {
            let mut corrupt = chunk.clone();
            corrupt[RECLEN_OFFSET..RECLEN_OFFSET + 2].copy_from_slice(
                &u16::try_from(declared).expect("fixture reclen fits u16").to_ne_bytes(),
            );
            assert!(parse_record(&corrupt, 0).is_none(), "accepted reclen {declared}");
        }

        // 19 + 7 + NUL is 27 bytes, but the record declares 24: no NUL inside it.
        assert!(parse_record(&record(b"abcdefg", libc::DT_REG, 24), 0).is_none());
        assert!(parse_record(&record(b"", libc::DT_REG, 24), 0).is_none());
        assert!(parse_record(&record(b"a/b", libc::DT_REG, 24), 0).is_none());

        // Records need not be 8-byte aligned; nothing is cast, so none is required.
        let mut unaligned = record(b"ab", libc::DT_REG, RECORD_HEADER + 3);
        unaligned.extend(record(b"cd", libc::DT_DIR, RECORD_HEADER + 3));
        assert_eq!(
            parse_all(&unaligned),
            Some(vec![(b"ab".to_vec(), libc::DT_REG), (b"cd".to_vec(), libc::DT_DIR)])
        );
    }

    #[test]
    fn a_chunk_yields_its_names_in_order_without_dot_entries() {
        // Under the summary policy directory and symlink records need no stat, so a
        // synthetic chunk exercises the name ranges without any filesystem: the
        // descriptor is never used.
        let records: [(&[u8], u8); 5] = [
            (b"first", libc::DT_DIR),
            (b".", libc::DT_DIR),
            (b"\xff\xfe-not-utf8", libc::DT_DIR),
            (b"..", libc::DT_DIR),
            (b"link", libc::DT_LNK),
        ];
        let mut reader = Reader::new();
        for (name, d_type) in records {
            reader.names.extend(record(name, d_type, kernel_reclen(name)));
        }
        let mut counts = Counts::default();
        assert!(reader.parse_chunk(0, -1, SUMMARY, &mut counts).is_some());

        let listing = Listing { names: &reader.names, entries: reader.entries.drain(..) };
        let yielded: Vec<_> = listing
            .map(|entry| match entry.outcome {
                Outcome::Observed { kind, attrs } => (entry.name.to_os_string(), kind, attrs),
                Outcome::Failed(error) => panic!("{:?}: {error}", entry.name),
            })
            .collect();
        assert_eq!(
            yielded,
            vec![
                (OsString::from("first"), EntryKind::Dir, Attrs::default()),
                (
                    OsStr::from_bytes(b"\xff\xfe-not-utf8").to_os_string(),
                    EntryKind::Dir,
                    Attrs::default()
                ),
                (OsString::from("link"), EntryKind::Symlink, Attrs::default()),
            ]
        );
        assert_eq!((counts.entries, counts.stats), (3, 0));
    }

    #[test]
    fn a_stat_error_other_than_enoent_is_reported_for_its_entry_alone() {
        // Root reads everything, so the one per-entry failure a fixture can induce anywhere
        // is a name no filesystem accepts. The kernel never lists one; the stat path is
        // what is under test.
        let directory = tempfile::tempdir().expect("temporary directory");
        fs::write(directory.path().join("present"), b"bytes").expect("regular file");
        let listed = fs::File::open(directory.path()).expect("open the directory");
        let too_long = vec![b'z'; 300];
        let names: [&[u8]; 3] = [b"present", b"missing", &too_long];
        let mut reader = Reader::new();
        for name in names {
            reader.names.extend(record(name, libc::DT_REG, kernel_reclen(name)));
        }
        let mut counts = Counts::default();
        assert!(reader.parse_chunk(0, listed.as_raw_fd(), INDEX, &mut counts).is_some());

        let mut listing = Listing { names: &reader.names, entries: reader.entries.drain(..) };
        let present = listing.next().expect("the present file");
        assert_eq!(present.name, "present");
        assert!(matches!(present.outcome, Outcome::Observed { kind: EntryKind::File, .. }));
        let failed = listing.next().expect("the over-long name");
        assert_eq!(failed.name.as_bytes(), too_long.as_slice());
        match failed.outcome {
            Outcome::Failed(error) => {
                assert_eq!(error.raw_os_error(), Some(libc::ENAMETOOLONG));
                // The same text std's error for that errno renders.
                assert_eq!(
                    error.to_string(),
                    std::io::Error::from_raw_os_error(libc::ENAMETOOLONG).to_string()
                );
            }
            Outcome::Observed { .. } => panic!("an over-long name cannot be stated"),
        }
        assert!(listing.next().is_none(), "a vanished entry is skipped, not reported");
        assert_eq!((counts.entries, counts.stats), (3, 3));
    }

    #[test]
    fn entries_match_the_portable_metadata_contract_byte_for_byte() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let root = directory.path();
        fs::write(root.join("file"), b"contents").expect("regular file");
        fs::create_dir(root.join("dir")).expect("child directory");
        symlink("file", root.join("link")).expect("symlink to the file");
        symlink("missing-target", root.join("dangling")).expect("dangling symlink");
        drop(UnixListener::bind(root.join("socket")).expect("socket, a special file"));
        fs::write(root.join(".hidden"), b"h").expect("hidden file");
        fs::write(root.join("l".repeat(255)), b"long").expect("255-byte name");
        fs::write(root.join(OsStr::from_bytes(b"not-utf8-\xff\xfe")), b"raw")
            .expect("non-UTF-8 name");
        fs::write(root.join("decomposed-e\u{301}"), b"nfd").expect("decomposed Unicode name");
        // Over ten 64 KiB chunks of records: several calls, and the buffer regrows.
        let padding = "w".repeat(195);
        for index in 0..3_000 {
            fs::write(root.join(format!("{index:05}{padding}")), b"").expect("wide entry");
        }

        let reference: Observed = fs::read_dir(root)
            .expect("portable listing")
            .map(|item| {
                let item = item.expect("portable entry");
                let metadata = metadata_for_fingerprint(&item).expect("portable metadata");
                (
                    item.file_name(),
                    (
                        kind_from(&metadata),
                        attrs_from(&item.path(), &metadata).expect("Unix metadata conversion"),
                    ),
                )
            })
            .collect();
        assert_eq!(reference.len(), 3_009);
        assert_eq!(native(root, INDEX), reference);
        for policy in POLICIES {
            assert_eq!(native(root, policy), portable(root, policy), "{policy:?}");
        }
    }

    #[test]
    fn reader_declines_what_the_portable_path_must_answer() {
        let _serial = crate::counters::test_serial();
        crate::counters::enable(true);
        let directory = tempfile::tempdir().expect("temporary directory");
        let root = directory.path();
        fs::write(root.join("file"), b"x").expect("regular file");
        fs::create_dir(root.join("dir")).expect("directory");
        symlink("dir", root.join("link")).expect("symlink to a directory");

        let mut reader = Reader::new();
        let mut assert_declines = |path: &Path, why: &str| {
            crate::counters::test_thread_reset();
            assert!(reader.read(path, INDEX).is_none(), "{why}");
            assert_eq!(
                crate::counters::test_thread_snapshot(),
                crate::counters::Counts::default(),
                "{why}: a declined directory moves no counter"
            );
        };
        assert_declines(&root.join("missing"), "ENOENT");
        assert_declines(&root.join("file"), "ENOTDIR");
        // A walk only descends into what its parent's non-following stat called a
        // directory, so a symlink here is a race. Declining lets the portable path
        // follow it exactly as it always has.
        assert_declines(&root.join("link"), "ELOOP from O_NOFOLLOW");
        assert!(fs::read_dir(root.join("link")).is_ok(), "the portable path follows it");
        if crate::test_support::require_permission_bits() {
            let sealed = root.join("sealed");
            fs::create_dir(&sealed).expect("directory to seal");
            fs::set_permissions(&sealed, fs::Permissions::from_mode(0o000)).expect("seal");
            assert_declines(&sealed, "EACCES");
            fs::set_permissions(&sealed, fs::Permissions::from_mode(0o755)).expect("unseal");
        }
        crate::counters::enable(false);
    }

    #[test]
    fn a_vanished_entry_is_skipped_and_a_replaced_one_is_stated_afresh() {
        let _serial = crate::counters::test_serial();
        crate::counters::enable(true);
        for policy in [INDEX, SUMMARY] {
            let directory = tempfile::tempdir().expect("temporary directory");
            let root = directory.path().to_path_buf();
            fs::write(root.join("gone"), b"doomed").expect("file to delete");
            fs::write(root.join("swap"), b"a file for now").expect("file to replace");
            fs::write(root.join("keep"), b"kept").expect("file to keep");

            let mut reader = Reader::new();
            let hooked = root.clone();
            reader.before_stat = Some(Box::new(move |name: &OsStr| {
                if name == "gone" {
                    fs::remove_file(hooked.join("gone")).expect("delete before the stat");
                } else if name == "swap" {
                    fs::remove_file(hooked.join("swap")).expect("remove before the stat");
                    fs::create_dir(hooked.join("swap")).expect("replace with a directory");
                }
            }));
            crate::counters::test_thread_reset();
            let found = observed(reader.read(&root, policy).expect("a readable directory"));
            let counts = crate::counters::test_thread_snapshot();

            let swapped = fs::symlink_metadata(root.join("swap")).expect("the replacement");
            let expected_swap =
                (EntryKind::Dir, attrs_from(&root.join("swap"), &swapped).expect("Unix attrs"));
            // The listing said DT_REG, so even the summary answers with the real attrs
            // of what the stat found, as the portable path does.
            assert_eq!(found.get(OsStr::new("swap")), Some(&expected_swap), "{policy:?}");
            assert!(found.contains_key(OsStr::new("keep")), "{policy:?}");
            assert_eq!(found.len(), 2, "{policy:?}: the deleted entry is not yielded");
            assert_eq!((counts.dir_entries, counts.stats), (3, 3), "{policy:?}");
        }
        crate::counters::enable(false);
    }

    #[test]
    fn a_directory_removed_while_listed_ends_like_the_portable_listing() {
        fn doomed(root: &Path, name: &str) -> PathBuf {
            let directory = root.join(name);
            fs::create_dir(&directory).expect("directory to remove");
            for child in ["a", "b", "c"] {
                fs::write(directory.join(child), b"x").expect("child");
            }
            directory
        }
        fn remove(directory: &Path) {
            for child in ["a", "b", "c"] {
                fs::remove_file(directory.join(child)).expect("empty the directory");
            }
            fs::remove_dir(directory).expect("remove the directory being listed");
        }
        let scratch = tempfile::tempdir().expect("temporary directory");

        // glibc's `readdir` ends a listing whose directory died mid-read, rather than
        // failing, so the portable walk records no error for it.
        let portable_target = doomed(scratch.path(), "portable");
        let mut listing = fs::read_dir(&portable_target).expect("portable listing");
        listing.next().expect("a first entry").expect("readable");
        remove(&portable_target);
        assert!(listing.all(|item| item.is_ok()), "the portable listing ends without error");

        // Neither may this one: it yields what it read, and those children are gone too.
        let native_target = doomed(scratch.path(), "native");
        let mut reader = Reader::new();
        let hooked = native_target.clone();
        reader.before_stat = Some(Box::new(move |_: &OsStr| {
            if hooked.exists() {
                remove(&hooked);
            }
        }));
        let listing =
            reader.read(&native_target, INDEX).expect("a dead directory ends, not declines");
        assert_eq!(listing.count(), 0, "every child vanished before its stat");
    }

    #[test]
    fn counters_move_once_per_successful_listing() {
        let _serial = crate::counters::test_serial();
        crate::counters::enable(true);
        let directory = tempfile::tempdir().expect("temporary directory");
        let small = directory.path().join("small");
        fs::create_dir(&small).expect("small directory");
        fs::write(small.join("a"), b"a").expect("file");
        fs::write(small.join("b"), b"b").expect("file");
        fs::create_dir(small.join("d")).expect("directory");
        symlink("a", small.join("l")).expect("symlink");
        let wide = directory.path().join("wide");
        fs::create_dir(&wide).expect("wide directory");
        let padding = "w".repeat(195);
        for index in 0..3_000 {
            fs::write(wide.join(format!("{index:05}{padding}")), b"").expect("wide entry");
        }

        let mut reader = Reader::new();
        let mut counted = |path: &Path, policy: StatPolicy| {
            crate::counters::test_thread_reset();
            let listed = reader.read(path, policy).expect("a readable directory").count();
            let counts = crate::counters::test_thread_snapshot();
            (
                listed,
                counts.dir_opens,
                counts.dir_entries,
                counts.stats,
                counts.dir_enumeration_calls,
            )
        };
        // One data call and the terminating empty one.
        assert_eq!(counted(&small, INDEX), (4, 1, 4, 4, 2));
        assert_eq!(counted(&small, SUMMARY), (4, 1, 4, 2, 2), "files alone are stated");
        let (listed, opens, entries, stats, calls) = counted(&wide, INDEX);
        assert_eq!((listed, opens, entries, stats), (3_000, 1, 3_000, 3_000));
        assert!(calls >= 4, "a wide directory takes several calls, not {calls}");
        crate::counters::enable(false);
    }

    /// The deterministic generator `tests/reference_model.rs` uses.
    struct Generator(u64);

    impl Generator {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            self.0
        }

        /// A value in `0..bound`, from the high bits, which an LCG mixes best.
        fn below(&mut self, bound: usize) -> usize {
            usize::try_from(self.next() >> 33).expect("31 bits fit usize") % bound
        }
    }

    const NAME_ASCII: &[u8] =
        b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 ._-~";
    const NAME_MULTIBYTE: [&str; 4] = ["\u{e9}", "\u{df}", "\u{6f22}", "\u{1f600}"];
    const MAX_DEPTH: usize = 3;
    const MAX_SUBDIRECTORIES: usize = 3;
    const MAX_FILE_BYTES: usize = 10 * 1024;

    /// A name of 1–255 bytes mixing ASCII, raw high bytes, and multibyte UTF-8.
    fn random_name(generator: &mut Generator) -> Vec<u8> {
        let length = 1 + generator.below(255);
        let mut name = Vec::with_capacity(length);
        while name.len() < length {
            match generator.below(4) {
                0 | 1 => name.push(NAME_ASCII[generator.below(NAME_ASCII.len())]),
                2 => name.push(0x80 | u8::try_from(generator.below(0x80)).expect("seven bits")),
                _ => {
                    let piece = NAME_MULTIBYTE[generator.below(NAME_MULTIBYTE.len())].as_bytes();
                    if name.len() + piece.len() <= length {
                        name.extend_from_slice(piece);
                    } else {
                        name.push(b'x');
                    }
                }
            }
        }
        name
    }

    /// Build a random tree under `root` from `seed`, returning every directory in it.
    ///
    /// Depth at most three, 0–300 entries per directory of which at most three are
    /// directories, and files of 0–10 KiB beside symlinks to siblings and dangling ones.
    fn random_tree(root: &Path, seed: u64) -> Vec<PathBuf> {
        let mut generator = Generator(seed);
        let contents = vec![b'z'; MAX_FILE_BYTES];
        let mut directories = vec![root.to_path_buf()];
        let mut pending = vec![(root.to_path_buf(), 0)];
        while let Some((directory, depth)) = pending.pop() {
            let mut used = HashSet::from([b".".to_vec(), b"..".to_vec()]);
            let mut files: Vec<Vec<u8>> = Vec::new();
            let mut subdirectories = 0;
            for _ in 0..generator.below(301) {
                let name = random_name(&mut generator);
                if !used.insert(name.clone()) {
                    continue;
                }
                let path = directory.join(OsStr::from_bytes(&name));
                match generator.below(8) {
                    0 if depth < MAX_DEPTH && subdirectories < MAX_SUBDIRECTORIES => {
                        fs::create_dir(&path).expect("random directory");
                        subdirectories += 1;
                        directories.push(path.clone());
                        pending.push((path, depth + 1));
                    }
                    1 if !files.is_empty() => {
                        let target = &files[generator.below(files.len())];
                        symlink(OsStr::from_bytes(target), &path).expect("symlink to a sibling");
                    }
                    2 => symlink("missing/target", &path).expect("dangling symlink"),
                    _ => {
                        let size = generator.below(MAX_FILE_BYTES + 1);
                        fs::write(&path, &contents[..size]).expect("random file");
                        files.push(name);
                    }
                }
            }
        }
        directories
    }

    #[test]
    fn random_trees_read_the_same_natively_and_portably() {
        for seed in SEEDS {
            let directory = tempfile::tempdir().expect("temporary directory");
            for listed in random_tree(directory.path(), seed) {
                for policy in POLICIES {
                    assert_eq!(
                        native(&listed, policy),
                        portable(&listed, policy),
                        "seed {seed:#x}, {policy:?}, {listed:?}"
                    );
                }
            }
        }
    }

    /// The walk-level differential for the transient summary tier, whose fold is private
    /// to the crate. With `.gitignore` reading off, one worker is the serial portable
    /// walk; more take this reader. Every count must emit the same observations.
    #[test]
    fn summary_folds_agree_across_worker_counts_on_random_trees() {
        for seed in SEEDS {
            let directory = tempfile::tempdir().expect("temporary directory");
            random_tree(directory.path(), seed);
            let mut reference = None;
            for threads in [1, 2, 4, 8] {
                let config = ScanConfig {
                    threads: Some(threads),
                    read_controls: false,
                    ..ScanConfig::default()
                };
                let mut ops = Vec::new();
                let report = crate::scan::scan_summary_fold(directory.path(), &config, &mut |op| {
                    ops.push(format!("{:?}", op.op));
                })
                .expect("summary fold");
                assert!(report.is_complete(), "seed {seed:#x}, {threads}: {:?}", report.errors);
                ops.sort_unstable();
                let tallies = (
                    report.entries,
                    report.files_walked,
                    report.bytes_walked,
                    report.allocated_walked,
                    report.dirs_read,
                );
                let Some((expected_ops, expected_tallies)) = &reference else {
                    reference = Some((ops, tallies));
                    continue;
                };
                assert_eq!(&tallies, expected_tallies, "seed {seed:#x}, {threads} workers");
                if let Some((left, right)) =
                    expected_ops.iter().zip(&ops).find(|(left, right)| left != right)
                {
                    panic!("seed {seed:#x}, {threads} workers:\n serial: {left}\n native: {right}");
                }
                assert_eq!(ops.len(), expected_ops.len(), "seed {seed:#x}, {threads} workers");
            }
        }
    }
}
