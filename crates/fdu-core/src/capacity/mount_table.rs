//! What the operating system has mounted where, read from its mount table: the gate every
//! capacity call below a report's root passes before it is made (review H1 on #209).
//!
//! A capacity call resolves its path, and resolving a path through an automount trigger
//! that is not mounted asks the automounter to mount it and waits for the answer. The stat
//! that never mounts does not make a path safe to touch: `AT_NO_AUTOMOUNT` covers only a
//! path's last component, and only where that component exists. A key of an indirect map
//! that has expired, whose directory the automounter removed, is looked up by name, which
//! asks the automounter; and every earlier component is resolved as any lookup is, through
//! whatever trigger lies on the way. Reading the mount table resolves no path at all.
//!
//! So a directory below a root is touched only where the table lists a mount at exactly its
//! path, visible there, whose topmost filesystem is not an automount trigger: then the path
//! resolves through mounted filesystems alone, and a stat of it, and the capacity call that
//! follows, mount nothing.
//!
//! - **Linux:** `/proc/self/mountinfo`, read whole. A mount point is unescaped (the kernel
//!   writes a space, tab, newline, or backslash as three octal digits), a stack of mounts at
//!   one path is topped by the one no other mount there names as its parent, and a mount is
//!   visible only where each mount it hangs from is the topmost at its own mount point. A
//!   mount an ancestor's later mount covers is listed but cannot be reached.
//! - **macOS:** `getfsstat` with `MNT_NOWAIT`, which answers from what the kernel holds and
//!   never asks a filesystem, so a server that does not answer cannot hold it. The list
//!   gives no parents, so a later mount at a path tops an earlier one, and each is taken to
//!   be visible.
//! - **Windows:** no automount, so there is no table to read: every path passes, and the
//!   stat that confirms the device decides, as it did before the table was read.
//! - Anywhere else, and wherever the table cannot be read, nothing below a root is asked.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// What the mount table says is mounted at exactly one path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MountAt {
    /// A filesystem other than an automount trigger is mounted here and visible, so the
    /// path resolves through mounted filesystems alone.
    Mounted,
    /// The topmost filesystem here is an automount trigger, so what it mounts is not
    /// mounted now, and touching the path would ask for it.
    Automount,
    /// Nothing visible is mounted at exactly this path.
    NotMounted,
}

/// What the mount table says is mounted where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum MountTable {
    /// Every visible mount point, and whether its topmost filesystem is an automount
    /// trigger.
    #[cfg_attr(windows, allow(dead_code, reason = "Windows reads no table"))]
    Read(BTreeMap<PathBuf, bool>),
    /// A platform with no automount, which keeps no table to read: every path passes.
    #[cfg_attr(not(windows), allow(dead_code, reason = "only Windows has no automount"))]
    NoAutomount,
    /// The table could not be read, or this platform's is not: nothing passes.
    #[cfg_attr(windows, allow(dead_code, reason = "Windows reads no table"))]
    Unreadable,
}

/// The type an automount trigger's filesystem has on Linux and macOS alike: autofs, which
/// systemd's automount units also use.
#[cfg(unix)]
const AUTOMOUNT_TYPE: &[u8] = b"autofs";

