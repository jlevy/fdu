//! Filesystem capacity on Unix: one `statfs` call, or `statvfs` where `statfs` has no
//! portable layout.

use std::ffi::CString;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use super::VolumeCapacity;

/// A `statfs` field widened to `u64`; field widths differ by target (`__fsword_t` is 64
/// bits on `x86_64` and 32 on 32-bit ARM), and a negative value means none.
#[cfg(not(any(target_os = "macos", target_os = "ios")))]
fn width<T: TryInto<u64>>(value: T) -> u64 {
    value.try_into().unwrap_or(0)
}

/// A `statfs` type field widened to `i64`, as [`width`] widens a count.
#[cfg(any(target_os = "linux", target_os = "android"))]
fn signed<T: TryInto<i64>>(value: T) -> i64 {
    value.try_into().unwrap_or(-1)
}

fn c_path(path: &Path) -> io::Result<CString> {
    CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "a path holds a NUL byte"))
}

/// Linux names a filesystem only by number. The two named here are the ones whose free
/// space is shared across volumes, which a report must say; any other is shown as the
/// number the kernel gave, in the hexadecimal its headers use.
#[cfg(any(target_os = "linux", target_os = "android"))]
fn linux_type_name(magic: i64) -> String {
    const BTRFS: i64 = 0x9123_683E;
    const ZFS: i64 = 0x2FC1_2FC1;
    match magic {
        BTRFS => "btrfs".to_owned(),
        ZFS => "zfs".to_owned(),
        other => format!("{other:#x}"),
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub(super) fn observe(path: &Path) -> io::Result<VolumeCapacity> {
    let path = c_path(path)?;
    let mut stats = std::mem::MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: `path` is NUL-terminated and outlives the call, and `stats` is a valid
    // out-pointer the call fully writes on success; it is read only after success.
    let stats = unsafe {
        if libc::statfs(path.as_ptr(), stats.as_mut_ptr()) != 0 {
            return Err(io::Error::last_os_error());
        }
        stats.assume_init()
    };
    // `f_blocks` counts fragments where the filesystem reports a fragment size, and
    // blocks of `f_bsize` otherwise, as glibc's `statvfs` reads them.
    let unit = match width(stats.f_frsize) {
        0 => width(stats.f_bsize),
        fragment => fragment,
    };
    Ok(VolumeCapacity {
        capacity_bytes: width(stats.f_blocks).saturating_mul(unit),
        available_bytes: width(stats.f_bavail).saturating_mul(unit),
        filesystem: Some(linux_type_name(signed(stats.f_type))),
    })
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub(super) fn observe(path: &Path) -> io::Result<VolumeCapacity> {
    let path = c_path(path)?;
    let mut stats = std::mem::MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: as above; `f_fstypename` is a NUL-terminated array the kernel fills.
    let stats = unsafe {
        if libc::statfs(path.as_ptr(), stats.as_mut_ptr()) != 0 {
            return Err(io::Error::last_os_error());
        }
        stats.assume_init()
    };
    let unit = u64::from(stats.f_bsize);
    let name: Vec<u8> = stats
        .f_fstypename
        .iter()
        .take_while(|byte| **byte != 0)
        .map(|byte| byte.to_ne_bytes()[0])
        .collect();
    Ok(VolumeCapacity {
        capacity_bytes: stats.f_blocks.saturating_mul(unit),
        available_bytes: stats.f_bavail.saturating_mul(unit),
        filesystem: (!name.is_empty()).then(|| String::from_utf8_lossy(&name).into_owned()),
    })
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios"
)))]
pub(super) fn observe(path: &Path) -> io::Result<VolumeCapacity> {
    let path = c_path(path)?;
    let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // SAFETY: as above.
    let stats = unsafe {
        if libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) != 0 {
            return Err(io::Error::last_os_error());
        }
        stats.assume_init()
    };
    let unit = width(stats.f_frsize);
    Ok(VolumeCapacity {
        capacity_bytes: width(stats.f_blocks).saturating_mul(unit),
        available_bytes: width(stats.f_bavail).saturating_mul(unit),
        filesystem: None,
    })
}

#[cfg(all(test, any(target_os = "linux", target_os = "android")))]
mod tests {
    use super::*;

    #[test]
    fn linux_names_the_pooled_filesystems_and_shows_every_other_number() {
        assert_eq!(linux_type_name(0x9123_683E), "btrfs");
        assert_eq!(linux_type_name(0x2FC1_2FC1), "zfs");
        assert_eq!(linux_type_name(0xEF53), "0xef53");
    }
}
