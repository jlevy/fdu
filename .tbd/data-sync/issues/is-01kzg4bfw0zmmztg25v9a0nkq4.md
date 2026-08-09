---
type: is
id: is-01kzg4bfw0zmmztg25v9a0nkq4
title: "Watch hardening: rename stitching, backend selection, kqueue sweep, failed-watch marking"
kind: feature
status: open
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:28:15.231Z
updated_at: 2026-08-09T20:37:09.716Z
---
The watch layer works (coalesce -> verify by stat -> delta, with Flag::Rescan escalated rather than dropped). What is missing is everything platform-specific:

- inotify renames are cookie-paired and self-contained: apply them as a move with ZERO filesystem access instead of the current stat-both-sides path.
- Everywhere else, stitch renames by file identity — a FileId cache of (device, inode) on Unix, volume/file-index on Windows. This is notify-debouncer-full's proven design. Unstitched renames escalate as UnpairedRename.
- Backend selection policy: native for local filesystems, polling for NFS/FUSE/CIFS. That tuned policy ports down from metabrowser's watch_backends.py and is known to work in daily use.
- kqueue has NO overflow signal at all — dropped events are silent — so it needs a periodic reconciliation sweep emitting PeriodicSweep invalidations.
- Mark entries where watching failed rather than silently not watching (fsearch's MONITORED_FAILED). A watch that quietly did not install is indistinguishable from a quiet tree.
- kqueue watches per FILE (one fd each), which is a real limit at scale: bound it and degrade explicitly.

Currently every created directory escalates for a re-list, which is correct but heavier than needed on natively-recursive backends (FSEvents, ReadDirectoryChangesW). Scope the escalation to backends that actually have the watch-setup race.

## Notes

Final phase-0 review confirmed that phase 1 must bound both the raw notify channel and the verified-observation queue. Backpressure or drops must become a root invalidation, never silent loss. Permanent backend errors must keep affected scope non-fresh until watch installation recovers. Clock-stable re-verification and watcher/index root matching are complete under fdu-xktk. The 2026-08-09 Rust guideline audit adds one public failure ambiguity: next_observation currently maps both timeout and channel disconnection to None. A stopped worker must be distinguishable from a quiet tree, coordinated with the guard-free API work in fdu-s7wr.
