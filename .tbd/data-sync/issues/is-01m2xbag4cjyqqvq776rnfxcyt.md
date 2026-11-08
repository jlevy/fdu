---
type: is
id: is-01m2xbag4cjyqqvq776rnfxcyt
title: Add a volumes view over devices in the scan, not a duf clone
kind: feature
status: open
priority: 3
version: 10
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels:
  - design
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
child_order_hints:
  - is-01m6yn49kdb913226ab1c5td6n
  - is-01m6yn4ae3k3zew55hvb5p2nqb
  - is-01m6yn4b92ezxh72j5mbh9svsp
  - is-01m6yn4c4102x1n9x7pj3ambwp
  - is-01m6yn4cxyczxzkggvm3jfk0hp
  - is-01m6yn4dqpp7r5hqgmfs5ap59m
created_at: 2026-09-19T17:27:15.851Z
updated_at: 2026-11-08T22:40:44.278Z
---
Design review (2026-09-19): fdu should not grow a duf/df clone.

fdu's fact model is a hierarchical inventory of directory entries (walk, index, cache, watch, views). duf's fact model is the mount table plus kernel capacity (statfs/statvfs, O(mounts)). Those are different engines. The six-axis CLI and "views are readers of the index" rules have no honest place for a mount inventory.

What would be a distraction: enumerating mounts, local/network/fuse filters, inode mode, usage-bar theming, becoming `df`. That product already exists (duf), and this host already shows why it is a product: `df /` reports 12Gi used / 45% while `duf` reports 445.9G used / 96.8% on the same mount — APFS container vs volume accounting. Cloning that means inheriting those presentation lies.

What would be coherent later, if evidence shows agents miss it: a path-scoped capacity annotation on the report envelope (statvfs of the scanned root: mount, total, avail). That is a join, not a second view. It must be labeled as kernel capacity, not walked inventory, and must never imply tree_size + free = volume_size (snapshots, other volumes, purgeable space).

Agent holism belongs in the skill (compose df/duf for capacity, fdu for attribution), not in one binary absorbing a second fact model. fdu already refuses deletion; it is not a storage-management toolkit.

Do not implement unless a later decision promotes the thin annotation to a spec.

## Notes

Refinement (2026-09-19): the coherent shape is --view volumes over capacity pools that intersect the scan.

Index already retains attrs.dev. Tree bytes by pool are a real roll-up. Free space is planner-observed statvfs in provenance (report() stays pure). Re-observe at query time; do not persist avail. Group by shared pool, not st_dev (APFS container / bind mounts). Not --analyze. Not the default. Compose as --view tree,volumes.

Promoted (2026-11-08): the maintainer asked for a volume capacity view beside ordinary reports, as a convenience and not a df/duf replacement. Phase 3 of plan-2026-11-08-fdu-recent-growth-and-volumes.md specifies it: one row per filesystem holding a root or entered by the walk (by device, which merges bind mounts); capacity from statvfs / GetDiskFreeSpaceExW at report time, passed to the reader as an explicit input and a capacity provenance tier, never stored; in-use share is (capacity - available) / capacity, as duf shows; a pooled-filesystem note for APFS, btrfs, ZFS; no sums across rows; --scan-depth 0 is the fast capacity-only form. Joins --view full.
