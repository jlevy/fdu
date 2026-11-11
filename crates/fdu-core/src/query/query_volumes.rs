//! The volumes view: each filesystem a report touches, its capacity beside the bytes the
//! report counted there.
//!
//! Rows come from the walk's own device changes, so no mount table is read: a root's
//! filesystem, and any entry whose device differs from its parent's, which a walk without
//! `--one-filesystem` crosses onto at a mount point. Capacity comes from an observation the
//! serving route made just before it read ([`crate::capacity`]), and a reader given none
//! reports it as unknown, never as zero.
//!
//! Capacity is asked only where the walk listed, and below a root only where touching the
//! path mounts nothing. A capacity call resolves its path, and on an unmounted automount
//! trigger, or on a path through one, it asks the automounter to mount it and waits for the
//! answer, as a listing would. Each root is asked: the user named it, and reading its device
//! opens it in any case. A directory the walk kept without listing it (at the scan depth, or
//! not yet reached by an opened root's discovery) keeps its row with capacity unknown and
//! the reason ([`NOT_LISTED`]), never a call the walk itself would not have made.
//!
//! And a listing is a fact about a walk that may be hours old, while the call is made now:
//! an automounter unmounts what it mounted once it sits idle, and a remount or a reboot
//! numbers many filesystems afresh. The stat that never mounts cannot tell which paths are
//! safe to touch, since it covers only a path's last component, and only where that
//! component exists. So every directory below a root is first looked up in the mount table,
//! which resolves no path ([`MountTable`]), and asked only where the table lists a mount at
//! exactly its path, visible there, whose topmost filesystem is not an automount trigger
//! ([`NOT_A_MOUNT_POINT`], [`AUTOMOUNT_NOT_MOUNTED`]); then, the path being a mount point,
//! the stat that never mounts confirms it is on the device the walk recorded
//! ([`NOT_ON_RECORDED_DEVICE`]). One that fails keeps its row and its counts with the
//! reason, and blocks nothing below it: the table lookup is what makes a path safe, so
//! each directory is looked up on its own, and a mount the table lists below a device
//! boundary that is no mount point, such as a btrfs subvolume or a macOS firmlink, is asked
//! like any other. A root is held to the same device rule: it is asked where it is still on
//! the device its walk recorded, which keys its row ([`Index::root_device`]).
//!
//! The gate is exact for the table as it was read, and no further. `observe_entries`
//! reads it once, then checks and asks each directory in turn, by path, so a mount that
//! expires between that read and a directory's own check or call (an idle autofs key, an
//! NFS submount) is resolved afresh and asked for again: `statfs` resolves with automounting
//! on. The window is as long as the checks and calls made before it, so one slow network
//! filesystem stretches it; asking on a descriptor opened when the table is read would hold
//! each mount busy throughout (fdu-pxd2).
//!
//! No row is summed with another. Available space can be shared between volumes of one
//! pool, and the bytes a report counted and the bytes in use never add to the capacity:
//! snapshots, other volumes, purgeable space, and deleted files still open all take space
//! no walk sees.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::EntryKind;
use crate::capacity::{CapacityObservation, MountAt, MountTable, VolumeCapacity};
use crate::index::{EntryId, Index};

/// What a serving route observed for a report that names `volumes`: the device each root
/// is keyed by, and the capacity of every filesystem the report touches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VolumesObservation {
    /// One device per root, in the roots' order, that the root's row is keyed by: the device
    /// its walk recorded ([`Index::root_device`]), or for an index no walk built, which
    /// recorded none, the device the root is on now; `None` where neither is known.
    pub root_devices: Vec<Option<u64>>,
    /// The capacity of each filesystem, by device.
    pub capacity: CapacityObservation,
}

/// Files and bytes a report counted on one filesystem.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VolumeCounted {
    /// Regular files.
    pub files: u64,
    /// Apparent bytes.
    pub bytes: u64,
    /// Allocated bytes.
    pub allocated: u64,
}

/// Where a report enters one filesystem: a root, or the entry where its walk crossed onto
/// another device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VolumeEntry {
    /// The position of the entry's root among the report's roots; 0 for one root.
    pub root: usize,
    /// The device the walk identified the filesystem by.
    pub device: u64,
    /// The entry's path relative to its root; empty for the root itself.
    pub path: PathBuf,
    /// The entry's absolute path, which a capacity call is made on.
    pub absolute: PathBuf,
    /// Whether the walk listed the entry: a root always counts as listed, and a directory
    /// below one when its scope admitted descent into it and the index holds its listing.
    /// A capacity call is made only on a listed entry, and below a root only where the
    /// mount table lists a mount at exactly its path that is no automount trigger and the
    /// stat that never mounts then finds it on [`Self::device`].
    pub listed: bool,
}

/// One filesystem a report touches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VolumeRow {
    /// The device the walk identified the filesystem by.
    pub device: u64,
    /// The root of the path that labels the row.
    pub root: usize,
    /// Where the report entered the filesystem, relative to [`Self::root`]: the
    /// bytewise-first of every path where it did.
    pub path: PathBuf,
    /// [`Self::path`] as an absolute path.
    pub absolute: PathBuf,
    /// Every other path the report entered this filesystem at, each as its root's
    /// position and its path relative to that root, in the byte order of their absolute
    /// paths. Machine output lists them as `also_entered`, each written as a row's path is.
    pub also_entered: Vec<(usize, PathBuf)>,
    /// The observed capacity, or `None` when it is unknown.
    pub capacity: Option<VolumeCapacity>,
    /// Why the capacity is unknown, when it is.
    pub unknown_reason: Option<String>,
    /// What the report counted on this filesystem.
    pub counted: VolumeCounted,
}

/// The filesystems a report touches, with capacity observed at one instant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VolumesSection {
    /// When capacity was observed, in epoch nanoseconds; `None` when it was not, or the
    /// instant is unrepresentable.
    pub observed_at_ns: Option<i64>,
    /// One row per filesystem, ordered by the path that labels it.
    pub rows: Vec<VolumeRow>,
}

/// Why a row's capacity is unknown when the reader was given no observation.
pub(crate) const NOT_OBSERVED: &str = "capacity was not observed";

/// Why a row's capacity is unknown when the reader was given an observation that does not
/// hold its filesystem: an opened root entered it after observing capacity and before the
/// read, which takes its guard again.
pub(crate) const NOT_IN_OBSERVATION: &str = "the filesystem was not observed";

/// Why a row's capacity is unknown when the walk listed none of the directories where the
/// report entered its filesystem, so no capacity call was made there.
pub(crate) const NOT_LISTED: &str = "the walk did not list this directory";

/// Why a row's capacity is unknown when the device found at its path now is not the one the
/// walk recorded there: the filesystem was unmounted, expired, replaced, or numbered afresh
/// by a remount or a reboot, or the directory was removed. For a root, the device is its
/// live one; below a root, the one the stat that never mounts finds at a mount point.
pub(crate) const NOT_ON_RECORDED_DEVICE: &str =
    "not found on the device the walk recorded (unmounted, expired, or renumbered since)";

/// Why a row's capacity is unknown when the mount table lists no mount at the path where
/// the walk entered its filesystem: an automount that has expired since the walk, whose
/// directory the automounter may have removed, a filesystem unmounted, or a device boundary
/// that is no mount point, such as a btrfs subvolume. The path is never touched: touching
/// it could ask an automounter for it.
pub(crate) const NOT_A_MOUNT_POINT: &str = "not a mount point now, so it was not asked";

/// Why a row's capacity is unknown when the topmost filesystem the mount table lists at its
/// path is autofs: a direct trigger whose mount has expired, which a call would ask the
/// automounter for, or an indirect map's own directory, which holds no capacity of its own.
pub(crate) const AUTOMOUNT_NOT_MOUNTED: &str =
    "an automount point (autofs) is topmost here, so it was not asked";

