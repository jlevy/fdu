//! The volumes view: each filesystem a report touches, its capacity beside the bytes the
//! report counted there.
//!
//! Rows come from the walk's own device changes, so no mount table is read: a root's
//! filesystem, and any entry whose device differs from its parent's, which a walk without
//! `--one-filesystem` crosses onto at a mount point. Capacity comes from an observation the
//! serving route made just before it read ([`crate::capacity`]), and a reader given none
//! reports it as unknown, never as zero.
//!
//! Capacity is asked only where the walk listed: of each root, which the user named and
//! whose listing mounts it in any case, and of each directory the walk listed. A capacity
//! call on an unmounted automount trigger mounts it and waits for the answer, as a listing
//! would, so a directory the walk kept without listing it (at the scan depth, or not yet
//! reached by an opened root's discovery) keeps its row with capacity unknown and the
//! reason ([`NOT_LISTED`]), never a call the walk itself would not have made.
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
    /// One device per root, in the roots' order; `None` where the root could not be read,
    /// which the reader then infers from the devices beneath it.
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
    /// A capacity call is made only on a listed entry.
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

/// Why a row's capacity is unknown when the walk listed none of the directories where the
/// report entered its filesystem, so no capacity call was made there.
pub(crate) const NOT_LISTED: &str = "the walk did not list this directory";

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
pub(super) fn devices(
    index: &Index,
    root_device: Option<u64>,
    counted: impl Fn(EntryId) -> bool,
) -> (Vec<Entered>, BTreeMap<u64, VolumeCounted>) {
    let root_device = root_device
        .filter(|device| *device != 0)
        .or_else(|| {
            index.attrs_of(EntryId::ROOT).map(|attrs| attrs.dev).filter(|device| *device != 0)
        })
        .unwrap_or_else(|| inferred_root_device(index));
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

/// The device each root is on, in the roots' order; `None` where a root cannot be read.
///
/// Read as the walk reads a root's device for `--one-filesystem`, so the two agree. This
/// reads the filesystem, and an opened root calls it before it takes its index guard.
pub(crate) fn root_devices<'a>(roots: impl IntoIterator<Item = &'a Path>) -> Vec<Option<u64>> {
    roots
        .into_iter()
        .map(|root| {
            std::fs::metadata(root).and_then(|meta| crate::scan::root_device(root, &meta)).ok()
        })
        .collect()
}

/// Observe the capacity of each filesystem `entries` enter, on the bytewise-first path the
/// walk listed for it; a filesystem entered only where the walk did not list is not asked.
pub(crate) fn observe_entries(
    root_devices: Vec<Option<u64>>,
    entries: &[VolumeEntry],
) -> VolumesObservation {
    observe_entries_with(root_devices, entries, |volumes| {
        crate::capacity::observe_capacity(volumes)
    })
}

/// [`observe_entries`], with the capacity calls made by `observe`: the seam a test uses to
/// see which paths would be asked.
fn observe_entries_with(
    root_devices: Vec<Option<u64>>,
    entries: &[VolumeEntry],
    observe: impl for<'p> FnOnce(Vec<(u64, &'p Path)>) -> CapacityObservation,
) -> VolumesObservation {
    let mut listed: Vec<&VolumeEntry> = entries.iter().filter(|entry| entry.listed).collect();
    listed.sort_by(|left, right| {
        left.absolute
            .as_os_str()
            .as_encoded_bytes()
            .cmp(right.absolute.as_os_str().as_encoded_bytes())
    });
    let capacity =
        observe(listed.into_iter().map(|entry| (entry.device, entry.absolute.as_path())).collect());
    VolumesObservation { root_devices, capacity }
}

/// Observe the capacity of every filesystem a report of `request` over `indexes` touches,
/// or nothing when the request names no `volumes` view.
///
/// The one step a route that owns its indexes takes before it reads such a report, so that
/// one-shot, retained, and watching routes observe the same filesystems the same way. An
/// opened root takes the same steps with its guard released around the calls.
pub fn observe_volumes(
    indexes: &[&Index],
    request: &crate::query::Request,
) -> Option<VolumesObservation> {
    request.query.views.contains(&crate::query::ViewSpec::Volumes).then(|| {
        let root_devices = root_devices(indexes.iter().map(|index| index.root_path()));
        let entries = volume_entries(indexes, &root_devices);
        observe_entries(root_devices, &entries)
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
            let other_paths = entries.len().saturating_sub(1) as u64;
            let listed = entries.iter().any(|entry| entry.3);
            let (absolute, root, path, _) = entries.into_iter().next()?;
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
                None => (None, Some("the filesystem was not observed".to_owned())),
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
    /// root is on device 1, and the observation it would make, with a stand-in for the
    /// calls that answers every path it is given.
    fn asked_of(index: &Index) -> (Vec<(u64, PathBuf)>, VolumesObservation) {
        let entries = volume_entries(&[index], &[Some(1)]);
        let mut asked = Vec::new();
        let observation = observe_entries_with(vec![Some(1)], &entries, |volumes| {
            asked = volumes.iter().map(|(device, path)| (*device, path.to_path_buf())).collect();
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
        });
        (asked, observation)
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
        let report = crate::query::report_with_capacity(
            index,
            &request,
            std::time::UNIX_EPOCH,
            Some(observation),
        )
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
        assert!(observed.root_devices[0].is_some_and(|device| device != 0));
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
