//! The volumes view: each filesystem a report touches, its capacity beside the bytes the
//! report counted there.
//!
//! Rows come from the walk's own device changes, so no mount table is read: a root's
//! filesystem, and any entry whose device differs from its parent's, which a walk without
//! `--one-filesystem` crosses onto at a mount point. Capacity comes from an observation the
//! serving route made just before it read ([`crate::capacity`]), and a reader given none
//! reports it as unknown, never as zero.
//!
//! No row is summed with another. Available space can be shared between volumes of one
//! pool, and the bytes a report counted and the bytes in use never add to the capacity:
//! snapshots, other volumes, purgeable space, and deleted files still open all take space
//! no walk sees.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::EntryKind;
use crate::capacity::{CapacityObservation, VolumeCapacity};
use crate::index::{EntryId, Index};

/// What a serving route observed for a report that names `volumes`: the device each root
/// is on, and the capacity of every filesystem the report touches.
///
/// The roots' devices are observed because an index does not hold them: a walk records
/// every entry's attributes but the root's own. Like capacity, they are read when the
/// report is, never stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VolumesObservation {
    /// One device per root, in the roots' order; zero where the root could not be read.
    pub root_devices: Vec<u64>,
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
    /// How many other paths the report entered this filesystem at.
    pub other_paths: u64,
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

/// One pass over `index`: where it enters each filesystem, and what the regular files
/// `counted` admits hold on each.
///
/// Only a directory's device starts a filesystem, and every other entry is counted on its
/// directory's. A mount point is a directory, and a file's own device can name something
/// else: overlayfs, which containers run on, reports each file's underlying layer while
/// its directories report the overlay, so a file's device would split one filesystem into
/// a row per layer. A bind-mounted file is counted on its directory's filesystem for the
/// same reason. A directory with no device of its own (zero, which a Windows walker
/// reports for an entry it could list but not open) is taken to be on its parent's.
pub(super) fn devices(
    index: &Index,
    root_device: Option<u64>,
    counted: impl Fn(EntryId) -> bool,
) -> (Vec<(u64, PathBuf)>, BTreeMap<u64, VolumeCounted>) {
    let root_device = root_device
        .filter(|device| *device != 0)
        .or_else(|| {
            index.attrs_of(EntryId::ROOT).map(|attrs| attrs.dev).filter(|device| *device != 0)
        })
        .unwrap_or_else(|| inferred_root_device(index));
    // A one-filesystem walk retains each mount point it reached as a boundary and never
    // enters it, so the report touches no other filesystem.
    let crosses = !index.scope().one_filesystem;
    let mut entries = vec![(root_device, PathBuf::new())];
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
                    entries.push((own, child_path.clone()));
                }
                stack.push((child, child_path, own));
            }
        }
    }
    (entries, tallies)
}

/// The device most of the root's directories are on, for a reader given no observation of
/// the root's own: a walk does not record it, and a mount point is the exception among a
/// directory's children. Zero when the root has no directory with a device.
fn inferred_root_device(index: &Index) -> u64 {
    let mut seen: Vec<(u64, usize)> = Vec::new();
    for (_, child) in index.children_of(EntryId::ROOT).into_iter().flatten() {
        let (Some(EntryKind::Dir), Some(attrs)) = (index.kind_of(child), index.attrs_of(child))
        else {
            continue;
        };
        if attrs.dev == 0 {
            continue;
        }
        match seen.iter_mut().find(|(device, _)| *device == attrs.dev) {
            Some((_, count)) => *count += 1,
            None => seen.push((attrs.dev, 1)),
        }
    }
    // The most common, and among equals the first seen, so the answer is stable.
    seen.iter()
        .fold((0, 0), |best, &(device, count)| if count > best.1 { (device, count) } else { best })
        .0
}

/// `relative` under `root`, without the separator `join` adds for an empty path, so the
/// root's own row reads `/srv`, not `/srv/`.
fn absolute(root: &Path, relative: &Path) -> PathBuf {
    if relative.as_os_str().is_empty() { root.to_path_buf() } else { root.join(relative) }
}

/// Where a report over `indexes` enters each filesystem, in the roots' order: what a
/// serving route observes before it reads a report that names `volumes`.
///
/// `root_devices` names the device each root is on, in the roots' order, as
/// [`VolumesObservation::root_devices`] does; a root it leaves out is inferred.
pub fn volume_entries(indexes: &[&Index], root_devices: &[u64]) -> Vec<VolumeEntry> {
    indexes
        .iter()
        .enumerate()
        .flat_map(|(root, index)| {
            let (entries, _) = devices(index, root_devices.get(root).copied(), |_| false);
            entries.into_iter().map(move |(device, path)| VolumeEntry {
                root,
                device,
                absolute: absolute(index.root_path(), &path),
                path,
            })
        })
        .collect()
}

