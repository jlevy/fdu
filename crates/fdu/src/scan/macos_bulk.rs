//! A macOS directory reader that asks the kernel for names and stat-tier metadata in
//! one bulk call.
//!
//! `getattrlistbulk` has no safe standard-library wrapper. This module is therefore the
//! sole FFI boundary: it validates every offset and returned-attribute bitmap before
//! constructing an entry. Any unsupported attribute, malformed record, or syscall
//! failure returns `None`, which makes the caller reopen the directory through the
//! portable `read_dir` reference path.

use std::ffi::OsString;
use std::fs::File;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStringExt;
use std::path::Path;

use super::{Attrs, EntryKind, compose_ns};

const BUFFER_BYTES: usize = 64 * 1024;

// libc exposes the bulk call and the other attr.h constants, but not this one.
const ATTR_CMN_ERROR: libc::attrgroup_t = 0x2000_0000;

// sys/stat.h marks a firmlink with this file flag. Like a mount point,
// getattrlistbulk reports its underlying vnode rather than the object stat sees.
const SF_FIRMLINK: u32 = 0x0080_0000;

// Values of Darwin's `enum vtype` from sys/vnode.h. libc does not currently expose
// them, and requesting POSIX mode would add another field solely to recover this enum.
const VREG: u32 = 1;
const VDIR: u32 = 2;
const VLNK: u32 = 5;

const REQUIRED_COMMON: libc::attrgroup_t = libc::ATTR_CMN_NAME
    | libc::ATTR_CMN_DEVID
    | libc::ATTR_CMN_OBJTYPE
    | libc::ATTR_CMN_MODTIME
    | libc::ATTR_CMN_CHGTIME
    | libc::ATTR_CMN_FLAGS
    | libc::ATTR_CMN_FILEID;
const REQUESTED_COMMON: libc::attrgroup_t =
    libc::ATTR_CMN_RETURNED_ATTRS | REQUIRED_COMMON | ATTR_CMN_ERROR;
const REQUESTED_DIRECTORY: libc::attrgroup_t =
    libc::ATTR_DIR_MOUNTSTATUS | libc::ATTR_DIR_ALLOCSIZE | libc::ATTR_DIR_DATALENGTH;
const REQUESTED_FILE: libc::attrgroup_t = libc::ATTR_FILE_ALLOCSIZE | libc::ATTR_FILE_DATALENGTH;

pub(super) struct Entry {
    pub(super) name: OsString,
    pub(super) kind: EntryKind,
    pub(super) attrs: Attrs,
}

pub(super) struct Reader {
    buffer: Box<[u8]>,
}

impl Reader {
    pub(super) fn new() -> Self {
        Self { buffer: vec![0; BUFFER_BYTES].into_boxed_slice() }
    }

    /// Read one complete directory, or decline so the portable backend can retry it.
    pub(super) fn read(&mut self, path: &Path) -> Option<Vec<Entry>> {
        let directory = File::open(path).ok()?;
        let mut request = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: REQUESTED_COMMON,
            volattr: 0,
            dirattr: REQUESTED_DIRECTORY,
            fileattr: REQUESTED_FILE,
            forkattr: 0,
        };
        let mut entries = Vec::new();

        loop {
            // SAFETY: `directory` remains open for the call; `request` is a fully
            // initialized attrlist; and `buffer` is a live, writable allocation whose
            // pointer and exact length are passed together. The return value is checked
            // before any bytes are interpreted, and `parse_entry` bounds-checks every
            // kernel-provided length and offset.
            let count = unsafe {
                libc::getattrlistbulk(
                    directory.as_raw_fd(),
                    (&raw mut request).cast(),
                    self.buffer.as_mut_ptr().cast(),
                    self.buffer.len(),
                    0,
                )
            };
            if count < 0 {
                return None;
            }
            if count == 0 {
                return Some(entries);
            }

            let count = usize::try_from(count).ok()?;
            let mut offset = 0usize;
            for _ in 0..count {
                let length = usize::try_from(read_u32(self.buffer.get(offset..)?)?).ok()?;
                let end = offset.checked_add(length)?;
                if length < size_of::<u32>() || end > self.buffer.len() {
                    return None;
                }
                let entry = parse_entry(&self.buffer[offset..end])?;
                if entry.name != "." && entry.name != ".." {
                    entries.push(entry);
                }
                offset = end;
            }
        }
    }
}