/// Why a row's capacity is unknown when the mount table could not be read, so no path below
/// a root can be shown safe to touch.
pub(crate) const NO_MOUNT_TABLE: &str = "the mount table could not be read, so it was not asked";

/// Where a walk entered one filesystem below a root: the device, the path relative to the
/// root, and whether the walk listed that directory.
pub(super) type Entered = (u64, PathBuf, bool);

/// Whether the walk that built `index` listed directory `id` at `path`, the condition for
/// asking its filesystem's capacity.
///
/// Listed: its scope admitted descent into it, as the walk decides it
/// ([`crate::admission::should_descend`]: a directory at the scan depth is kept and not
/// listed), and the index holds its listing: it is complete, or it holds an entry, which a
/// walk reaches only through its directory. A directory an opened root's discovery has
/// kept and not yet listed is neither, and neither is one whose listing failed, where a
/// second call would wait on the same mount again.
fn walk_listed(index: &Index, id: EntryId, path: &Path) -> bool {
    let depth = path.components().count();
    index.scope().max_depth.is_none_or(|maximum| depth < maximum)
        && (index.directory_complete_of(id) == Some(true)
            || index.children_of(id).is_some_and(|children| children.len() > 0))
}

/// One pass over `index`: where it enters each filesystem, whether the walk listed each
/// such directory, and what the regular files `counted` admits hold on each.
///
/// Only a directory's device starts a filesystem, and every other entry is counted on its
/// directory's. A mount point is a directory, and a file's own device can name something
/// else: overlayfs, which containers run on, reports each file's underlying layer while
/// its directories report the overlay, so a file's device would split one filesystem into
/// a row per layer. A bind-mounted file is counted on its directory's filesystem for the
/// same reason. A directory with no device of its own (zero, which a Windows walker
/// reports for an entry it could list but not open) is taken to be on its parent's.
///
/// The root is on `root_device`, the key an observation gave it, or else the device its
/// walk recorded ([`Index::root_device`]).
pub(super) fn devices(
    index: &Index,
    root_device: Option<u64>,
    counted: impl Fn(EntryId) -> bool,
) -> (Vec<Entered>, BTreeMap<u64, VolumeCounted>) {
    let root_device =
        root_device.filter(|device| *device != 0).or_else(|| index.root_device()).unwrap_or(0);
    // A one-filesystem walk retains each mount point it reached as a boundary and never
    // enters it, so the report touches no other filesystem.
    let crosses = !index.scope().one_filesystem;
    let mut entries = vec![(root_device, PathBuf::new(), true)];
    let mut tallies: BTreeMap<u64, VolumeCounted> = BTreeMap::new();
    let mut stack = vec![(EntryId::ROOT, PathBuf::new(), root_device)];
    while let Some((id, path, device)) = stack.pop() {
        let Some(children) = index.children_of(id) else { continue };
        for (name, child) in children {
            let (Some(kind), Some(attrs)) = (index.kind_of(child), index.attrs_of(child)) else {
                continue;
            };
            if kind == EntryKind::File && counted(child) {
                let tally = tallies.entry(device).or_default();
                tally.files += 1;
                tally.bytes += attrs.size;
                tally.allocated += attrs.allocated;
            } else if kind == EntryKind::Dir {
                let own = if attrs.dev == 0 { device } else { attrs.dev };
                let child_path = path.join(name);
                if own != device {
                    if !crosses {
                        continue;
                    }
                    entries.push((own, child_path.clone(), walk_listed(index, child, &child_path)));
                }
                stack.push((child, child_path, own));
            }
        }
    }
    (entries, tallies)
}

/// `relative` under `root`, without the separator `join` adds for an empty path, so the
/// root's own row reads `/srv`, not `/srv/`.
fn absolute(root: &Path, relative: &Path) -> PathBuf {
    if relative.as_os_str().is_empty() { root.to_path_buf() } else { root.join(relative) }
}

/// Where a report over `indexes` enters each filesystem, in the roots' order: what a
/// serving route observes before it reads a report that names `volumes`.
///
/// `root_devices` names the device each root's row is keyed by, in the roots' order, as
/// [`VolumesObservation::root_devices`] does; a root it leaves out is keyed by the device
/// its walk recorded.
pub fn volume_entries(indexes: &[&Index], root_devices: &[Option<u64>]) -> Vec<VolumeEntry> {
    indexes
        .iter()
        .enumerate()
        .flat_map(|(root, index)| {
            let (entries, _) = devices(index, root_devices.get(root).copied().flatten(), |_| false);
            entries.into_iter().map(move |(device, path, listed)| VolumeEntry {
                root,
                device,
                absolute: absolute(index.root_path(), &path),
                path,
                listed,
            })
        })
        .collect()
}

/// The device each root is on now, in the roots' order; `None` where a root cannot be read.
///
/// Read as the walk reads a root's device, so the two agree. This reads the filesystem, and
/// an opened root calls it before it takes its index guard.
pub(crate) fn root_devices<'a>(roots: impl IntoIterator<Item = &'a Path>) -> Vec<Option<u64>> {
    roots
        .into_iter()
        .map(|root| {
            std::fs::metadata(root).and_then(|meta| crate::scan::root_device(root, &meta)).ok()
        })
        .collect()
}

/// The device each root's row is keyed by, in the roots' order: the device its walk
/// recorded, or for an index that recorded none, the device it is on now (`live`).
pub(crate) fn root_keys(indexes: &[&Index], live: &[Option<u64>]) -> Vec<Option<u64>> {
    indexes
        .iter()
        .enumerate()
        .map(|(at, index)| index.root_device().or_else(|| live.get(at).copied().flatten()))
        .collect()
}

/// The device the stat that never mounts finds a directory at `path` on, or `None` where
/// it finds no directory or cannot read one.
///
/// [`crate::scan::observe_path`], which on Linux passes `AT_NO_AUTOMOUNT`. That covers the
/// path's last component, and only where it exists, so this is called only on a path the
/// mount table lists as a mount point whose topmost filesystem is no automount trigger:
/// every component then resolves through mounted filesystems, and nothing is mounted.
fn directory_device(path: &Path) -> Option<u64> {
    match crate::scan::observe_path(path) {
        Ok((EntryKind::Dir, attrs)) if attrs.dev != 0 => Some(attrs.dev),
        _ => None,
    }
}

/// Observe the capacity of each filesystem `entries` enter, on the first path where touching
/// it mounts nothing, given the device each root's row is keyed by (`root_devices`) and the
/// device each root is on now (`live`); a filesystem entered nowhere it passes is not asked.
pub(crate) fn observe_entries(
    root_devices: Vec<Option<u64>>,
    live: &[Option<u64>],
    entries: &[VolumeEntry],
) -> VolumesObservation {
    observe_entries_with(
        root_devices,
        live,
        entries,
        |root| std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf()),
        MountTable::read,
        directory_device,
        |volumes| crate::capacity::observe_capacity(volumes),
    )
}

