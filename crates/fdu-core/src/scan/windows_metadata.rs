//! Stable-Rust Windows entry identity and change-time observation.

use std::fs::{File, Metadata, OpenOptions};
use std::io;
use std::mem::{MaybeUninit, size_of};
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::os::windows::io::AsRawHandle;
use std::path::Path;

use windows_sys::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_SHARING_VIOLATION, HANDLE};
use windows_sys::Win32::Storage::FileSystem::{
    BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
    FILE_ATTRIBUTE_TAG_INFO, FILE_BASIC_INFO, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_FLAG_OPEN_REPARSE_POINT, FILE_ID_INFO, FILE_SHARE_DELETE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, FileAttributeTagInfo, FileBasicInfo, FileIdInfo, GetFileInformationByHandle,
    GetFileInformationByHandleEx,
};

use crate::{Attrs, EntryKind};

const WINDOWS_TO_UNIX_EPOCH_100NS: i128 = 116_444_736_000_000_000;
const REPARSE_TAG_NAME_SURROGATE: u32 = 0x2000_0000;

/// Observe an entry through a non-following handle on `path`.
///
/// Two open failures are answered from `listed` rather than with an error. A file locked
/// against even attribute reads (`C:\hiberfil.sys`, `pagefile.sys`) and one the caller may
/// not open at all (`System Volume Information`) are exactly the entries std's `metadata`
/// serves from directory enumeration instead, and they are among the largest on a system
/// volume, so a disk-usage tool that drops them reports the wrong total. `listed` supplies
/// what enumeration already knows — kind, size, and write time — and change time,
/// identity, and volume stay zero for that entry, which the `Attrs` contract defines as
/// unavailable. Any other failure is the entry's error, as before.
pub(super) fn observe(
    path: &Path,
    listed: impl FnOnce() -> io::Result<Metadata>,
) -> io::Result<(EntryKind, Attrs)> {
    let file = match open_for_attributes(path) {
        Ok(file) => file,
        Err(error) if is_locked_or_denied(&error) => {
            return match listed() {
                Ok(meta) => Ok(observe_listed(&meta)),
                // The open error names the real obstacle; the listing rarely fails at all.
                Err(_) => Err(error),
            };
        }
        Err(error) => return Err(error),
    };
    let observed = query(&file)?;
    Ok((observed.kind(), observed.attrs()))
}

pub(super) fn attrs_from_file(file: &File) -> io::Result<Attrs> {
    Ok(query(file)?.attrs())
}

/// Open `path` for attribute queries and nothing else.
///
/// Zero desired access is what std's `metadata` asks for. `CreateFileW` documents that an
/// application may then "query certain metadata such as file, directory, or device
/// attributes without accessing that file or device, even if `GENERIC_READ` access would
/// have been denied", which covers `GetFileInformationByHandle` and the `FileBasicInfo`,
/// `FileAttributeTagInfo`, and `FileIdInfo` classes. Sharing delete as well as reads and
/// writes prevents observation from introducing a Windows-only rename or replacement
/// lock; `OPEN_REPARSE_POINT` preserves the non-following scan contract, and
/// `BACKUP_SEMANTICS` is what lets a directory be opened at all.
fn open_for_attributes(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .access_mode(0)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)
}

/// Whether an open failed for one of the two reasons std's `metadata` falls back on.
///
/// `ERROR_SHARING_VIOLATION` is a file locked in a way that denies even attribute reads,
/// which std's own comment names `C:\hiberfil.sys` for; `ERROR_ACCESS_DENIED` is one the
/// caller may not open at all, such as `System Volume Information`. Neither says the
/// entry is absent or unreadable by enumeration.
fn is_locked_or_denied(error: &io::Error) -> bool {
    error
        .raw_os_error()
        .and_then(|code| u32::try_from(code).ok())
        .is_some_and(|code| code == ERROR_SHARING_VIOLATION || code == ERROR_ACCESS_DENIED)
}