fn parse_entry(record: &[u8]) -> Option<Entry> {
    let mut cursor = Cursor::new(record);
    let declared = usize::try_from(cursor.u32()?).ok()?;
    if declared != record.len() {
        return None;
    }

    // ATTR_CMN_RETURNED_ATTRS is special: it is always first, regardless of its high
    // bit. ATTR_CMN_ERROR, when present, immediately follows it. Remaining fields use
    // the documented common/directory/file attribute order.
    let returned = AttributeSet {
        common: cursor.u32()?,
        volume: cursor.u32()?,
        directory: cursor.u32()?,
        file: cursor.u32()?,
        fork: cursor.u32()?,
    };
    if returned.common & (REQUIRED_COMMON | libc::ATTR_CMN_RETURNED_ATTRS)
        != REQUIRED_COMMON | libc::ATTR_CMN_RETURNED_ATTRS
        || returned.common & !REQUESTED_COMMON != 0
        || returned.volume != 0
        || returned.directory & !REQUESTED_DIRECTORY != 0
        || returned.file & !REQUESTED_FILE != 0
        || returned.fork != 0
    {
        return None;
    }
    if returned.common & ATTR_CMN_ERROR != 0 && cursor.u32()? != 0 {
        return None;
    }

    let reference_position = cursor.position();
    let name_offset = cursor.i32()?;
    let name_length = usize::try_from(cursor.u32()?).ok()?;
    let dev = cursor.i32()?;
    let object_type = cursor.u32()?;
    let mtime = cursor.timespec()?;
    let ctime = cursor.timespec()?;
    let flags = cursor.u32()?;
    let inode = cursor.u64()?;

    let mount_status = if returned.directory & libc::ATTR_DIR_MOUNTSTATUS == 0 {
        None
    } else {
        Some(cursor.u32()?)
    };
    let directory_allocated =
        if returned.directory & libc::ATTR_DIR_ALLOCSIZE == 0 { None } else { Some(cursor.i64()?) };
    let directory_size = if returned.directory & libc::ATTR_DIR_DATALENGTH == 0 {
        None
    } else {
        Some(cursor.i64()?)
    };
    let file_allocated =
        if returned.file & libc::ATTR_FILE_ALLOCSIZE == 0 { None } else { Some(cursor.i64()?) };
    let file_size =
        if returned.file & libc::ATTR_FILE_DATALENGTH == 0 { None } else { Some(cursor.i64()?) };
    let fixed_end = cursor.position();

    let kind = match object_type {
        VREG => EntryKind::File,
        VDIR => EntryKind::Dir,
        VLNK => EntryKind::Symlink,
        _ => EntryKind::Other,
    };
    // getattrlistbulk deliberately reports a mount point's underlying vnode, whereas
    // stat reports the mounted root. That difference would break one-filesystem scans
    // and filesystem identity, so make the entire containing directory take the
    // portable path whenever the kernel marks a mount or trigger boundary.
    if flags & SF_FIRMLINK != 0 || kind.is_dir() && mount_status? != 0 {
        return None;
    }
    let (size, allocated) = if kind.is_dir() {
        (directory_size?, directory_allocated?)
    } else {
        (file_size?, file_allocated?)
    };
    let size = u64::try_from(size).ok()?;
    let allocated = u64::try_from(allocated).ok()?;

    let name_offset = usize::try_from(name_offset).ok()?;
    let name_start = reference_position.checked_add(name_offset)?;
    let name_end = name_start.checked_add(name_length)?;
    // Variable-length attributes follow the fixed-width fields. A reference back into
    // those fields can still be in bounds and can even contain a plausible NUL-terminated
    // component, but it is not a valid kernel record and must take the portable path.
    if name_start < fixed_end {
        return None;
    }
    let name = record.get(name_start..name_end)?;
    let (&0, name) = name.split_last()? else { return None };
    if name.is_empty() || name.contains(&0) || name.contains(&b'/') {
        return None;
    }

    Some(Entry {
        name: OsString::from_vec(name.to_vec()),
        kind,
        attrs: Attrs {
            size,
            allocated,
            mtime_ns: compose_ns(mtime.0, mtime.1),
            ctime_ns: compose_ns(ctime.0, ctime.1),
            inode,
            // Match std::os::unix::fs::MetadataExt::dev, including sign extension
            // from Darwin's signed 32-bit dev_t.
            dev: u64::from_ne_bytes(i64::from(dev).to_ne_bytes()),
        },
    })
}

