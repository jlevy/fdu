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

/// A `statfs` type field as the 32-bit magic number the kernel headers define.
///
/// The field is signed and its width differs by target, so a magic with the high bit set,
/// btrfs's among them, reads as negative where it is 32 bits wide; keeping its low 32 bits
/// gives the same number on every target.
#[cfg(any(target_os = "linux", target_os = "android"))]
fn magic<T: TryInto<i64>>(value: T) -> u32 {
    let value: i64 = value.try_into().unwrap_or(0);
    u32::try_from(value & 0xFFFF_FFFF).unwrap_or(0)
}

fn c_path(path: &Path) -> io::Result<CString> {
    CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "a path holds a NUL byte"))
}

/// The filesystems Linux names by number that a report is likely to touch, by the magic
/// the kernel defines (most in `include/uapi/linux/magic.h`; configfs, fusectl, hfsplus,
/// and mqueue in their own sources; ZFS in its own), with the name `mount` gives each.
/// They include the pseudo filesystems a report of `/` crosses on an ordinary machine,
/// such as cgroup, tracefs, and securityfs. The ext family shares one number, so it is
/// named as one.
#[cfg(any(target_os = "linux", target_os = "android"))]
const LINUX_TYPES: [(u32, &str); 42] = [
    (0x0000_0187, "autofs"),
    (0x0000_1CD1, "devpts"),
    (0x0000_482B, "hfsplus"),
    (0x0000_4D44, "vfat"),
    (0x0000_6969, "nfs"),
    (0x0000_9660, "iso9660"),
    (0x0000_9FA0, "proc"),
    (0x0000_EF53, "ext2/3/4"),
    (0x0027_E0EB, "cgroup"),
    (0x00C3_6400, "ceph"),
    (0x0102_1994, "tmpfs"),
    (0x0102_1997, "9p"),
    (0x1501_3346, "udf"),
    (0x1980_0202, "mqueue"),
    (0x2011_BAB0, "exfat"),
    (0x2FC1_2FC1, "zfs"),
    (0x4249_4E4D, "binfmt_misc"),
    (0x5346_544E, "ntfs"),
    (0x5846_5342, "xfs"),
    (0x6165_676C, "pstore"),
    (0x6265_6570, "configfs"),
    (0x6265_6572, "sysfs"),
    (0x6367_7270, "cgroup2"),
    (0x6462_6720, "debugfs"),
    (0x6573_5543, "fusectl"),
    (0x6573_5546, "fuse"),
    (0x6E73_6673, "nsfs"),
    (0x7363_6673, "securityfs"),
    (0x7371_7368, "squashfs"),
    (0x7472_6163, "tracefs"),
    (0x794C_7630, "overlay"),
    (0x8584_58F6, "ramfs"),
    (0x9123_683E, "btrfs"),
    (0x9584_58F6, "hugetlbfs"),
    (0xCA45_1A4E, "bcachefs"),
    (0xCAFE_4A11, "bpf"),
    (0xDE5E_81E4, "efivarfs"),
    (0xE0F5_E1E2, "erofs"),
    (0xF2F5_2010, "f2fs"),
    (0xF97C_FF8C, "selinuxfs"),
    (0xFE53_4D42, "smb2"),
    (0xFF53_4D42, "cifs"),
];

/// Linux names a filesystem only by number: the common ones by the name `mount` gives
/// them ([`LINUX_TYPES`]), btrfs and ZFS among them, whose free space is shared across
/// volumes, which a report must say; any other as the number the kernel gave, in the
/// hexadecimal its headers use.
#[cfg(any(target_os = "linux", target_os = "android"))]
fn linux_type_name(magic: u32) -> String {
    LINUX_TYPES
        .binary_search_by_key(&magic, |(number, _)| *number)
        .map_or_else(|_| format!("{magic:#x}"), |at| LINUX_TYPES[at].1.to_owned())
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub(super) fn observe(path: &Path) -> io::Result<VolumeCapacity> {
    let path = c_path(path)?;
    // On 32-bit glibc the block counts are 32 bits wide, and a filesystem too large for
    // them fails with `EOVERFLOW`, which the report shows as capacity unknown with that
    // reason rather than as a truncated size.
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
        filesystem: Some(linux_type_name(magic(stats.f_type))),
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
    fn linux_names_the_common_filesystems_and_shows_every_other_number() {
        assert_eq!(linux_type_name(0x9123_683E), "btrfs");
        assert_eq!(linux_type_name(0x2FC1_2FC1), "zfs");
        assert_eq!(linux_type_name(0xEF53), "ext2/3/4");
        assert_eq!(linux_type_name(0x0102_1994), "tmpfs");
        assert_eq!(linux_type_name(0x794C_7630), "overlay");
        assert_eq!(linux_type_name(0x0187), "autofs");
        // The pseudo filesystems a whole-machine report crosses are named too.
        assert_eq!(linux_type_name(0x0027_E0EB), "cgroup");
        assert_eq!(linux_type_name(0x7472_6163), "tracefs");
        // One the table does not name keeps the kernel's number.
        assert_eq!(linux_type_name(0x1234_5678), "0x12345678");
    }

    /// The names are found by binary search, so the table must stay sorted by number,
    /// with each number named once.
    #[test]
    fn the_linux_type_table_is_sorted_by_number() {
        assert!(LINUX_TYPES.windows(2).all(|pair| pair[0].0 < pair[1].0), "{LINUX_TYPES:?}");
        for (number, name) in LINUX_TYPES {
            assert_eq!(linux_type_name(number), name);
        }
    }

    #[test]
    fn a_magic_reads_the_same_whatever_the_width_of_its_field() {
        // btrfs's magic has its high bit set: a 32-bit field holds it as negative.
        let narrow = i32::from_ne_bytes(0x9123_683E_u32.to_ne_bytes());
        assert!(narrow < 0);
        assert_eq!(magic(narrow), 0x9123_683E);
        assert_eq!(magic(0x9123_683E_i64), 0x9123_683E);
        assert_eq!(magic(0xEF53_i64), 0xEF53);
    }
}