impl MountTable {
    /// Read the table this platform keeps. No path is resolved.
    pub(crate) fn read() -> Self {
        #[cfg(target_os = "linux")]
        {
            std::fs::read("/proc/self/mountinfo")
                .map_or(Self::Unreadable, |text| Self::from_mountinfo(&text))
        }
        #[cfg(target_os = "macos")]
        {
            macos::mounts().map_or(Self::Unreadable, Self::from_stacked)
        }
        #[cfg(windows)]
        {
            Self::NoAutomount
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
        {
            Self::Unreadable
        }
    }

    /// What is mounted at exactly `path`, or `None` where the table could not be read.
    pub(crate) fn at(&self, path: &Path) -> Option<MountAt> {
        match self {
            Self::Read(mounts) => Some(match mounts.get(path) {
                Some(true) => MountAt::Automount,
                Some(false) => MountAt::Mounted,
                None => MountAt::NotMounted,
            }),
            Self::NoAutomount => Some(MountAt::Mounted),
            Self::Unreadable => None,
        }
    }

    /// The table a Linux `mountinfo` text describes: each line's mount ID, parent ID, and
    /// mount point (its first, second, and fifth fields) and the filesystem type after the
    /// lone `-` that ends its optional fields. A line that does not parse is left out.
    #[cfg(any(target_os = "linux", all(unix, test)))]
    pub(crate) fn from_mountinfo(text: &[u8]) -> Self {
        use std::os::unix::ffi::OsStrExt as _;
        let mut mounts: Vec<Mount> = Vec::new();
        for line in text.split(|byte| *byte == b'\n') {
            let fields: Vec<&[u8]> = line.split(|byte| *byte == b' ').collect();
            let (Some(id), Some(parent), Some(point)) = (
                fields.first().and_then(|field| number(field)),
                fields.get(1).and_then(|field| number(field)),
                fields.get(4),
            ) else {
                continue;
            };
            let Some(separator) =
                fields.iter().skip(6).position(|field| *field == b"-").map(|at| at + 6)
            else {
                continue;
            };
            let Some(kind) = fields.get(separator + 1) else { continue };
            let point = PathBuf::from(std::ffi::OsStr::from_bytes(&unescape(point)));
            mounts.push(Mount {
                id,
                parent: Some(parent),
                point,
                automount: *kind == AUTOMOUNT_TYPE,
            });
        }
        Self::from_mounts(&mounts)
    }

    /// The table a list of mounts in mount order describes, which names no parents: a later
    /// mount at a path tops an earlier one, and every mount is taken to be visible.
    #[cfg(any(target_os = "macos", test))]
    fn from_stacked(mounts: Vec<(PathBuf, bool)>) -> Self {
        Self::Read(mounts.into_iter().collect())
    }

    /// The table `mounts` describes, each naming the mount it hangs from.
    #[cfg(any(target_os = "linux", all(unix, test)))]
    fn from_mounts(mounts: &[Mount]) -> Self {
        let mut at: BTreeMap<&Path, Vec<usize>> = BTreeMap::new();
        for (position, mount) in mounts.iter().enumerate() {
            at.entry(mount.point.as_path()).or_default().push(position);
        }
        let by_id: BTreeMap<u64, usize> =
            mounts.iter().enumerate().map(|(position, mount)| (mount.id, position)).collect();
        // The topmost of a stack is the mount no other at its path hangs from; the base is
        // the one that hangs from a mount elsewhere. Each falls back to table order, later
        // on top, should a table name neither.
        let top = |stack: &[usize]| {
            stack
                .iter()
                .rev()
                .find(|&&position| {
                    !stack.iter().any(|&other| mounts[other].parent == Some(mounts[position].id))
                })
                .or_else(|| stack.last())
                .copied()
        };
        let base = |stack: &[usize]| {
            stack
                .iter()
                .find(|&&position| {
                    !stack.iter().any(|&other| Some(mounts[other].id) == mounts[position].parent)
                })
                .or_else(|| stack.first())
                .copied()
        };
        // Reachable: the mount a stack hangs from is the topmost at its own path, no mount
        // lies between the two paths to cross into instead, and the same holds above.
        let visible = |point: &Path| {
            let mut point = point;
            // A chain no longer than the table, so a table whose parents loop ends.
            for _ in 0..=mounts.len() {
                let Some(base) = at.get(point).and_then(|stack| base(stack)) else { return false };
                // A mount whose parent is not listed, or is itself, is the root of what this
                // process sees.
                let Some(&parent) = mounts[base].parent.and_then(|parent| by_id.get(&parent))
                else {
                    return true;
                };
                if parent == base {
                    return true;
                }
                let above = mounts[parent].point.as_path();
                if above == point
                    || !point.starts_with(above)
                    || at.get(above).and_then(|stack| top(stack)) != Some(parent)
                    || point
                        .ancestors()
                        .skip(1)
                        .take_while(|between| *between != above)
                        .any(|between| at.contains_key(between))
                {
                    return false;
                }
                point = above;
            }
            false
        };
        Self::Read(
            at.iter()
                .filter(|(point, _)| visible(point))
                .filter_map(|(point, stack)| {
                    top(stack).map(|topmost| (point.to_path_buf(), mounts[topmost].automount))
                })
                .collect(),
        )
    }
}

/// One mount in a table.
#[cfg(any(target_os = "linux", all(unix, test)))]
struct Mount {
    id: u64,
    /// The mount this one hangs from, where the table names one.
    parent: Option<u64>,
    point: PathBuf,
    automount: bool,
}

/// A decimal field.
#[cfg(any(target_os = "linux", all(unix, test)))]
fn number(field: &[u8]) -> Option<u64> {
    std::str::from_utf8(field).ok()?.parse().ok()
}

/// A `mountinfo` path with each `\` and three octal digits replaced by the byte they name,
/// as the kernel escapes a space, tab, newline, and backslash (`seq_path` in `fs/seq_file.c`).
#[cfg(any(target_os = "linux", all(unix, test)))]
fn unescape(field: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(field.len());
    let mut at = 0;
    while at < field.len() {
        let digits = field.get(at + 1..at + 4).filter(|digits| {
            field[at] == b'\\' && digits.iter().all(|digit| (b'0'..=b'7').contains(digit))
        });
        let escaped = digits.and_then(|digits| {
            u8::try_from(
                digits.iter().fold(0_u32, |value, digit| value * 8 + u32::from(digit - b'0')),
            )
            .ok()
        });
        if let Some(byte) = escaped {
            bytes.push(byte);
            at += 4;
        } else {
            bytes.push(field[at]);
            at += 1;
        }
    }
    bytes
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod macos {
    use std::ffi::CStr;
    use std::os::unix::ffi::OsStrExt as _;
    use std::path::PathBuf;

    /// No host mounts more than this; a larger count from the kernel is refused rather
    /// than sized from.
    const MAX_MOUNTS: usize = 1 << 16;

    /// Every mount, in the kernel's order, as its mount point and whether it is autofs.
    pub(super) fn mounts() -> std::io::Result<Vec<(PathBuf, bool)>> {
        // Room for a few mounts made between the count and the read; a table that filled it
        // is read again.
        for _ in 0..4 {
            // SAFETY: a null buffer asks only for the count, and writes nothing.
            let count = unsafe { libc::getfsstat(std::ptr::null_mut(), 0, libc::MNT_NOWAIT) };
            let count = usize::try_from(count).map_err(|_| std::io::Error::last_os_error())?;
            if count > MAX_MOUNTS {
                return Err(std::io::Error::other("the mount table is larger than any host's"));
            }
            let room = count + 8;
            // SAFETY: `statfs` is integers and byte arrays, for which all-zero bytes are a
            // valid value.
            let mut table: Vec<libc::statfs> =
                (0..room).map(|_| unsafe { std::mem::zeroed() }).collect();
            let bytes = libc::c_int::try_from(room * std::mem::size_of::<libc::statfs>())
                .map_err(|_| std::io::Error::other("the mount table is larger than any host's"))?;
            // SAFETY: `table` holds `room` writable `statfs` values, `bytes` in all, and the
            // call writes at most `bytes` of them.
            let filled = unsafe { libc::getfsstat(table.as_mut_ptr(), bytes, libc::MNT_NOWAIT) };
            let filled = usize::try_from(filled).map_err(|_| std::io::Error::last_os_error())?;
            if filled >= room {
                continue;
            }
            table.truncate(filled);
            return Ok(table
                .iter()
                .map(|mount| {
                    (
                        PathBuf::from(std::ffi::OsStr::from_bytes(name(&mount.f_mntonname))),
                        name(&mount.f_fstypename) == super::AUTOMOUNT_TYPE,
                    )
                })
                .collect());
        }
        Err(std::io::Error::other("the mount table kept growing while it was read"))
    }

    /// The NUL-terminated name in a fixed array, or all of it where no NUL ends it.
    fn name(field: &[libc::c_char]) -> &[u8] {
        // SAFETY: `c_char` and `u8` have one size and alignment, and the slice is borrowed
        // for no longer than `field`.
        let bytes = unsafe { std::slice::from_raw_parts(field.as_ptr().cast::<u8>(), field.len()) };
        CStr::from_bytes_until_nul(bytes).map_or(bytes, CStr::to_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(components: &[&str]) -> PathBuf {
        let mut path = PathBuf::from(std::path::MAIN_SEPARATOR_STR);
        path.extend(components);
        path
    }

    /// A Linux table: the root, a tmpfs at a path holding a space, an indirect map with one
    /// key mounted and a bind mounted below that, a direct trigger with its filesystem
    /// mounted over it, a tmpfs with a direct trigger mounted over it, and a tmpfs covered by
    /// a later mount at its parent's path.
    #[cfg(unix)]
    #[test]
    fn the_mountinfo_table_names_each_visible_mount_point_and_its_topmost_filesystem() {
        let text = b"\
22 1 0:21 / / rw,relatime shared:1 - ext4 /dev/vda rw\n\
30 22 0:30 / /srv/with\\040space rw - tmpfs tmpfs rw\n\
31 22 0:31 / /home rw,relatime shared:2 - autofs auto.home rw,fd=5,indirect\n\
32 31 0:32 / /home/alice rw - nfs server:/alice rw\n\
33 32 0:33 / /home/alice/inner rw - tmpfs tmpfs rw\n\
40 22 0:40 / /net/direct rw - autofs auto.direct rw,direct\n\
41 40 0:41 / /net/direct rw - nfs server:/direct rw\n\
50 22 0:50 / /data/trig rw - tmpfs tmpfs rw\n\
51 50 0:51 / /data/trig rw - autofs auto.trig rw,direct\n\
60 22 0:60 / /cover/hidden rw - tmpfs tmpfs rw\n\
61 22 0:61 / /cover rw - autofs auto.cover rw,indirect\n\
a malformed line\n\
70 22 0:70 / /missing/separator rw tmpfs\n";
        let table = MountTable::from_mountinfo(text);
        let at = |components: &[&str]| table.at(&path(components));
        assert_eq!(at(&[]), Some(MountAt::Mounted), "the root");
        assert_eq!(at(&["srv", "with space"]), Some(MountAt::Mounted), "unescaped");
        assert_eq!(at(&["srv", "with\\040space"]), Some(MountAt::NotMounted), "never raw");
        assert_eq!(at(&["home"]), Some(MountAt::Automount), "the indirect map itself");
        assert_eq!(at(&["home", "alice"]), Some(MountAt::Mounted), "a key, mounted");
        assert_eq!(at(&["home", "alice", "inner"]), Some(MountAt::Mounted), "nested below it");
        assert_eq!(at(&["home", "bob"]), Some(MountAt::NotMounted), "an expired key");
        assert_eq!(at(&["home", "alice", "sub"]), Some(MountAt::NotMounted), "not a mount point");
        assert_eq!(at(&["net", "direct"]), Some(MountAt::Mounted), "the trigger's mount on top");
        assert_eq!(at(&["data", "trig"]), Some(MountAt::Automount), "a trigger on top");
        assert_eq!(at(&["cover", "hidden"]), Some(MountAt::NotMounted), "covered from above");
        assert_eq!(at(&["missing", "separator"]), Some(MountAt::NotMounted), "unparsed");
        assert_eq!(unescape(b"a\\134b\\011c\\0"), b"a\\b\tc\\0", "a partial escape stays");
    }

    #[test]
    fn a_stacked_table_is_topped_by_its_later_mount() {
        let table = MountTable::from_stacked(vec![
            (path(&["Volumes", "share"]), true),
            (path(&["Volumes", "share"]), false),
            (path(&["Network", "auto"]), true),
        ]);
        assert_eq!(table.at(&path(&["Volumes", "share"])), Some(MountAt::Mounted));
        assert_eq!(table.at(&path(&["Network", "auto"])), Some(MountAt::Automount));
        assert_eq!(table.at(&path(&["Volumes"])), Some(MountAt::NotMounted));
        assert_eq!(MountTable::Unreadable.at(&path(&["Volumes"])), None);
        assert_eq!(MountTable::NoAutomount.at(&path(&["Volumes"])), Some(MountAt::Mounted));
    }

    /// The table this host keeps reads, and holds the mount the root is on.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn this_hosts_table_holds_its_root() {
        assert_eq!(MountTable::read().at(Path::new("/")), Some(MountAt::Mounted));
    }
}
