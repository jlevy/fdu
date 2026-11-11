---
type: is
id: is-01m75v690exesn86x6pv44ysx8
title: Give a device boundary that is no mount point its capacity (btrfs subvolume, macOS Data volume)
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-11T17:41:17.453Z
updated_at: 2026-11-11T17:41:17.453Z
---
From suggestion I7 of follow-up review I on PR #209 (https://github.com/jlevy/fdu/pull/209#issuecomment-6802796047). Suggestion; not for the 0.6.0 release. Sits beside fdu-vxji.

The volumes view asks a filesystem's capacity, below a report's root, only where the OS mount table lists a visible mount at exactly the path where the walk entered that filesystem, whose topmost filesystem is not autofs, and the stat that never mounts then finds the device the walk recorded (crates/fdu-core/src/query/query_volumes.rs, observe_entries_with; crates/fdu-core/src/capacity/mount_table.rs). Review I1 removed the rule that refused everything below an entry that failed, so a mount the table lists below a device boundary that is no mount point is now asked.

What is left: the row for such a boundary itself still reads `capacity unknown: not a mount point now, so it was not asked` (NOT_A_MOUNT_POINT), although asking it would mount nothing:
- Linux: a btrfs subvolume that is not mounted on its own (for example `home` in a btrfs root), and the same for bcachefs subvolumes. Reproduction: the btrfs case in review I1 (mkfs.btrfs on a loop image, `btrfs subvolume create root/home`, `fdu root --view volumes`): the `home` row reads unknown.
- macOS, by reading: the Data volume's own row wherever the walk did not list `/System/Volumes/Data`, as under `--scan-depth 3` or less from `/`, since each firmlink (`/Users`, `/Volumes`, `/Library`, `/private`) is a directory on the Data volume's device and none is a mount point.

Direction (from reviews H and I): check each component from the covering mount. A boundary whose covering mount (the nearest table-listed visible mount above it) is btrfs, bcachefs, or APFS lies inside one mounted filesystem that holds no automount points, so it is safe to ask once no component between the covering mount and the boundary has become a symlink (lstat each component from the covering mount down, with the stat that never mounts, and require each to be a directory on the covering mount's or the boundary's device). Done when: the btrfs subvolume row reads its capacity on a fresh walk and under --stale-ok, a root-only test in crates/fdu-core/tests/volumes_automount.rs (or a unit test on the seam) pins it, the macOS Data-volume row is checked by hand, and the design principle's "A Stat Never Mounts" section, docs/usage.md, and the CHANGELOG describe the rule.