/// Observe the capacity every filesystem a report of `request` over `indexes` touches,
/// or nothing when the request names no `volumes` view.
///
/// The one step a serving route takes before it reads such a report, so that each route,
/// one-shot, retained, opened, or watching, observes the same filesystems the same way.
pub fn observe_volumes(
    indexes: &[&Index],
    request: &crate::query::Request,
) -> Option<VolumesObservation> {
    request.query.views.contains(&crate::query::ViewSpec::Volumes).then(|| {
        // Read as the walk reads a root's device for `--one-filesystem`, so the two agree.
        let root_devices: Vec<u64> = indexes
            .iter()
            .map(|index| {
                let root = index.root_path();
                std::fs::metadata(root)
                    .and_then(|meta| crate::scan::root_device(root, &meta))
                    .unwrap_or(0)
            })
            .collect();
        let entries = volume_entries(indexes, &root_devices);
        let mut ordered: Vec<&VolumeEntry> = entries.iter().collect();
        ordered.sort_by(|left, right| {
            left.absolute
                .as_os_str()
                .as_encoded_bytes()
                .cmp(right.absolute.as_os_str().as_encoded_bytes())
        });
        let capacity = crate::capacity::observe_capacity(
            ordered.into_iter().map(|entry| (entry.device, entry.absolute.as_path())),
        );
        VolumesObservation { root_devices, capacity }
    })
}

/// One root's part of a volumes section: its position, its path, and what [`devices`]
/// found under it.
pub(super) type RootDevices<'a> =
    (usize, &'a Path, Vec<(u64, PathBuf)>, BTreeMap<u64, VolumeCounted>);

/// Every path a report entered one device at, as (absolute, root, relative), with what it
/// counted there.
type Reached = (Vec<(PathBuf, usize, PathBuf)>, VolumeCounted);

/// The rows of a volumes section: every root's devices joined with `observation`.
pub(super) fn volumes_section(
    parts: Vec<RootDevices<'_>>,
    observation: Option<&VolumesObservation>,
) -> VolumesSection {
    let observation = observation.map(|seen| &seen.capacity);
    let mut by_device: BTreeMap<u64, Reached> = BTreeMap::new();
    for (root, root_path, entries, tallies) in parts {
        for (device, path) in entries {
            by_device.entry(device).or_default().0.push((absolute(root_path, &path), root, path));
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
            let other_paths = entries.len().saturating_sub(1) as u64;
            let (absolute, root, path) = entries.into_iter().next()?;
            let (capacity, unknown_reason) = match observation.map(|seen| seen.volume(device)) {
                None => (None, Some(NOT_OBSERVED.to_owned())),
                Some(None) => (None, Some("the filesystem was not observed".to_owned())),
                Some(Some(volume)) => match &volume.capacity {
                    Ok(capacity) => (Some(capacity.clone()), None),
                    Err(reason) => (None, Some(reason.clone())),
                },
            };
            Some(VolumeRow {
                device,
                root,
                path,
                absolute,
                other_paths,
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
        assert_eq!(entries, [(1, PathBuf::new()), (2, PathBuf::from("mnt"))]);
        // `mnt/sub` has no device of its own and stays on `mnt`'s; `layer/d.bin` reports
        // a layer's device and counts on its directory's.
        assert_eq!(tallies[&1], VolumeCounted { files: 2, bytes: 50, allocated: 50 });
        assert_eq!(tallies[&2], VolumeCounted { files: 2, bytes: 50, allocated: 50 });
        // The selection decides what is counted, not which filesystems are listed.
        let (entries, tallies) = devices(&index, Some(1), |_| false);
        assert_eq!(entries.len(), 2);
        assert!(tallies.is_empty());
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
        assert_eq!(entries, [(1, PathBuf::new())], "the mount point was a boundary");
        assert_eq!(tallies.keys().copied().collect::<Vec<_>>(), [1]);
    }

    #[test]
    fn an_unobserved_root_takes_the_device_most_of_its_directories_share() {
        let index = mounted();
        // `mnt` is on 2 and `layer` on 1: a tie, which the first-seen directory breaks.
        let first = index
            .children_of(EntryId::ROOT)
            .expect("root")
            .find_map(|(_, child)| {
                (index.kind_of(child) == Some(EntryKind::Dir)).then(|| index.attrs_of(child))
            })
            .flatten()
            .map(|attrs| attrs.dev);
        assert_eq!(Some(inferred_root_device(&index)), first);
        assert_eq!(inferred_root_device(&Index::new("/empty")), 0);
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
            .map(|row| (row.device, row.absolute.clone(), row.other_paths, row.counted.bytes))
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
        assert_ne!(observed.root_devices[0], 0);
        let report = crate::query::report_with_capacity(
            &index,
            &request,
            std::time::SystemTime::now(),
            Some(&observed),
        )
        .expect("report");
        let Section::Volumes(volumes) = &report.sections[0] else { panic!("volumes") };
        assert_eq!(volumes.rows.len(), 1, "a temporary directory is one filesystem");
        let row = &volumes.rows[0];
        assert_eq!(row.absolute, root.path());
        assert!(row.capacity.as_ref().is_some_and(|capacity| capacity.capacity_bytes > 0));
        assert_eq!(row.counted.files, 1);
        assert!(volumes.observed_at_ns.is_some());
        // The pure reader with no observation answers the same rows, capacity unknown.
        let pure =
            crate::query::report(&index, &request, std::time::SystemTime::now()).expect("report");
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