/// [`observe_entries`], with each root's real path resolved by `real_path`, what is mounted
/// where read by `mounts`, each mount point's current device read by `current_device`, and
/// the capacity calls made by `observe`: the seams a test uses to see which paths would be
/// read and asked.
///
/// The table, and each root's real path, which the table names mount points by, are read
/// once, and only where a listed entry below a root needs them: a root is checked against
/// the device it is on now, never the table. So a report whose only listed entries are its
/// roots, under `--one-filesystem`, at `--scan-depth 0`, or over a tree that crosses no
/// mount, reads neither, and asks each root at the path its walk recorded (review K3 on
/// #209).
///
/// Entries are visited in the byte order of their real paths, so each filesystem is asked
/// at, or refused with the reason of, the first of its paths in that order, and every listed
/// entry is checked, one whose filesystem was already asked at an earlier path too. A root
/// is asked where `live` finds it on the device its row is keyed by. Any other entry is
/// looked up in `mounts`, under its root's real path, which the table names mount points
/// by, and passes only where the table lists a mount at exactly that path whose topmost
/// filesystem is no automount trigger, and `current_device` then finds it on its row's
/// device, which a walk records from the descriptor it opens
/// ([`crate::scan::entered_directory_attrs`]), so the filesystem the walk counted is the one
/// asked.
///
/// An entry that fails blocks none below it, since the lookup alone is what makes a path
/// safe to touch (review I1 on #209). A mount the table lists is visible at its path, so
/// resolving the path crosses mounted filesystems only; a mount covered from above since
/// the walk is not visible, and is not touched; and below an expired trigger or a
/// filesystem unmounted since, the table lists nothing, since a mount pins the one it hangs
/// from. What the table lists below autofs hangs from it, as a mounted key hangs from its
/// indirect map. So a mount below a device boundary that is no mount point, a btrfs
/// subvolume or a macOS firmlink, is asked like any other. A filesystem asked nowhere is
/// observed as the first reason one of its paths failed with.
fn observe_entries_with(
    root_devices: Vec<Option<u64>>,
    live: &[Option<u64>],
    entries: &[VolumeEntry],
    real_path: impl Fn(&Path) -> PathBuf,
    mounts: impl FnOnce() -> MountTable,
    current_device: impl Fn(&Path) -> Option<u64>,
    observe: impl for<'p> FnOnce(Vec<(u64, &'p Path)>) -> CapacityObservation,
) -> VolumesObservation {
    // Each root's real path, resolved once: the table names mount points by real path.
    // Neither is read where no listed entry below a root looks anything up in the table.
    let below = entries.iter().any(|entry| entry.listed && !entry.path.as_os_str().is_empty());
    let mut real_roots: BTreeMap<usize, PathBuf> = BTreeMap::new();
    if below {
        for entry in entries.iter().filter(|entry| entry.path.as_os_str().is_empty()) {
            real_roots.entry(entry.root).or_insert_with(|| real_path(&entry.absolute));
        }
    }
    let mounts = below.then(mounts);
    let mut listed: Vec<(&VolumeEntry, PathBuf)> = entries
        .iter()
        .filter(|entry| entry.listed)
        .map(|entry| {
            let real = real_roots
                .get(&entry.root)
                .map_or_else(|| entry.absolute.clone(), |root| absolute(root, &entry.path));
            (entry, real)
        })
        .collect();
    listed.sort_by(|left, right| {
        left.1.as_os_str().as_encoded_bytes().cmp(right.1.as_os_str().as_encoded_bytes())
    });
    let mut asked: Vec<(u64, PathBuf)> = Vec::new();
    let mut refused: Vec<(u64, PathBuf, &'static str)> = Vec::new();
    for (entry, real) in listed {
        let reason = if entry.path.as_os_str().is_empty() {
            // A root, which the user named and whose device the walk read by opening it.
            (live.get(entry.root).copied().flatten() != Some(entry.device))
                .then_some(NOT_ON_RECORDED_DEVICE)
        } else {
            // The table was read, as this listed entry is below a root; had it not been,
            // nothing would be shown safe to touch.
            match mounts.as_ref().and_then(|mounts| mounts.at(&real)) {
                None => Some(NO_MOUNT_TABLE),
                Some(MountAt::NotMounted) => Some(NOT_A_MOUNT_POINT),
                Some(MountAt::Automount) => Some(AUTOMOUNT_NOT_MOUNTED),
                Some(MountAt::Mounted) => {
                    (current_device(&real) != Some(entry.device)).then_some(NOT_ON_RECORDED_DEVICE)
                }
            }
        };
        match reason {
            None if !asked.iter().any(|(device, _)| *device == entry.device) => {
                asked.push((entry.device, real));
            }
            Some(reason) if !refused.iter().any(|(device, _, _)| *device == entry.device) => {
                refused.push((entry.device, real, reason));
            }
            // A filesystem asked or refused at an earlier path keeps that answer.
            None | Some(_) => {}
        }
    }
    let mut capacity =
        observe(asked.iter().map(|(device, path)| (*device, path.as_path())).collect());
    for (device, path, reason) in refused {
        if capacity.volume(device).is_none() {
            capacity.volumes.push(crate::capacity::ObservedVolume {
                device,
                path,
                capacity: Err(reason.to_owned()),
            });
        }
    }
    VolumesObservation { root_devices, capacity }
}

/// Observe the capacity of every filesystem a report of `request` over `indexes` touches,
/// or nothing when the request names no `volumes` view.
///
/// Each root is keyed by the device its walk recorded and asked where it is still on it,
/// and each other filesystem is asked only where the walk listed it, the mount table shows
/// touching it mounts nothing, and the stat that never mounts finds it on the device the
/// walk recorded, as the module documentation describes.
///
/// The one step a route that owns its indexes takes before it reads such a report, so that
/// one-shot, retained, and watching routes observe the same filesystems the same way. An
/// opened root takes the same steps with its guard released around the calls.
pub fn observe_volumes(
    indexes: &[&Index],
    request: &crate::query::Request,
) -> Option<VolumesObservation> {
    request.query.views.contains(&crate::query::ViewSpec::Volumes).then(|| {
        let live = root_devices(indexes.iter().map(|index| index.root_path()));
        let root_devices = root_keys(indexes, &live);
        let entries = volume_entries(indexes, &root_devices);
        observe_entries(root_devices, &live, &entries)
    })
}

/// One root's part of a volumes section: its position, its path, and what [`devices`]
/// found under it.
pub(super) type RootDevices<'a> = (usize, &'a Path, Vec<Entered>, BTreeMap<u64, VolumeCounted>);

/// Every path a report entered one device at, as (absolute, root, relative, listed), with
/// what it counted there.
type Reached = (Vec<(PathBuf, usize, PathBuf, bool)>, VolumeCounted);