/// What directory enumeration knows about an entry: its kind, size, and write time.
///
/// This is the observation for an entry whose handle cannot be opened. Change time,
/// identity, and volume are not in enumeration data and stay zero, so the fingerprint
/// degrades to size and write time for that entry alone. std's `is_symlink` applies the
/// same name-surrogate rule as [`kind_from_attributes`], so the two sources agree on kind.
fn observe_listed(meta: &Metadata) -> (EntryKind, Attrs) {
    let file_type = meta.file_type();
    let kind = if file_type.is_symlink() {
        EntryKind::Symlink
    } else if file_type.is_dir() {
        EntryKind::Dir
    } else {
        EntryKind::File
    };
    let size = meta.file_size();
    let write_time = i64::try_from(meta.last_write_time()).unwrap_or(i64::MAX);
    let attrs = Attrs {
        size,
        allocated: size,
        mtime_ns: windows_time_to_unix_ns(write_time),
        ctime_ns: 0,
        inode: 0,
        dev: 0,
    };
    (kind, attrs)
}

struct Observed {
    basic: FILE_BASIC_INFO,
    tag: FILE_ATTRIBUTE_TAG_INFO,
    identity: BY_HANDLE_FILE_INFORMATION,
    /// The entry's identity on its volume; see [`file_id`].
    file_id: u64,
}

impl Observed {
    fn kind(&self) -> EntryKind {
        kind_from_attributes(self.tag.FileAttributes, self.tag.ReparseTag)
    }

    fn attrs(&self) -> Attrs {
        let size =
            (u64::from(self.identity.nFileSizeHigh) << 32) | u64::from(self.identity.nFileSizeLow);
        Attrs {
            size,
            allocated: size,
            mtime_ns: windows_time_to_unix_ns(self.basic.LastWriteTime),
            ctime_ns: windows_time_to_unix_ns(self.basic.ChangeTime),
            inode: self.file_id,
            dev: u64::from(self.identity.dwVolumeSerialNumber),
        }
    }
}

fn kind_from_attributes(attributes: u32, reparse_tag: u32) -> EntryKind {
    if attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        && reparse_tag & REPARSE_TAG_NAME_SURROGATE != 0
    {
        EntryKind::Symlink
    } else if attributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
        EntryKind::Dir
    } else {
        EntryKind::File
    }
}

/// Reads of one handle allowed for two consecutive ones to agree, counting the first.
///
/// The three queries in [`query_once`] are not atomic, so one pair of reads can straddle
/// a write and disagree. An entry that is merely being written (a log, or a directory
/// whose children are changing) settles within a read or two; only an entry that keeps
/// changing across every pair is reported as changed rather than recorded torn.
const MAX_OBSERVATION_READS: usize = 4;

fn query(file: &File) -> io::Result<Observed> {
    let handle = file.as_raw_handle() as HANDLE;
    let mut previous = query_once(handle)?;
    for _ in 1..MAX_OBSERVATION_READS {
        let next = query_once(handle)?;
        if same_observation(&previous, &next) {
            return Ok(next);
        }
        previous = next;
    }
    Err(io::Error::other("file changed while Windows metadata was observed"))
}

/// The volume serial number of the volume `path` is on.
///
/// This bounds a one-filesystem walk, and a volume does not change while an entry is
/// written, so it is read once rather than through [`query`]: a root directory whose
/// children are being created still has a volume, where demanding a consistent
/// observation of its times would fail the whole walk. A root that is locked or denied
/// reports zero, the same unavailable device [`observe`] records for such an entry.
pub(super) fn volume_serial(path: &Path) -> io::Result<u64> {
    let file = match open_for_attributes(path) {
        Ok(file) => file,
        Err(error) if is_locked_or_denied(&error) => return Ok(0),
        Err(error) => return Err(error),
    };
    let handle = file.as_raw_handle() as HANDLE;
    let mut identity = MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
    // SAFETY: the API receives a live handle borrowed from `file` and correctly sized,
    // aligned writable storage for its documented output structure, and retains no
    // pointer. A zero return leaves the storage uninitialized and is handled before it is
    // read.
    let identity = unsafe {
        if GetFileInformationByHandle(handle, identity.as_mut_ptr()) == 0 {
            return Err(io::Error::last_os_error());
        }
        identity.assume_init()
    };
    Ok(u64::from(identity.dwVolumeSerialNumber))
}

