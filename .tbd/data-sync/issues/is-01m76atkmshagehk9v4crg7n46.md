---
type: is
id: is-01m76atkmshagehk9v4crg7n46
title: "volumes: hold a descriptor from the mount-table check through the capacity call (Linux)"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-11T22:14:32.345Z
updated_at: 2026-11-11T22:14:32.345Z
---
Review L1 on #209. observe_entries reads /proc/self/mountinfo once, checks every listed entry by path (statx AT_NO_AUTOMOUNT, AT_STATX_SYNC_AS_STAT), then makes every statfs by path, serially. The window between the table read and an entry's check, and between its check and its statfs, is as long as the slowest earlier call (an NFS GETATTR or FSSTAT), not milliseconds. An autofs key or kernel submount that expires in that window is mounted again: statfs resolves with LOOKUP_AUTOMOUNT, and a lookup of an expired indirect key asks the automounter. Fix: right after the table read, open each passing entry with O_PATH|O_NOFOLLOW|O_CLOEXEC (never O_DIRECTORY, per review F1), via openat2 RESOLVE_NO_SYMLINKS where available; confirm on the descriptor with statx(fd, '', AT_EMPTY_PATH|AT_STATX_DONT_SYNC, STATX_TYPE|STATX_MNT_ID) against the table's topmost mount id and the recorded device; call fstatfs(fd) and hold the fd until it returns (an open descriptor keeps the mount from expiring). Bound open descriptors. Add a root-only volumes_automount test that expires a mounted key between check and call through the observe seam. Keep macOS path-based until measured. Changes the public capacity::observe_capacity seam. 0.6.0 states the residual window in the design principle and user docs instead.