/// The rows of a volumes section: every root's devices joined with `observation`.
pub(super) fn volumes_section(
    parts: Vec<RootDevices<'_>>,
    observation: Option<&VolumesObservation>,
) -> VolumesSection {
    let observation = observation.map(|seen| &seen.capacity);
    let mut by_device: BTreeMap<u64, Reached> = BTreeMap::new();
    for (root, root_path, entries, tallies) in parts {
        for (device, path, listed) in entries {
            by_device.entry(device).or_default().0.push((
                absolute(root_path, &path),
                root,
                path,
                listed,
            ));
        }
        for (device, tally) in tallies {
            let total = &mut by_device.entry(device).or_default().1;
            total.files += tally.files;
            total.bytes += tally.bytes;
            total.allocated += tally.allocated;
        }
    }
    let mut rows: Vec<VolumeRow> = by_device
        .into_iter()
        .filter_map(|(device, (mut entries, counted))| {
            entries.sort_by(|left, right| {
                left.0.as_os_str().as_encoded_bytes().cmp(right.0.as_os_str().as_encoded_bytes())
            });
            let listed = entries.iter().any(|entry| entry.3);
            let mut entries = entries.into_iter();
            let (absolute, root, path, _) = entries.next()?;
            let also_entered = entries.map(|(_, root, path, _)| (root, path)).collect();
            // The observation answers where it holds the device. Otherwise the reason is
            // the walk's, when no path it entered the filesystem at was listed and so none
            // was asked; else no observation was made, or the filesystem appeared after it.
            let (capacity, unknown_reason) = match observation.and_then(|seen| seen.volume(device))
            {
                Some(volume) => match &volume.capacity {
                    Ok(capacity) => (Some(capacity.clone()), None),
                    Err(reason) => (None, Some(reason.clone())),
                },
                None if !listed => (None, Some(NOT_LISTED.to_owned())),
                None if observation.is_none() => (None, Some(NOT_OBSERVED.to_owned())),
                None => (None, Some(NOT_IN_OBSERVATION.to_owned())),
            };
            Some(VolumeRow {
                device,
                root,
                path,
                absolute,
                also_entered,
                capacity,
                unknown_reason,
                counted,
            })
        })
        .collect();
    rows.sort_by(|left, right| {
        left.absolute
            .as_os_str()
            .as_encoded_bytes()
            .cmp(right.absolute.as_os_str().as_encoded_bytes())
    });
    VolumesSection {
        observed_at_ns: observation
            .and_then(|seen| crate::query::system_time_to_nanos(seen.observed_at)),
        rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_contract::{Attrs, Observation, Op};

    fn upsert(path: &str, kind: EntryKind, size: u64, dev: u64) -> Op {
        let attrs = Attrs { size, allocated: size, mtime_ns: 1, ctime_ns: 1, inode: size + 1, dev };
        Op::Upsert { path: PathBuf::from(path), kind, attrs }
    }

    /// `/srv` on device 1 holds `mnt`, a mount of device 2, and `layer`, whose files
    /// report another device as overlayfs files do.
    fn mounted() -> Index {
        let mut index = Index::new("/srv");
        index
            .apply(&Observation::new(vec![
                upsert("a.txt", EntryKind::File, 10, 1),
                upsert("mnt", EntryKind::Dir, 0, 2),
                upsert("mnt/b.bin", EntryKind::File, 20, 2),
                upsert("mnt/sub", EntryKind::Dir, 0, 0),
                upsert("mnt/sub/c.bin", EntryKind::File, 30, 2),
                upsert("layer", EntryKind::Dir, 0, 1),
                upsert("layer/d.bin", EntryKind::File, 40, 9),
            ]))
            .expect("apply");
        index
    }

    #[test]
    fn only_a_directory_starts_a_filesystem_and_files_count_on_their_directorys() {
        let index = mounted();
        let (entries, tallies) = devices(&index, Some(1), |_| true);
        assert_eq!(entries, [(1, PathBuf::new(), true), (2, PathBuf::from("mnt"), true)]);
        // `mnt/sub` has no device of its own and stays on `mnt`'s; `layer/d.bin` reports
        // a layer's device and counts on its directory's.
        assert_eq!(tallies[&1], VolumeCounted { files: 2, bytes: 50, allocated: 50 });
        assert_eq!(tallies[&2], VolumeCounted { files: 2, bytes: 50, allocated: 50 });
        // The selection decides what is counted, not which filesystems are listed.
        let (entries, tallies) = devices(&index, Some(1), |_| false);
        assert_eq!(entries.len(), 2);
        assert!(tallies.is_empty());
    }

    /// The paths `observe_entries` would ask capacity of for a report over `index`, whose
    /// root is on device 1, and the observation it would make, with every filesystem still
    /// mounted where the walk entered it and a stand-in for the calls that answers every
    /// path it is given.
    fn asked_of(index: &Index) -> (Vec<(u64, PathBuf)>, VolumesObservation) {
        let entries = volume_entries(&[index], &[Some(1)]);
        let mounts = mounted_at(entries.iter().map(|entry| (entry.absolute.clone(), false)));
        let mounted: Vec<(PathBuf, u64)> =
            entries.into_iter().map(|entry| (entry.absolute, entry.device)).collect();
        asked_with(index, &mounts, |path| {
            mounted.iter().find(|(at, _)| at == path).map(|(_, device)| *device)
        })
    }

    /// A mount table listing each path, and whether its topmost filesystem is an automount
    /// trigger.
    fn mounted_at(points: impl IntoIterator<Item = (PathBuf, bool)>) -> MountTable {
        MountTable::Read(points.into_iter().collect())
    }

    /// [`asked_of`], with `mounts` standing for the mount table and `current` for the stat
    /// that never mounts: what is mounted where, and the device each mount point is on now.
    /// The root is on device 1 now, and its path is its real path.
    fn asked_with(
        index: &Index,
        mounts: &MountTable,
        current: impl Fn(&Path) -> Option<u64>,
    ) -> (Vec<(u64, PathBuf)>, VolumesObservation) {
        asked_live(index, Some(1), mounts, current)
    }

    /// [`asked_with`], with the root's device keyed as its walk recorded it, and `live` the
    /// device the root is on now.
    fn asked_live(
        index: &Index,
        live: Option<u64>,
        mounts: &MountTable,
        current: impl Fn(&Path) -> Option<u64>,
    ) -> (Vec<(u64, PathBuf)>, VolumesObservation) {
        let keys = root_keys(&[index], &[live]);
        let entries = volume_entries(&[index], &keys);
        let mut asked = Vec::new();
        let observation = observe_entries_with(
            keys,
            &[live],
            &entries,
            Path::to_path_buf,
            || mounts.clone(),
            current,
            |volumes| {
                asked =
                    volumes.iter().map(|(device, path)| (*device, path.to_path_buf())).collect();
                answered(volumes)
            },
        );
        (asked, observation)
    }

    /// A stand-in for the capacity calls that answers every path it is given.
    fn answered(volumes: Vec<(u64, &Path)>) -> CapacityObservation {
        CapacityObservation {
            observed_at: std::time::UNIX_EPOCH,
            volumes: volumes
                .into_iter()
                .map(|(device, path)| crate::capacity::ObservedVolume {
                    device,
                    path: path.to_path_buf(),
                    capacity: Ok(VolumeCapacity {
                        capacity_bytes: 100,
                        available_bytes: 50,
                        filesystem: None,
                    }),
                })
                .collect(),
        }
    }

    /// Reading the mount table costs a read of the whole table, which on a host with
    /// thousands of mounts takes milliseconds, and a root is checked against its live device,
    /// never looked up in it. So a report whose only listed entries are its roots reads no
    /// table and resolves no root's real path: under `--one-filesystem`, at `--scan-depth 0`,
    /// where it kept a crossing without listing it, or over a tree that crosses no mount.
    /// One with a listed entry below a root reads the table exactly once, however many such
    /// entries and roots it has (review K3 on #209).
    #[test]
    fn only_a_listed_entry_below_a_root_reads_the_mount_table() {
        let scoped = |scope: crate::ScanScope, ops: Vec<Op>| {
            let mut index = Index::new_with_scope("/srv", scope);
            index.apply(&Observation::new(ops)).expect("apply");
            index
        };
        let kept =
            || vec![upsert("a.txt", EntryKind::File, 10, 1), upsert("mnt", EntryKind::Dir, 0, 2)];
        let one_filesystem = crate::ScanScope { one_filesystem: true, ..Default::default() };
        let boundary = scoped(one_filesystem, kept());
        let depth_zero = Index::new_with_scope(
            "/srv",
            crate::ScanScope { max_depth: Some(0), ..Default::default() },
        );
        let unlisted =
            scoped(crate::ScanScope { max_depth: Some(1), ..Default::default() }, kept());
        let flat = walked(1, None, crate::ScanScope::default());
        let crossing = mounted();
        let twice = scoped(
            crate::ScanScope::default(),
            vec![
                upsert("mnt", EntryKind::Dir, 0, 2),
                upsert("mnt/b.bin", EntryKind::File, 20, 2),
                upsert("zz", EntryKind::Dir, 0, 3),
                upsert("zz/c.bin", EntryKind::File, 30, 3),
            ],
        );
        let mut other = Index::new("/data");
        other
            .apply(&Observation::new(vec![upsert("e.bin", EntryKind::File, 5, 1)]))
            .expect("apply");
        let srv = |components: &[&str]| {
            components.iter().fold(PathBuf::from("/srv"), |path, part| path.join(part))
        };

        // What observing a report over `indexes`, each root on device 1, read: how many
        // times it read the table, the roots it resolved, and the paths it asked.
        let observed = |indexes: &[&Index]| {
            let keys = vec![Some(1); indexes.len()];
            let entries = volume_entries(indexes, &keys);
            let tables = std::cell::Cell::new(0);
            let resolved = std::cell::RefCell::new(Vec::new());
            let mut asked = Vec::new();
            observe_entries_with(
                keys.clone(),
                &keys,
                &entries,
                |root| {
                    resolved.borrow_mut().push(root.to_path_buf());
                    root.to_path_buf()
                },
                || {
                    tables.set(tables.get() + 1);
                    mounted_at([(srv(&["mnt"]), false)])
                },
                |path| (path == srv(&["mnt"])).then_some(2),
                |volumes| {
                    asked = volumes
                        .iter()
                        .map(|(device, path)| (*device, path.to_path_buf()))
                        .collect::<Vec<_>>();
                    answered(volumes)
                },
            );
            (tables.get(), resolved.into_inner(), asked)
        };

        let root_only: [&[&Index]; 6] = [
            &[&boundary],
            &[&depth_zero],
            &[&unlisted],
            &[&flat],
            &[&flat, &other],
            &[&other, &boundary],
        ];
        for indexes in root_only {
            let (tables, resolved, asked) = observed(indexes);
            assert_eq!((tables, resolved), (0, vec![]), "{:?}", indexes[0].scope());
            let first = if indexes.len() == 1 { "/srv" } else { "/data" };
            assert_eq!(asked, [(1, PathBuf::from(first))], "each root is still asked");
        }
        assert_eq!(
            observed(&[&crossing]),
            (1, vec![srv(&[])], vec![(1, srv(&[])), (2, srv(&["mnt"]))]),
            "a listed crossing reads the table once"
        );
        assert_eq!(
            observed(&[&twice, &other]),
            (
                1,
                vec![srv(&[]), PathBuf::from("/data")],
                vec![(1, PathBuf::from("/data")), (2, srv(&["mnt"]))]
            ),
            "and so do two crossings under two roots, each root resolved once"
        );
    }

    /// The volume rows of a report over `index` given `observation`, as (path, device,
    /// whether capacity is known, the reason it is not).
    fn rows_of(
        index: &Index,
        observation: &VolumesObservation,
    ) -> Vec<(PathBuf, u64, bool, Option<String>)> {
        use crate::query::{Query, Request, Section, ViewSpec};
        let request = Request::new(
            crate::query::Basis::held_by(index),
            Query { views: vec![ViewSpec::Volumes], ..Query::default() },
            std::time::UNIX_EPOCH,
        );
        let report =
            crate::query::report(index, &request, std::time::UNIX_EPOCH, Some(observation))
                .expect("report");
        let Section::Volumes(volumes) = &report.sections[0] else { panic!("volumes") };
        volumes
            .rows
            .iter()
            .map(|row| {
                (row.path.clone(), row.device, row.capacity.is_some(), row.unknown_reason.clone())
            })
            .collect()
    }

    /// `--view full` describes the tree and leaves `volumes` out, so whatever analyzers
    /// it runs with, a full request observes no capacity on any route: each route asks
    /// [`observe_volumes`], or the same question of the request's views, before it reads
    /// (the maintainer's decision on review F13 of #209).
    #[test]
    fn a_full_report_asks_no_filesystem_for_its_capacity() {
        use crate::content::AnalysisSet;
        use crate::query::{Query, Request, ViewSpec};
        let root = tempfile::tempdir().expect("root");
        let (index, _) =
            crate::scan::scan_into_index(root.path(), &crate::ScanConfig::default()).expect("scan");
        for analysis in [AnalysisSet::NONE, AnalysisSet::ALL] {
            let (views, omitted) =
                ViewSpec::resolve(Some("full"), analysis, "view").expect("full resolves");
            assert!(!views.contains(&ViewSpec::Volumes), "{views:?}");
            assert!(!omitted.contains(&ViewSpec::Volumes), "not dropped, never part of it");
            assert!(views.contains(&ViewSpec::Ages), "full keeps ages: {views:?}");
            let request = Request::new(
                crate::query::Basis::held_by(&index),
                Query { views, omitted_views: omitted, ..Query::default() },
                std::time::SystemTime::now(),
            );
            assert!(observe_volumes(&[&index], &request).is_none(), "{analysis:?}");
        }
    }

    /// A capacity call on an unmounted automount trigger asks the kernel to mount it and
    /// waits for the automounter, as a listing would, so it is made only where the walk
    /// listed: never on a directory it kept at the scan depth, or one an opened root's
    /// discovery has not reached (review F1 on #209). The root is always asked.
    #[test]
    fn a_filesystem_entered_where_the_walk_did_not_list_is_never_asked() {
        // `--scan-depth 1`: the walk listed the root and kept `mnt`, a mount point at the
        // scan depth, without listing it.
        let mut shallow = Index::new_with_scope(
            "/srv",
            crate::ScanScope { max_depth: Some(1), ..crate::ScanScope::default() },
        );
        shallow
            .apply(&Observation::new(vec![
                upsert("a.txt", EntryKind::File, 10, 1),
                upsert("mnt", EntryKind::Dir, 0, 2),
            ]))
            .expect("apply");
        let (asked, observation) = asked_of(&shallow);
        assert_eq!(asked, [(1, PathBuf::from("/srv"))], "only the root is asked");
        assert_eq!(
            rows_of(&shallow, &observation),
            [
                (PathBuf::new(), 1, true, None),
                (PathBuf::from("mnt"), 2, false, Some(NOT_LISTED.to_owned())),
            ],
            "the unlisted mount keeps its row, capacity unknown, with the walk's reason"
        );

        // An opened root mid-discovery: it has listed `mnt`, which holds what it found
        // there, and kept `later` from the root's listing without listing it yet.
        let mut discovering = Index::new("/srv");
        discovering
            .apply(&Observation::new(vec![
                upsert("mnt", EntryKind::Dir, 0, 2),
                upsert("mnt/b.bin", EntryKind::File, 20, 2),
                upsert("later", EntryKind::Dir, 0, 3),
            ]))
            .expect("apply");
        let (asked, observation) = asked_of(&discovering);
        assert_eq!(asked, [(1, PathBuf::from("/srv")), (2, PathBuf::from("/srv/mnt"))]);
        let rows = rows_of(&discovering, &observation);
        assert_eq!(rows[1], (PathBuf::from("later"), 3, false, Some(NOT_LISTED.to_owned())));
        assert!(rows[2].2, "the listed mount's capacity is known: {rows:?}");
    }

    /// A filesystem entered at a listed path and an unlisted one is asked on the listed
    /// path, whichever path labels its row.
    #[test]
    fn a_filesystem_entered_twice_is_asked_where_the_walk_listed_it() {
        let mut index = Index::new("/srv");
        index
            .apply(&Observation::new(vec![
                upsert("a", EntryKind::Dir, 0, 2),
                upsert("b", EntryKind::Dir, 0, 1),
                upsert("b/mnt", EntryKind::Dir, 0, 2),
                upsert("b/mnt/c.bin", EntryKind::File, 5, 2),
            ]))
            .expect("apply");
        let (asked, observation) = asked_of(&index);
        assert_eq!(asked, [(1, PathBuf::from("/srv")), (2, PathBuf::from("/srv/b/mnt"))]);
        let rows = rows_of(&index, &observation);
        assert_eq!(rows[1].0, PathBuf::from("a"), "the bytewise-first path labels the row");
        assert!(rows[1].2, "and its capacity is the listed path's: {rows:?}");
    }

    /// A listing is a fact about a walk that may be hours old, and the capacity call is made
    /// now. An automounter unmounts what it mounted once it sits idle, so a call on a
    /// directory the walk listed would mount it again and wait (review G1 on #209), and the
    /// stat that never mounts cannot say where it is safe to look: it covers only a path's
    /// last component, where that exists (review H1). So each directory is looked up in the
    /// mount table first, and asked only where the table lists a mount at exactly its path
    /// that is no automount trigger, and the stat, safe there, finds the recorded device.
    /// One that fails blocks nothing below it: each is looked up on its own, and a mount
    /// the table lists below one is read and asked (review I1).
    #[test]
    fn a_filesystem_no_longer_mounted_where_the_walk_entered_it_is_never_asked() {
        let mut index = Index::new("/srv");
        index
            .apply(&Observation::new(vec![
                upsert("a.txt", EntryKind::File, 10, 1),
                upsert("mnt", EntryKind::Dir, 0, 2),
                upsert("mnt/b.bin", EntryKind::File, 20, 2),
                upsert("mnt/inner", EntryKind::Dir, 0, 3),
                upsert("mnt/inner/c.bin", EntryKind::File, 30, 3),
                upsert("other", EntryKind::Dir, 0, 4),
                upsert("other/d.bin", EntryKind::File, 40, 4),
                upsert("gone", EntryKind::Dir, 0, 5),
                upsert("gone/e.bin", EntryKind::File, 50, 5),
                upsert("gone/below", EntryKind::Dir, 0, 7),
                upsert("gone/below/f.bin", EntryKind::File, 60, 7),
                upsert("re", EntryKind::Dir, 0, 6),
                upsert("re/g.bin", EntryKind::File, 70, 6),
                upsert("re/in", EntryKind::Dir, 0, 8),
                upsert("re/in/h.bin", EntryKind::File, 80, 8),
            ]))
            .expect("apply");
        let at = |components: &[&str]| {
            components.iter().fold(PathBuf::from("/srv"), |path, part| path.join(part))
        };
        let mounts = mounted_at([
            // A direct trigger whose automount expired: the trigger is topmost again, and
            // what was mounted below it is gone with it.
            (at(&["mnt"]), true),
            (at(&["other"]), false),
            // Mounted again since, under a new number.
            (at(&["re"]), false),
            (at(&["re", "in"]), false),
            // `gone` is no mount point now, and `gone/below` is mounted at no path.
        ]);
        let read = std::cell::RefCell::new(Vec::new());
        let (asked, observation) = asked_with(&index, &mounts, |path| {
            read.borrow_mut().push(path.to_path_buf());
            if path == at(&["other"]) {
                Some(4)
            } else if path == at(&["re"]) {
                Some(9)
            } else if path == at(&["re", "in"]) {
                Some(8)
            } else {
                panic!("read a path the table does not show safe to touch: {path:?}")
            }
        });
        assert_eq!(
            asked,
            [(1, PathBuf::from("/srv")), (4, at(&["other"])), (8, at(&["re", "in"]))],
            "a mount the table lists below one on another device is asked"
        );
        assert_eq!(
            read.into_inner(),
            [at(&["other"]), at(&["re"]), at(&["re", "in"])],
            "only mount points the table lists are read, never the root"
        );
        assert_eq!(
            rows_of(&index, &observation),
            [
                (PathBuf::new(), 1, true, None),
                (PathBuf::from("gone"), 5, false, Some(NOT_A_MOUNT_POINT.to_owned())),
                (["gone", "below"].iter().collect(), 7, false, Some(NOT_A_MOUNT_POINT.to_owned())),
                (PathBuf::from("mnt"), 2, false, Some(AUTOMOUNT_NOT_MOUNTED.to_owned())),
                (["mnt", "inner"].iter().collect(), 3, false, Some(NOT_A_MOUNT_POINT.to_owned())),
                (PathBuf::from("other"), 4, true, None),
                (PathBuf::from("re"), 6, false, Some(NOT_ON_RECORDED_DEVICE.to_owned())),
                (["re", "in"].iter().collect(), 8, true, None),
            ],
            "each keeps its row, with the counts the walk made, and says why"
        );
        // Where the table cannot be read, nothing below a root is shown safe to touch.
        let (asked, observation) = asked_with(&index, &MountTable::Unreadable, |path| {
            panic!("read {path:?} without a mount table")
        });
        assert_eq!(asked, [(1, PathBuf::from("/srv"))]);
        assert_eq!(
            rows_of(&index, &observation)[1],
            (PathBuf::from("gone"), 5, false, Some(NO_MOUNT_TABLE.to_owned()))
        );
    }

    /// A filesystem entered at two paths, where it is no longer mounted at the first, is
    /// asked at the second.
    #[test]
    fn a_filesystem_still_mounted_at_another_path_is_asked_there() {
        let mut index = Index::new("/srv");
        index
            .apply(&Observation::new(vec![
                upsert("a", EntryKind::Dir, 0, 2),
                upsert("a/f.bin", EntryKind::File, 5, 2),
                upsert("b", EntryKind::Dir, 0, 2),
                upsert("b/g.bin", EntryKind::File, 5, 2),
            ]))
            .expect("apply");
        let b = PathBuf::from("/srv").join("b");
        let (asked, observation) =
            asked_with(&index, &mounted_at([(b.clone(), false)]), |path| (path == b).then_some(2));
        assert_eq!(asked, [(1, PathBuf::from("/srv")), (2, b)]);
        assert!(rows_of(&index, &observation)[1].2, "the second path answers for the row");
    }

    /// Every entry is checked, one whose filesystem was already asked at an earlier path
    /// too: skipped, it was never found unmounted, and a mount below it was read through it,
    /// which mounted it again (review H1 case 3 on #209).
    #[test]
    fn an_entry_whose_filesystem_was_asked_elsewhere_is_still_checked() {
        let mut index = Index::new("/srv");
        index
            .apply(&Observation::new(vec![
                upsert("a", EntryKind::Dir, 0, 2),
                upsert("a/f.bin", EntryKind::File, 5, 2),
                upsert("z", EntryKind::Dir, 0, 2),
                upsert("z/inner", EntryKind::Dir, 0, 3),
                upsert("z/inner/g.bin", EntryKind::File, 5, 3),
            ]))
            .expect("apply");
        let at = |components: &[&str]| {
            components.iter().fold(PathBuf::from("/srv"), |path, part| path.join(part))
        };
        // `z`, a second trigger for `a`'s filesystem, has expired, and `inner` with it.
        let mounts = mounted_at([(at(&["a"]), false), (at(&["z"]), true)]);
        let (asked, observation) = asked_with(&index, &mounts, |path| {
            assert_eq!(path, at(&["a"]), "read below an expired trigger");
            Some(2)
        });
        assert_eq!(asked, [(1, PathBuf::from("/srv")), (2, at(&["a"]))]);
        assert_eq!(
            rows_of(&index, &observation)[2],
            (["z", "inner"].iter().collect(), 3, false, Some(NOT_A_MOUNT_POINT.to_owned()))
        );
    }

    /// A mount the table lists below a device boundary that is no mount point is asked: a
    /// tmpfs below a btrfs subvolume, which has a device of its own and is mounted nowhere,
    /// as a disk under `/Volumes` lies below a macOS firmlink. Refused for the boundary
    /// above it, it read as unknown on every fresh walk (review I1 on #209). The boundary
    /// itself is still no mount point, and a mount covered from above since the walk, which
    /// the table lists but no path reaches, is still never touched.
    #[cfg(unix)]
    #[test]
    fn a_mount_below_a_boundary_that_is_no_mount_point_is_asked() {
        let mut index = Index::new("/srv");
        index
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: PathBuf::new(),
                    kind: EntryKind::Dir,
                    attrs: Index::root_attrs(1),
                },
                upsert("top.bin", EntryKind::File, 3, 1),
                // The subvolume: a device boundary, mounted nowhere.
                upsert("home", EntryKind::Dir, 0, 10),
                upsert("home/alice", EntryKind::Dir, 0, 10),
                upsert("home/alice/a.bin", EntryKind::File, 7, 10),
                upsert("home/alice/mnt", EntryKind::Dir, 0, 11),
                upsert("home/alice/mnt/m.bin", EntryKind::File, 1, 11),
                upsert("plain", EntryKind::Dir, 0, 1),
                upsert("plain/mnt2", EntryKind::Dir, 0, 12),
                upsert("plain/mnt2/n.bin", EntryKind::File, 2, 12),
                upsert("cover", EntryKind::Dir, 0, 1),
                upsert("cover/hidden", EntryKind::Dir, 0, 13),
                upsert("cover/hidden/h.bin", EntryKind::File, 4, 13),
            ]))
            .expect("apply");
        // What `/proc/self/mountinfo` held for the reproduction: both tmpfs mounts hang
        // from the btrfs mount, and nothing is mounted at the subvolume. Since the walk, a
        // mount at `cover` has covered the one at `cover/hidden`.
        let mounts = MountTable::from_mountinfo(
            b"\
1 0 0:1 / / rw - ext4 /dev/vda rw\n\
20 1 0:20 / /srv rw,relatime - btrfs /dev/loop0 rw,space_cache=v2,subvolid=5\n\
21 20 0:21 / /srv/home/alice/mnt rw,relatime - tmpfs tmpfs rw,size=8192k\n\
22 20 0:22 / /srv/plain/mnt2 rw,relatime - tmpfs tmpfs rw,size=8192k\n\
23 20 0:23 / /srv/cover/hidden rw,relatime - tmpfs tmpfs rw\n\
24 20 0:24 / /srv/cover rw,relatime - tmpfs tmpfs rw\n",
        );
        let at = |components: &[&str]| {
            components.iter().fold(PathBuf::from("/srv"), |path, part| path.join(part))
        };
        let read = std::cell::RefCell::new(Vec::new());
        let (asked, observation) = asked_with(&index, &mounts, |path| {
            read.borrow_mut().push(path.to_path_buf());
            if path == at(&["home", "alice", "mnt"]) {
                Some(11)
            } else if path == at(&["plain", "mnt2"]) {
                Some(12)
            } else {
                panic!("read a path the table does not show safe to touch: {path:?}")
            }
        });
        assert_eq!(
            asked,
            [
                (1, PathBuf::from("/srv")),
                (11, at(&["home", "alice", "mnt"])),
                (12, at(&["plain", "mnt2"])),
            ]
        );
        assert_eq!(read.into_inner(), [at(&["home", "alice", "mnt"]), at(&["plain", "mnt2"])]);
        assert_eq!(
            rows_of(&index, &observation),
            [
                (PathBuf::new(), 1, true, None),
                (
                    ["cover", "hidden"].iter().collect(),
                    13,
                    false,
                    Some(NOT_A_MOUNT_POINT.to_owned())
                ),
                (PathBuf::from("home"), 10, false, Some(NOT_A_MOUNT_POINT.to_owned())),
                (["home", "alice", "mnt"].iter().collect(), 11, true, None),
                (["plain", "mnt2"].iter().collect(), 12, true, None),
            ]
        );
    }

    /// `/srv` as a walk recorded it on device `root`: `top.txt`, `alpha` with two files,
    /// `beta`, and `gamma`, each file and directory on `root`, and `mount` at `name` on
    /// `device` with two files of its own, which a one-filesystem walk keeps as a boundary.
    fn walked(root: u64, mount: Option<(&str, u64)>, scope: crate::ScanScope) -> Index {
        let mut index = Index::new_with_scope("/srv", scope);
        let mut ops = vec![
            Op::Upsert {
                path: PathBuf::new(),
                kind: EntryKind::Dir,
                attrs: Index::root_attrs(root),
            },
            upsert("top.txt", EntryKind::File, 4, root),
            upsert("alpha", EntryKind::Dir, 0, root),
            upsert("alpha/a.bin", EntryKind::File, 30_000, root),
            upsert("alpha/x", EntryKind::Dir, 0, root),
            upsert("alpha/x/deep.bin", EntryKind::File, 20_000, root),
            upsert("beta", EntryKind::Dir, 0, root),
            upsert("beta/b.bin", EntryKind::File, 10_000, root),
            upsert("gamma", EntryKind::Dir, 0, root),
            upsert("gamma/g.txt", EntryKind::File, 2, root),
        ];
        if let Some((name, device)) = mount {
            ops.push(upsert(name, EntryKind::Dir, 0, device));
            if !scope.one_filesystem {
                ops.push(upsert(&format!("{name}/m.bin"), EntryKind::File, 7, device));
                ops.push(upsert(&format!("{name}/n.bin"), EntryKind::File, 9, device));
            }
        }
        index.apply(&Observation::new(ops)).expect("apply");
        index
    }

    /// A report keys its root by the device its walk recorded, on every route, however the
    /// host has numbered the root since (review H2 on #209). Inferred from its children's
    /// numbers, as it was, a root could take a mount's number, a number dealt again at a
    /// remount, or the filesystem under an unplugged disk's mount point, and a
    /// one-filesystem report then counted one file of five. And a root no longer on that
    /// device keeps its row and its counts, capacity unknown, as a nested filesystem does
    /// (review H6).
    #[test]
    fn a_root_is_keyed_by_the_device_its_walk_recorded() {
        // A one-filesystem request needs platform device identity, which Windows refuses.
        let one_filesystem =
            crate::ScanScope { one_filesystem: true, ..crate::ScanScope::default() };
        let scopes: Vec<crate::ScanScope> = std::iter::once(crate::ScanScope::default())
            .chain(cfg!(unix).then_some(one_filesystem))
            .collect();
        let no_reads = |path: &Path| -> Option<u64> { panic!("read {path:?}, no mount point") };
        // (a) The root was 40 and a mount at `zzz` 41; remounted, the root was dealt 41.
        // (b) A mount at `aaa`, which sorts first, is gone, and the root was dealt 41.
        // (c) The root's filesystem is not mounted: its path is on the one beneath, 65024.
        for (case, mount, live) in
            [("a", Some(("zzz", 41)), 41), ("b", Some(("aaa", 42)), 41), ("c", None, 65_024)]
        {
            for scope in scopes.iter().copied() {
                let index = walked(40, mount, scope);
                let (asked, observation) =
                    asked_live(&index, Some(live), &mounted_at([]), no_reads);
                assert_eq!(observation.root_devices, [Some(40)], "({case}) {scope:?}");
                assert!(asked.is_empty(), "({case}) nothing is asked: {asked:?}");
                let rows = rows_of(&index, &observation);
                assert_eq!(
                    rows[0],
                    (PathBuf::new(), 40, false, Some(NOT_ON_RECORDED_DEVICE.to_owned())),
                    "({case}) {scope:?}"
                );
                let (entries, tallies) = devices(&index, observation.root_devices[0], |_| true);
                assert_eq!(tallies[&40].files, 5, "({case}) the summary's five files: {scope:?}");
                match (mount, scope.one_filesystem) {
                    (Some((name, device)), false) => {
                        assert_eq!(
                            rows[1],
                            (
                                PathBuf::from(name),
                                device,
                                false,
                                Some(NOT_A_MOUNT_POINT.to_owned())
                            ),
                            "({case})"
                        );
                        assert_eq!(tallies[&device].files, 2, "({case})");
                    }
                    _ => assert_eq!(entries, [(40, PathBuf::new(), true)], "({case}) {scope:?}"),
                }
            }
        }

        // (d) The root is an indirect map, mounted at 41, whose keys were mounted at the walk
        // and have expired: the map is asked, as a root always is, and no key is touched.
        let mut map = Index::new("/srv");
        map.apply(&Observation::new(vec![
            Op::Upsert { path: PathBuf::new(), kind: EntryKind::Dir, attrs: Index::root_attrs(41) },
            upsert("alice", EntryKind::Dir, 0, 40),
            upsert("alice/one.bin", EntryKind::File, 7, 40),
            upsert("bob", EntryKind::Dir, 0, 43),
            upsert("bob/two.bin", EntryKind::File, 9, 43),
        ]))
        .expect("apply");
        let map_point = mounted_at([(PathBuf::from("/srv"), true)]);
        let (asked, observation) = asked_live(&map, Some(41), &map_point, no_reads);
        assert_eq!(asked, [(41, PathBuf::from("/srv"))]);
        assert_eq!(
            rows_of(&map, &observation),
            [
                (PathBuf::new(), 41, true, None),
                (PathBuf::from("alice"), 40, false, Some(NOT_A_MOUNT_POINT.to_owned())),
                (PathBuf::from("bob"), 43, false, Some(NOT_A_MOUNT_POINT.to_owned())),
            ]
        );

        // Unchanged since the walk: the root is asked on the device it was walked on.
        let index = walked(40, None, crate::ScanScope::default());
        let (asked, observation) = asked_live(&index, Some(40), &mounted_at([]), no_reads);
        assert_eq!(asked, [(40, PathBuf::from("/srv"))]);
        assert_eq!(rows_of(&index, &observation), [(PathBuf::new(), 40, true, None)]);
        // A root the reader is given no observation for is keyed by the same device.
        assert_eq!(devices(&index, None, |_| true).0, [(40, PathBuf::new(), true)]);

        // An index no walk built records no root device, and is keyed by the live one.
        let hand_built = Index::new("/empty");
        assert_eq!(hand_built.root_device(), None);
        assert_eq!(root_keys(&[&hand_built], &[Some(3)]), [Some(3)]);
        assert_eq!(root_keys(&[&hand_built], &[None]), [None]);
    }

    /// An opened root finds where it enters each filesystem under one guard, observes their
    /// capacity with it released, and reads under another, so a filesystem it entered in
    /// between has no observation. Its row says so, and borrows no other's capacity
    /// (review F2; G8 on #209).
    #[test]
    fn a_filesystem_entered_after_capacity_was_observed_reads_as_not_observed() {
        let index = mounted();
        let observation = VolumesObservation {
            root_devices: vec![Some(1)],
            capacity: CapacityObservation {
                observed_at: std::time::UNIX_EPOCH,
                volumes: vec![crate::capacity::ObservedVolume {
                    device: 1,
                    path: PathBuf::from("/srv"),
                    capacity: Ok(VolumeCapacity {
                        capacity_bytes: 100,
                        available_bytes: 50,
                        filesystem: None,
                    }),
                }],
            },
        };
        assert_eq!(
            rows_of(&index, &observation),
            [
                (PathBuf::new(), 1, true, None),
                (PathBuf::from("mnt"), 2, false, Some(NOT_IN_OBSERVATION.to_owned())),
            ]
        );
    }

    #[test]
    fn a_one_filesystem_walk_touches_only_its_roots_filesystem() {
        let mut index = Index::new_with_scope(
            "/srv",
            crate::ScanScope { one_filesystem: true, ..crate::ScanScope::default() },
        );
        index
            .apply(&Observation::new(vec![
                upsert("a.txt", EntryKind::File, 10, 1),
                upsert("mnt", EntryKind::Dir, 0, 2),
            ]))
            .expect("apply");
        let (entries, tallies) = devices(&index, Some(1), |_| true);
        assert_eq!(entries, [(1, PathBuf::new(), true)], "the mount point was a boundary");
        assert_eq!(tallies.keys().copied().collect::<Vec<_>>(), [1]);
    }

    #[test]
    fn one_filesystem_reached_from_two_roots_is_one_row_named_by_its_first_path() {
        let left = mounted();
        let mut right = Index::new("/data");
        right
            .apply(&Observation::new(vec![upsert("e.bin", EntryKind::File, 5, 2)]))
            .expect("apply");
        let parts = [(&left, 1), (&right, 2)]
            .into_iter()
            .enumerate()
            .map(|(root, (index, device))| {
                let (entries, tallies) = devices(index, Some(device), |_| true);
                (root, index.root_path(), entries, tallies)
            })
            .collect();
        let section = volumes_section(parts, None);
        let rows: Vec<_> = section
            .rows
            .iter()
            .map(|row| {
                (row.device, row.absolute.clone(), row.also_entered.len(), row.counted.bytes)
            })
            .collect();
        assert_eq!(
            rows,
            [(2, PathBuf::from("/data"), 1, 55), (1, PathBuf::from("/srv"), 0, 50),],
            "device 2 is entered at /data and /srv/mnt, and /data sorts first"
        );
        // A reader given no observation knows no capacity, and says why.
        assert!(section.rows.iter().all(
            |row| row.capacity.is_none() && row.unknown_reason.as_deref() == Some(NOT_OBSERVED)
        ));
        assert_eq!(section.observed_at_ns, None);
    }

    #[test]
    fn a_served_report_observes_the_filesystem_its_root_is_on() {
        use crate::query::{Query, Request, Section, ViewSpec};
        let root = tempfile::tempdir().expect("root");
        std::fs::write(root.path().join("f.bin"), vec![0_u8; 300]).expect("file");
        let (index, _) =
            crate::scan::scan_into_index(root.path(), &crate::ScanConfig::default()).expect("scan");
        let request = Request::new(
            crate::query::Basis::held_by(&index),
            Query { views: vec![ViewSpec::Volumes], ..Query::default() },
            std::time::SystemTime::now(),
        );
        let observed = observe_volumes(&[&index], &request).expect("the request names volumes");
        assert_eq!(observed.root_devices.len(), 1);
        assert!(observed.root_devices[0].is_some_and(|device| device != 0));
        assert_eq!(observed.root_devices[0], index.root_device(), "keyed as the walk recorded");
        let report =
            crate::query::report(&index, &request, std::time::SystemTime::now(), Some(&observed))
                .expect("report");
        let Section::Volumes(volumes) = &report.sections[0] else { panic!("volumes") };
        assert_eq!(volumes.rows.len(), 1, "a temporary directory is one filesystem");
        let row = &volumes.rows[0];
        // The index's root, which a walk canonicalizes: on macOS `/var` is `/private/var`.
        assert_eq!(row.absolute, index.root_path());
        assert!(row.capacity.as_ref().is_some_and(|capacity| capacity.capacity_bytes > 0));
        assert_eq!(row.counted.files, 1);
        assert!(volumes.observed_at_ns.is_some());
        // The row is named as every other row is, relative to its root, never by the
        // canonical path the capacity call used (`\\?\C:\…` on Windows).
        let text = crate::report_format::render(&report, crate::report_format::Format::Text, false)
            .expect("text");
        let first = text.lines().next().expect("a volume row");
        assert!(first.contains(" in use  .  ("), "{text}");
        assert!(!text.contains(r"\\?\"), "{text}");
        assert!(!text.contains(&*index.root_path().to_string_lossy()), "{text}");
        // The pure reader with no observation answers the same rows, capacity unknown.
        let pure = crate::query::report(&index, &request, std::time::SystemTime::now(), None)
            .expect("report");
        let Section::Volumes(unobserved) = &pure.sections[0] else { panic!("volumes") };
        assert_eq!(unobserved.rows.len(), 1);
        assert!(unobserved.rows[0].capacity.is_none());
        assert!(pure.notes.iter().any(|note| note.starts_with("note: capacity unknown")));
        // A request without the view observes nothing.
        let summary = Request::new(
            crate::query::Basis::held_by(&index),
            Query { views: vec![ViewSpec::Summary], ..Query::default() },
            std::time::SystemTime::now(),
        );
        assert!(observe_volumes(&[&index], &summary).is_none());
    }
}
