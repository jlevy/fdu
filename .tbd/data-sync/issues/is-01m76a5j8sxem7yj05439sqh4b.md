---
type: is
id: is-01m76a5j8sxem7yj05439sqh4b
title: "volumes: bound capacity calls with a deadline"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-11T22:03:02.809Z
updated_at: 2026-11-11T22:03:02.809Z
---
Review J8 and K7 on #209. A hard NFS mount below the root whose server is gone passes the mount-table gate, and the confirming stat and statfs then block: --stale-ok --view volumes, a watch repaint, Index.report and OpenedIndex.read never return. Run each observation on a helper thread with a bound (a few seconds) and report 'capacity unknown: the filesystem did not answer within N s'. 0.6.0 documents the delay in the usage guide instead.
