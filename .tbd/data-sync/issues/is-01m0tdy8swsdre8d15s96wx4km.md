---
type: is
id: is-01m0tdy8swsdre8d15s96wx4km
title: Watch invalidation batches lose required dirty information
kind: bug
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels:
  - pr47-review
  - metabrowser
dependencies:
  - type: blocks
    target: is-01m0rw7bvxtw87tgde30emgs56
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T17:43:53.915Z
updated_at: 2026-08-24T17:44:16.249Z
---
At PR 47 head e658915, two paths lose invalidation information. Core dirty_rollups treats every Remove as a non-directory and omits the removed path itself, so a cached rollup for a deleted or renamed directory is never invalidated. The Python async adapter queues only tuple[Change, ...]; dirty_rollups is a side property on the worker-owned Watch, and the adapter drops an empty selected batch even when hidden changes dirtied aggregates. It also returns after setting stop without joining the worker. Fix: define one immutable WatchBatch returned by sync and async surfaces, carrying resulting cursor or version, changes, bounded dirty data, reset or all-dirty, state, and work. Include removed paths conservatively or retain old kind. Test filtered-out mutations, removed directories, async delivery, and joined cancellation. This supplies the lossless carrier that fdu-fltq can extend. Review finding FDU47-R5.
