//! Stable-Rust Windows entry identity and change-time observation.

use std::fs::{File, OpenOptions};
use std::io;
use std::mem::{MaybeUninit, size_of};
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use std::path::Path;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Storage::FileSystem::{
    BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
    FILE_ATTRIBUTE_TAG_INFO, FILE_BASIC_INFO, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, FileAttributeTagInfo, FileBasicInfo, GetFileInformationByHandle,
    GetFileInformationByHandleEx,
};

use crate::{Attrs, EntryKind};

const WINDOWS_TO_UNIX_EPOCH_100NS: i128 = 116_444_736_000_000_000;
const REPARSE_TAG_NAME_SURROGATE: u32 = 0x2000_0000;

pub(super) fn observe(path: &Path) -> io::Result<(EntryKind, Attrs)> {
    // FILE_BASIC_INFO and FILE_ATTRIBUTE_TAG_INFO require FILE_READ_ATTRIBUTES. Sharing
    // delete as well as reads and writes prevents observation from introducing a
    // Windows-only rename or replacement lock; OPEN_REPARSE_POINT preserves the
    // non-following scan contract.
    let file = OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)?;
    let observed = query(&file)?;
    Ok((observed.kind(), observed.attrs()?))
}

pub(super) fn attrs_from_file(file: &File) -> io::Result<Attrs> {
    query(file)?.attrs()
}

struct Observed {
    basic: FILE_BASIC_INFO,
    tag: FILE_ATTRIBUTE_TAG_INFO,
    identity: BY_HANDLE_FILE_INFORMATION,
}

impl Observed {
    fn kind(&self) -> EntryKind {
        kind_from_attributes(self.tag.FileAttributes, self.tag.ReparseTag)
    }

    fn attrs(&self) -> io::Result<Attrs> {
        let size =
            (u64::from(self.identity.nFileSizeHigh) << 32) | u64::from(self.identity.nFileSizeLow);
        Ok(Attrs {
            size,
            allocated: size,
            mtime_ns: windows_time_to_unix_ns(self.basic.LastWriteTime)?,
            ctime_ns: windows_time_to_unix_ns(self.basic.ChangeTime)?,
            inode: (u64::from(self.identity.nFileIndexHigh) << 32)
                | u64::from(self.identity.nFileIndexLow),
            dev: u64::from(self.identity.dwVolumeSerialNumber),
        })
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

fn query(file: &File) -> io::Result<Observed> {
    let handle = file.as_raw_handle() as HANDLE;
    let first = query_once(handle)?;
    let second = query_once(handle)?;
    if !same_observation(&first, &second) {
        return Err(io::Error::other("file changed while Windows metadata was observed"));
    }
    Ok(second)
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
        Ok(Observed {
            basic: basic.assume_init(),
            tag: tag.assume_init(),
            identity: identity.assume_init(),
        })
    }
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
}

fn windows_time_to_unix_ns(ticks: i64) -> io::Result<i64> {
    let nanos = (i128::from(ticks) - WINDOWS_TO_UNIX_EPOCH_100NS)
        .checked_mul(100)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Windows timestamp overflow"))?;
    i64::try_from(nanos)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Windows timestamp out of range"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_epoch_conversion_is_checked_and_signed() {
        assert_eq!(windows_time_to_unix_ns(116_444_736_000_000_000).expect("Unix epoch"), 0);
        assert_eq!(
            windows_time_to_unix_ns(116_444_735_999_999_999).expect("before Unix epoch"),
            -100
        );
        assert_eq!(
            windows_time_to_unix_ns(i64::MAX).expect_err("out of range").kind(),
            io::ErrorKind::InvalidData
        );
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
