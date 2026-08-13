---
type: is
id: is-01kzy2fc4js5y0yew4q1vbjhdv
title: Share the index with the snapshot writer instead of deep-cloning it
kind: task
status: open
priority: 2
version: 1
labels:
  - perf
dependencies: []
created_at: 2026-08-13T17:24:47.378Z
updated_at: 2026-08-13T17:24:47.378Z
---
spawn_save clones the whole index so the write can overlap rendering; at 450k entries that is a second ~140-190 MiB tree of Box'd entries and BTreeMaps, and at the measured 1M scale it is the difference between ~400 MiB and ~800 MiB transient peaks whenever a save happens. The index is read-only from that point: an Arc<Index> (or serialize-before-render for the one-shot CLI path) gives the writer and the renderer two readers of one allocation. Touches PendingSave lifetime only; no delta/mutation semantics change. Found during PR #8 senior review.
