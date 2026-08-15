---
type: is
id: is-01kzy2fc4js5y0yew4q1vbjhdv
title: Share the index with the snapshot writer instead of deep-cloning it
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - perf
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-08-13T17:24:47.378Z
updated_at: 2026-08-15T05:04:14.712Z
closed_at: 2026-08-15T05:04:14.711Z
close_reason: "Confirmed as exp-059: open_with_pending_save returns Arc<Index>, the writer shares it instead of deep-cloning. cold-open-save wall -10.50% [-21.96%, -8.11%], peak RSS -35.26%. macOS validation rides with fdu-m4r6"
---
spawn_save clones the whole index so the write can overlap rendering; at 450k entries that is a second ~140-190 MiB tree of Box'd entries and BTreeMaps, and at the measured 1M scale it is the difference between ~400 MiB and ~800 MiB transient peaks whenever a save happens. The index is read-only from that point: an Arc<Index> (or serialize-before-render for the one-shot CLI path) gives the writer and the renderer two readers of one allocation. Touches PendingSave lifetime only; no delta/mutation semantics change. Found during PR #8 senior review.

## Notes

2026-08-15 review: the deep clone is on the render path of EVERY cache-writing run, not only the changed-warm path. Measured on the 450k Linux subject: --cache off 860ms vs --cache refresh 1341ms (+56%); spawn_save runs Arc::new(index.clone()) synchronously before the save thread spawns. See docs/project/research/research-2026-08-15-consumer-structural-headroom.md.
