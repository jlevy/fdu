---
type: is
id: is-01m76b043whs978xtx4vx3vne7
title: Saturate the older roll-up size additions in index.rs
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-11T22:17:33.052Z
updated_at: 2026-11-11T22:17:33.052Z
---
Review L4 on #209: the new volumes/ages tallies now saturate (fix/review-kl), but the tree roll-up's own additions (index.rs:631-635 and nearby) still use unchecked +=, so sizes a FUSE daemon chooses (st_size up to LLONG_MAX) can wrap in release builds and panic in debug builds. Use saturating_add as index.rs:952 already does; below 16 EiB nothing changes.