fn read_u32(bytes: &[u8]) -> Option<u32> {
    Some(u32::from_ne_bytes(bytes.get(..size_of::<u32>())?.try_into().ok()?))
}

struct AttributeSet {
    common: libc::attrgroup_t,
    volume: libc::attrgroup_t,
    directory: libc::attrgroup_t,
    file: libc::attrgroup_t,
    fork: libc::attrgroup_t,
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    const fn position(&self) -> usize {
        self.position
    }

    fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
        let end = self.position.checked_add(N)?;
        let value = self.bytes.get(self.position..end)?.try_into().ok()?;
        self.position = end;
        Some(value)
    }

    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_ne_bytes(self.take()?))
    }

    fn i32(&mut self) -> Option<i32> {
        Some(i32::from_ne_bytes(self.take()?))
    }

    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_ne_bytes(self.take()?))
    }

    fn i64(&mut self) -> Option<i64> {
        Some(i64::from_ne_bytes(self.take()?))
    }

    fn timespec(&mut self) -> Option<(i64, i64)> {
        Some((self.i64()?, self.i64()?))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::os::unix::fs::symlink;

    use super::*;
    use crate::scan::{attrs_from, kind_from, metadata_for_fingerprint};

    fn valid_file_record() -> Vec<u8> {
        let mut record = Vec::new();
        record.extend_from_slice(&0_u32.to_ne_bytes()); // Filled after the name.
        record.extend_from_slice(&REQUESTED_COMMON.to_ne_bytes());
        record.extend_from_slice(&0_u32.to_ne_bytes()); // volume attributes
        record.extend_from_slice(&0_u32.to_ne_bytes()); // directory attributes
        record.extend_from_slice(&REQUESTED_FILE.to_ne_bytes());
        record.extend_from_slice(&0_u32.to_ne_bytes()); // fork attributes
        record.extend_from_slice(&0_u32.to_ne_bytes()); // per-entry error
        let reference_position = record.len();
        record.extend_from_slice(&0_i32.to_ne_bytes()); // Filled after fixed fields.
        record.extend_from_slice(&5_u32.to_ne_bytes());
        record.extend_from_slice(&7_i32.to_ne_bytes()); // dev
        record.extend_from_slice(&VREG.to_ne_bytes());
        record.extend_from_slice(&2_i64.to_ne_bytes()); // mtime seconds
        record.extend_from_slice(&3_i64.to_ne_bytes()); // mtime nanoseconds
        record.extend_from_slice(&4_i64.to_ne_bytes()); // ctime seconds
        record.extend_from_slice(&5_i64.to_ne_bytes()); // ctime nanoseconds
        record.extend_from_slice(&0_u32.to_ne_bytes()); // file flags
        record.extend_from_slice(&6_u64.to_ne_bytes()); // inode
        record.extend_from_slice(&8_i64.to_ne_bytes()); // allocated size
        record.extend_from_slice(&7_i64.to_ne_bytes()); // data-fork size
        let name_position = record.len();
        record.extend_from_slice(b"file\0");

        let length = u32::try_from(record.len()).expect("fixture record fits u32");
        record[..4].copy_from_slice(&length.to_ne_bytes());
        let name_offset = i32::try_from(name_position - reference_position)
            .expect("fixture name offset fits i32");
        record[reference_position..reference_position + 4]
            .copy_from_slice(&name_offset.to_ne_bytes());
        record
    }

    #[test]
    fn bulk_entries_match_the_portable_metadata_contract_byte_for_byte() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let file = directory.path().join("file");
        fs::write(&file, b"contents").expect("regular file");
        fs::write(file.join("..namedfork/rsrc"), b"resource fork").expect("resource fork");
        fs::create_dir(directory.path().join("dir")).expect("child directory");
        symlink("file", directory.path().join("link")).expect("symbolic link");
        fs::write(directory.path().join("decomposed-e\u{301}"), b"bytes")
            .expect("decomposed Unicode name");

        let expected: BTreeMap<_, _> = fs::read_dir(directory.path())
            .expect("portable directory listing")
            .map(|item| {
                let item = item.expect("portable entry");
                let metadata = metadata_for_fingerprint(&item).expect("portable metadata");
                (item.file_name(), (kind_from(&metadata), attrs_from(&metadata)))
            })
            .collect();

        let actual: BTreeMap<_, _> = Reader::new()
            .read(directory.path())
            .expect("APFS supports the requested bulk attributes")
            .into_iter()
            .map(|entry| (entry.name, (entry.kind, entry.attrs)))
            .collect();

        assert_eq!(actual, expected);
    }

    #[test]
    fn parser_accepts_only_complete_in_bounds_records() {
        let valid = valid_file_record();
        assert_eq!(
            parse_entry(&valid).map(|entry| (entry.name, entry.kind, entry.attrs)),
            Some((
                OsString::from("file"),
                EntryKind::File,
                Attrs {
                    size: 7,
                    allocated: 8,
                    mtime_ns: 2_000_000_003,
                    ctime_ns: 4_000_000_005,
                    inode: 6,
                    dev: 7,
                },
            ))
        );

        for length in 0..valid.len() {
            let mut truncated = valid[..length].to_vec();
            if truncated.len() >= size_of::<u32>() {
                let declared = u32::try_from(length).expect("fixture length fits u32");
                truncated[..4].copy_from_slice(&declared.to_ne_bytes());
            }
            assert!(parse_entry(&truncated).is_none(), "accepted truncation at {length}");
        }

        let mut escaped_name = valid.clone();
        let reference_position = 4 + size_of::<libc::attribute_set_t>() + size_of::<u32>();
        escaped_name[reference_position..reference_position + 4]
            .copy_from_slice(&i32::MAX.to_ne_bytes());
        assert!(parse_entry(&escaped_name).is_none());

        let mut backward_name = valid.clone();
        let device_position = reference_position + 2 * size_of::<u32>();
        backward_name[device_position..device_position + 4].copy_from_slice(b"a\0\0\0");
        let backward_offset =
            i32::try_from(device_position - reference_position).expect("fixture offset fits i32");
        backward_name[reference_position..reference_position + 4]
            .copy_from_slice(&backward_offset.to_ne_bytes());
        backward_name[reference_position + 4..reference_position + 8]
            .copy_from_slice(&2_u32.to_ne_bytes());
        assert!(parse_entry(&backward_name).is_none());

        let mut missing_fingerprint = valid.clone();
        let common = REQUESTED_COMMON & !libc::ATTR_CMN_CHGTIME;
        missing_fingerprint[4..8].copy_from_slice(&common.to_ne_bytes());
        assert!(parse_entry(&missing_fingerprint).is_none());

        let mut unexpected_layout = valid;
        let common = REQUESTED_COMMON | libc::ATTR_CMN_ACCTIME;
        unexpected_layout[4..8].copy_from_slice(&common.to_ne_bytes());
        assert!(parse_entry(&unexpected_layout).is_none());

        let mut firmlink = valid_file_record();
        let flags_position = 4
            + size_of::<libc::attribute_set_t>()
            + size_of::<u32>()
            + 2 * size_of::<u32>()
            + size_of::<i32>()
            + size_of::<u32>()
            + 2 * 2 * size_of::<i64>();
        firmlink[flags_position..flags_position + 4].copy_from_slice(&SF_FIRMLINK.to_ne_bytes());
        assert!(parse_entry(&firmlink).is_none());

        let mut signed_device = valid_file_record();
        let device_position =
            4 + size_of::<libc::attribute_set_t>() + size_of::<u32>() + 2 * size_of::<u32>();
        signed_device[device_position..device_position + 4]
            .copy_from_slice(&(-1_i32).to_ne_bytes());
        assert_eq!(parse_entry(&signed_device).map(|entry| entry.attrs.dev), Some(u64::MAX));
    }
}