fn query_once(handle: HANDLE) -> io::Result<Observed> {
    let mut basic = MaybeUninit::<FILE_BASIC_INFO>::uninit();
    let mut tag = MaybeUninit::<FILE_ATTRIBUTE_TAG_INFO>::uninit();
    let mut identity = MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
    // SAFETY: both APIs receive a live handle borrowed from `file`, correctly sized,
    // aligned writable storage for their documented output structure, and no pointer is
    // retained. A zero return leaves the storage uninitialized and is handled before it
    // is read.
    unsafe {
        if GetFileInformationByHandleEx(
            handle,
            FileBasicInfo,
            basic.as_mut_ptr().cast(),
            u32::try_from(size_of::<FILE_BASIC_INFO>()).expect("FILE_BASIC_INFO fits u32"),
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
        if GetFileInformationByHandleEx(
            handle,
            FileAttributeTagInfo,
            tag.as_mut_ptr().cast(),
            u32::try_from(size_of::<FILE_ATTRIBUTE_TAG_INFO>())
                .expect("FILE_ATTRIBUTE_TAG_INFO fits u32"),
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
        if GetFileInformationByHandle(handle, identity.as_mut_ptr()) == 0 {
            return Err(io::Error::last_os_error());
        }
        let identity = identity.assume_init();
        Ok(Observed {
            basic: basic.assume_init(),
            tag: tag.assume_init(),
            file_id: file_id(handle, &identity),
            identity,
        })
    }
}

/// The entry's identity on its volume.
///
/// `BY_HANDLE_FILE_INFORMATION` carries a 64-bit index that NTFS keeps unique and `ReFS` —
/// Windows 11 Dev Drive — does not: its identifiers are 128 bits, and the documentation
/// says the 64-bit one "is not guaranteed to be unique on `ReFS`". `FileIdInfo` returns the
/// full identifier, folded by [`fold_file_id`] so that an identifier that is the 64-bit
/// index zero-extended, as on NTFS, keeps that index. A volume that does not answer
/// `FileIdInfo` keeps the 64-bit index too, so no volume loses the identity it had.
fn file_id(handle: HANDLE, identity: &BY_HANDLE_FILE_INFORMATION) -> u64 {
    let index = (u64::from(identity.nFileIndexHigh) << 32) | u64::from(identity.nFileIndexLow);
    let mut info = MaybeUninit::<FILE_ID_INFO>::uninit();
    // SAFETY: as in `query_once` — a live handle, correctly sized and aligned writable
    // storage for the documented output structure, no pointer retained, and a zero
    // return handled before the storage is read.
    unsafe {
        if GetFileInformationByHandleEx(
            handle,
            FileIdInfo,
            info.as_mut_ptr().cast(),
            u32::try_from(size_of::<FILE_ID_INFO>()).expect("FILE_ID_INFO fits u32"),
        ) == 0
        {
            return index;
        }
        fold_file_id(info.assume_init().FileId.Identifier)
    }
}

/// Fold a 128-bit file identifier into the 64-bit `Attrs::inode`.
///
/// The low half is the identifier as NTFS reports it, where the high half is zero, so
/// such an identifier folds to itself and agrees with the 64-bit index. A nonzero high
/// half is mixed in through an odd multiplier, which maps no nonzero value to zero and
/// is not symmetric in the halves, so identifiers that share either half — or exchange
/// them — stay distinct.
fn fold_file_id(identifier: [u8; 16]) -> u64 {
    let (low, high) = identifier.split_at(8);
    let low = u64::from_le_bytes(low.try_into().expect("eight bytes"));
    let high = u64::from_le_bytes(high.try_into().expect("eight bytes"));
    if high == 0 {
        return low;
    }
    low ^ high.rotate_left(32).wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

fn same_observation(left: &Observed, right: &Observed) -> bool {
    left.basic.LastWriteTime == right.basic.LastWriteTime
        && left.basic.ChangeTime == right.basic.ChangeTime
        && left.basic.FileAttributes == right.basic.FileAttributes
        && left.tag.FileAttributes == right.tag.FileAttributes
        && left.tag.ReparseTag == right.tag.ReparseTag
        && left.identity.dwVolumeSerialNumber == right.identity.dwVolumeSerialNumber
        && left.identity.nFileSizeHigh == right.identity.nFileSizeHigh
        && left.identity.nFileSizeLow == right.identity.nFileSizeLow
        && left.identity.nFileIndexHigh == right.identity.nFileIndexHigh
        && left.identity.nFileIndexLow == right.identity.nFileIndexLow
        && left.file_id == right.file_id
}

/// A `FILETIME` as nanoseconds since the Unix epoch.
///
/// Zero ticks is how Windows reports a time it does not have: FAT and exFAT keep no change
/// time, fastfat returns the field zeroed, and `BY_HANDLE_FILE_INFORMATION` documents zero
/// as "not supported". That yields zero, which the `Attrs` contract defines as
/// unavailable. Any other value outside the nanosecond range saturates, as the Unix
/// `compose_ns` does: a timestamp is a fact about the entry, never a reason to drop the
/// entry from the totals.
fn windows_time_to_unix_ns(ticks: i64) -> i64 {
    if ticks == 0 {
        return 0;
    }
    let nanos = (i128::from(ticks) - WINDOWS_TO_UNIX_EPOCH_100NS) * 100;
    match i64::try_from(nanos) {
        Ok(nanos) => nanos,
        Err(_) if nanos < 0 => i64::MIN,
        Err(_) => i64::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_epoch_conversion_is_signed_and_saturating() {
        assert_eq!(windows_time_to_unix_ns(116_444_736_000_000_000), 0);
        assert_eq!(windows_time_to_unix_ns(116_444_735_999_999_999), -100);
        // Zero ticks is a time the filesystem does not keep — FAT and exFAT have no change
        // time — and is unavailable, never an error that drops the entry.
        assert_eq!(windows_time_to_unix_ns(0), 0);
        // Anything else outside the nanosecond range saturates, as the Unix path does.
        assert_eq!(windows_time_to_unix_ns(1), i64::MIN);
        assert_eq!(windows_time_to_unix_ns(i64::MIN), i64::MIN);
        assert_eq!(windows_time_to_unix_ns(i64::MAX), i64::MAX);
    }

    #[test]
    fn only_locked_and_denied_opens_fall_back_to_listing_data() {
        let os_error = |code: u32| io::Error::from_raw_os_error(i32::try_from(code).expect("code"));
        assert!(is_locked_or_denied(&os_error(ERROR_ACCESS_DENIED)));
        assert!(is_locked_or_denied(&os_error(ERROR_SHARING_VIOLATION)));
        // ERROR_FILE_NOT_FOUND and ERROR_PATH_NOT_FOUND: the entry is gone, which
        // `missing_as_none` answers, not the listing.
        assert!(!is_locked_or_denied(&os_error(2)));
        assert!(!is_locked_or_denied(&os_error(3)));
        assert!(!is_locked_or_denied(&io::Error::other("not an OS error")));
    }

    #[test]
    fn a_root_changing_underneath_still_reports_its_volume() {
        use std::sync::atomic::{AtomicBool, Ordering};
        struct StopOnDrop<'a>(&'a AtomicBool);
        impl Drop for StopOnDrop<'_> {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Relaxed);
            }
        }
        let root = tempfile::tempdir().expect("tempdir");
        let (_, observed) = observe(root.path(), || panic!("a directory opens")).expect("observe");
        let stop = AtomicBool::new(false);
        std::thread::scope(|scope| {
            // A failed assertion must stop the writer before the scope joins it.
            let _stop_on_unwind = StopOnDrop(&stop);
            // Creating and removing children keeps changing the root's write and change
            // times, which is what made a consistent observation of the root fail.
            scope.spawn(|| {
                let mut round = 0u32;
                while !stop.load(Ordering::Relaxed) {
                    let child = root.path().join(format!("c{}", round % 8));
                    let _ = std::fs::create_dir(&child);
                    let _ = std::fs::remove_dir(&child);
                    round = round.wrapping_add(1);
                }
            });
            for _ in 0..2_000 {
                let serial = volume_serial(root.path());
                assert_eq!(serial.ok(), Some(observed.dev), "the volume is read, not observed");
            }
        });
    }

    #[test]
    fn listing_data_observes_kind_size_and_write_time_with_identity_unavailable() {
        let root = tempfile::tempdir().expect("tempdir");
        let file = root.path().join("listed.txt");
        std::fs::write(&file, b"12345").expect("write");
        let dir = root.path().join("sub");
        std::fs::create_dir(&dir).expect("mkdir");

        let (kind, listed) = observe_listed(&std::fs::symlink_metadata(&file).expect("metadata"));
        let (_, opened) = observe(&file, || panic!("an ordinary file opens")).expect("observe");
        assert_eq!(kind, EntryKind::File);
        assert_eq!(listed.size, 5);
        assert_eq!(listed.mtime_ns, opened.mtime_ns, "one write time from either source");
        assert_eq!((listed.ctime_ns, listed.inode, listed.dev), (0, 0, 0));
        assert_ne!((opened.inode, opened.dev), (0, 0), "the handle identifies the entry");

        let (kind, _) = observe_listed(&std::fs::symlink_metadata(&dir).expect("metadata"));
        assert_eq!(kind, EntryKind::Dir);
    }

    #[test]
    fn file_identity_keeps_a_zero_extended_index_and_separates_128_bit_ids() {
        fn identifier(low: u64, high: u64) -> [u8; 16] {
            let mut bytes = [0; 16];
            bytes[..8].copy_from_slice(&low.to_le_bytes());
            bytes[8..].copy_from_slice(&high.to_le_bytes());
            bytes
        }
        let index = 0x0005_0000_0000_1234;
        assert_eq!(fold_file_id(identifier(index, 0)), index, "NTFS-shaped ids keep the index");
        let a = fold_file_id(identifier(7, 5));
        assert_ne!(a, fold_file_id(identifier(7, 6)), "ids sharing the low half differ");
        assert_ne!(a, fold_file_id(identifier(8, 5)), "ids sharing the high half differ");
        assert_ne!(a, fold_file_id(identifier(5, 7)), "ids with the halves exchanged differ");
        assert_ne!(a, 7, "a 128-bit id is not its truncation");
    }

    #[test]
    fn only_name_surrogate_reparse_tags_are_link_like() {
        const IO_REPARSE_TAG_SYMLINK: u32 = 0xa000_000c;
        const IO_REPARSE_TAG_WOF: u32 = 0x8000_0017;
        assert_eq!(
            kind_from_attributes(FILE_ATTRIBUTE_REPARSE_POINT, IO_REPARSE_TAG_SYMLINK),
            EntryKind::Symlink
        );
        assert_eq!(
            kind_from_attributes(FILE_ATTRIBUTE_REPARSE_POINT, IO_REPARSE_TAG_WOF),
            EntryKind::File
        );
        assert_eq!(
            kind_from_attributes(
                FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DIRECTORY,
                IO_REPARSE_TAG_WOF,
            ),
            EntryKind::Dir
        );
    }
}
