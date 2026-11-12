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
#[cfg(any(target_os = "linux", all(unix, test)))]
use std::collections::BTreeSet;
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
        Self::from_mounts_counting(mounts).0
    }

    /// [`Self::from_mounts`], and how many steps it took: each mount of a stack visited to
    /// find its top or base, each comparison of two mount points, and each point decided.
    ///
    /// Each count is bounded by a small multiple of the mounts, so a table costs about what
    /// sorting its mount points costs, which grows with the table's bytes, however long its
    /// paths. Walking each mount's whole parent chain afresh, as this did before, made a
    /// chain of mounts nested in one another cost the cube of its length: any user can make
    /// one with FUSE, up to fusermount's `mount_max` (1,000 by default), and every report on
    /// the host that named `volumes` then spent seconds of CPU on it (review L2 on #209).
    #[cfg(any(target_os = "linux", all(unix, test)))]
    fn from_mounts_counting(mounts: &[Mount]) -> (Self, usize) {
        let mut steps = 0_usize;
        // Every mount point in path order, with the mounts at it in table order. Path order
        // puts every point below one right after it, and each point after every point above.
        let mut at: BTreeMap<&Path, Vec<usize>> = BTreeMap::new();
        for (position, mount) in mounts.iter().enumerate() {
            at.entry(mount.point.as_path()).or_default().push(position);
        }
        let points: Vec<(&Path, Vec<usize>)> = at.into_iter().collect();
        let mut point_of = vec![0; mounts.len()];
        for (point, (_, stack)) in points.iter().enumerate() {
            for &position in stack {
                point_of[position] = point;
            }
        }
        let by_id: BTreeMap<u64, usize> =
            mounts.iter().enumerate().map(|(position, mount)| (mount.id, position)).collect();
        // Each point with the ID of every mount there, and with every ID a mount there hangs
        // from.
        let ids: BTreeSet<(usize, u64)> = mounts
            .iter()
            .enumerate()
            .map(|(position, mount)| (point_of[position], mount.id))
            .collect();
        let hung_from: BTreeSet<(usize, u64)> = mounts
            .iter()
            .enumerate()
            .filter_map(|(position, mount)| mount.parent.map(|parent| (point_of[position], parent)))
            .collect();
        // The topmost of a stack is the mount no other at its path hangs from; the base is
        // the one that hangs from a mount elsewhere. Each falls back to table order, later
        // on top, should a table name neither.
        let mut top: Vec<Option<usize>> = Vec::with_capacity(points.len());
        let mut base: Vec<Option<usize>> = Vec::with_capacity(points.len());
        for (point, (_, stack)) in points.iter().enumerate() {
            top.push(
                stack
                    .iter()
                    .rev()
                    .find(|&&position| {
                        steps += 1;
                        !hung_from.contains(&(point, mounts[position].id))
                    })
                    .or_else(|| stack.last())
                    .copied(),
            );
            base.push(
                stack
                    .iter()
                    .find(|&&position| {
                        steps += 1;
                        mounts[position].parent.is_none_or(|parent| !ids.contains(&(point, parent)))
                    })
                    .or_else(|| stack.first())
                    .copied(),
            );
        }
        // The nearest point above each: the stack of points above the last one, less those
        // the next one is not below, ends with the next one's nearest.
        let mut nearest: Vec<Option<usize>> = Vec::with_capacity(points.len());
        let mut above: Vec<usize> = Vec::new();
        for (point, (path, _)) in points.iter().enumerate() {
            while let Some(&last) = above.last() {
                steps += 1;
                if path.starts_with(points[last].0) {
                    break;
                }
                above.pop();
            }
            nearest.push(above.last().copied());
            above.push(point);
        }
        // Reachable: the mount a stack hangs from is the topmost at its own path, no mount
        // lies between the two paths to cross into instead (that path is the nearest point
        // above), and the same holds above. A parent's point is above, so it sorts first and
        // is decided first: one pass in path order decides every point from its nearest.
        let mut visible = vec![false; points.len()];
        for point in 0..points.len() {
            steps += 1;
            let Some(base) = base[point] else { continue };
            visible[point] = match mounts[base].parent.and_then(|parent| by_id.get(&parent)) {
                // A mount whose parent is not listed, or is itself, is the root of what this
                // process sees.
                None => true,
                Some(&parent) if parent == base => true,
                Some(&parent) => {
                    let above = point_of[parent];
                    nearest[point] == Some(above) && top[above] == Some(parent) && visible[above]
                }
            };
        }
        let table = points
            .iter()
            .enumerate()
            .filter(|(point, _)| visible[*point])
            .filter_map(|(point, (path, _))| {
                top[point].map(|topmost| (path.to_path_buf(), mounts[topmost].automount))
            })
            .collect();
        (Self::Read(table), steps)
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

    /// The rule [`MountTable::from_mounts`] applied before review L2 on #209, walking each
    /// mount point's whole parent chain afresh: kept to hold the single pass to the same
    /// answer.
    #[cfg(unix)]
    fn walked_chain_by_chain(mounts: &[Mount]) -> MountTable {
        let mut at: BTreeMap<&Path, Vec<usize>> = BTreeMap::new();
        for (position, mount) in mounts.iter().enumerate() {
            at.entry(mount.point.as_path()).or_default().push(position);
        }
        let by_id: BTreeMap<u64, usize> =
            mounts.iter().enumerate().map(|(position, mount)| (mount.id, position)).collect();
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
        let visible = |point: &Path| {
            let mut point = point;
            for _ in 0..=mounts.len() {
                let Some(base) = at.get(point).and_then(|stack| base(stack)) else { return false };
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
        MountTable::Read(
            at.iter()
                .filter(|(point, _)| visible(point))
                .filter_map(|(point, stack)| {
                    top(stack).map(|topmost| (point.to_path_buf(), mounts[topmost].automount))
                })
                .collect(),
        )
    }

    /// One pass in path order decides every mount point as walking its chain did, on every
    /// table of up to a dozen mounts drawn from a few paths, odd spellings, and relative
    /// ones included: stacked, covered, overmounted ancestors (where a stack's top and base
    /// fall back to table order), duplicate and missing IDs, self-parents, and loops.
    #[cfg(unix)]
    #[test]
    fn one_pass_decides_each_point_as_walking_its_chain_did() {
        const POINTS: [&str; 16] = [
            "/", "/a", "/a/b", "/a/b/c", "/a/b/c/d", "/a-b", "/a/c", "/b", "/b/a", "/a//b",
            "/a/b/", "/c", "a", "", ".", "./a",
        ];
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut next = |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % bound
        };
        for _ in 0..40_000 {
            let count = 1 + next(12);
            let mounts: Vec<Mount> = (0..count)
                .map(|_| Mount {
                    id: 1 + next(count + 2),
                    parent: (next(10) != 0).then(|| next(count + 3)),
                    point: PathBuf::from(POINTS[usize::try_from(next(16)).expect("small")]),
                    automount: next(3) == 0,
                })
                .collect();
            let table = MountTable::from_mounts(&mounts);
            assert_eq!(table, walked_chain_by_chain(&mounts), "{:?}", described(&mounts));
        }
        // And so is a chain of nested mounts, whole and covered from above part of the way.
        for mounts in [nested(64, None), nested(64, Some(30))] {
            assert_eq!(MountTable::from_mounts(&mounts), walked_chain_by_chain(&mounts));
        }
    }

    #[cfg(unix)]
    fn described(mounts: &[Mount]) -> Vec<(u64, Option<u64>, &Path, bool)> {
        mounts
            .iter()
            .map(|mount| (mount.id, mount.parent, mount.point.as_path(), mount.automount))
            .collect()
    }

    /// The root, `/home/user` on it, and `depth` mounts below that, each nested in the one
    /// before, as fusermount lets any user make up to its `mount_max`; and, if `cover` names
    /// one of them, a later mount at its path that hangs from the root's.
    #[cfg(unix)]
    fn nested(depth: u64, cover: Option<u64>) -> Vec<Mount> {
        let mut point = PathBuf::from("/home/user");
        let mut mounts = vec![
            Mount { id: 1, parent: Some(0), point: PathBuf::from("/"), automount: false },
            Mount { id: 2, parent: Some(1), point: point.clone(), automount: false },
        ];
        for level in 0..depth {
            point.push("m");
            mounts.push(Mount {
                id: 3 + level,
                parent: Some(2 + level),
                point: point.clone(),
                automount: false,
            });
        }
        if let Some(level) = cover {
            let point = mounts[usize::try_from(level + 2).expect("small")].point.clone();
            mounts.push(Mount { id: 3 + depth, parent: Some(1), point, automount: true });
        }
        mounts
    }

    /// A table of 2,000 mounts nested in one another, which any local user can make with
    /// FUSE, is decided in steps bounded by a small multiple of its mounts: walking each
    /// mount's chain afresh took about two million (review L2 on #209). Every one is visible
    /// and mounted; covered from above at one level, that level is topped by what covers
    /// it, and nothing below it is visible.
    #[cfg(unix)]
    #[test]
    fn a_table_of_nested_mounts_takes_steps_linear_in_its_mounts() {
        const DEPTH: u64 = 2_000;
        let mounts = nested(DEPTH, None);
        let (table, steps) = MountTable::from_mounts_counting(&mounts);
        assert!(steps <= 5 * mounts.len(), "{steps} steps for {} mounts", mounts.len());
        let MountTable::Read(points) = &table else { panic!("read") };
        assert_eq!(points.len(), mounts.len(), "every nested mount is visible");
        let deepest = mounts.last().map(|mount| mount.point.clone()).expect("mounts");
        assert_eq!(table.at(&deepest), Some(MountAt::Mounted));

        let covered = nested(DEPTH, Some(1_000));
        let (table, steps) = MountTable::from_mounts_counting(&covered);
        assert!(steps <= 5 * covered.len(), "{steps} steps for {} mounts", covered.len());
        let level = |level: usize| covered[level + 2].point.as_path();
        assert_eq!(table.at(level(999)), Some(MountAt::Mounted));
        assert_eq!(table.at(level(1_000)), Some(MountAt::Automount), "topped by its cover");
        assert_eq!(table.at(level(1_001)), Some(MountAt::NotMounted), "hidden below it");
        assert_eq!(table.at(&deepest), Some(MountAt::NotMounted));
        let MountTable::Read(points) = &table else { panic!("read") };
        assert_eq!(points.len(), 2 + 1_001, "the root, the home, and levels 0 to 1,000");
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

    /// The table this host keeps reads, and holds the mount the root is on wherever one is
    /// mounted there. In a chroot whose root directory is no mount point, as distribution
    /// build tools make, Linux's table has no line for `/` (review I5 on #209), so there
    /// the root is asked of the table only where a line names it, and `/proc`, which the
    /// table is read from, is asked in any case.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn this_hosts_table_holds_its_root() {
        let table = MountTable::read();
        assert!(matches!(table, MountTable::Read(_)), "the table reads: {table:?}");
        #[cfg(target_os = "linux")]
        {
            assert_eq!(table.at(Path::new("/proc")), Some(MountAt::Mounted), "its own source");
            let text = std::fs::read("/proc/self/mountinfo").expect("mountinfo");
            let names_root = text
                .split(|byte| *byte == b'\n')
                .any(|line| line.split(|byte| *byte == b' ').nth(4) == Some(b"/".as_slice()));
            if !names_root {
                return;
            }
        }
        assert_eq!(table.at(Path::new("/")), Some(MountAt::Mounted));
    }
}
