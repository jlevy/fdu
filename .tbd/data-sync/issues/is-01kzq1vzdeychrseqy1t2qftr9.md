---
type: is
id: is-01kzq1vzdeychrseqy1t2qftr9
title: "Phase 3: Watch mode — Session API, --watch/--interval, fdu.stream/1"
kind: feature
status: closed
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzq1w4rnhr2z0eamhsy19h6m
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
child_order_hints:
  - is-01kzs4gd4ebz6b6r06zet3wmpc
  - is-01kzs66sekegfybkcjb5k3drmz
created_at: 2026-08-10T23:59:30.733Z
updated_at: 2026-08-11T21:20:37.565Z
closed_at: 2026-08-11T21:20:37.564Z
close_reason: "Phase 3 delivered: Session composing IndexHandle/Watcher/Query, the --watch loop with fdu.stream/1 records and dirty-gated repaint, scope validation for --scan-depth and --one-filesystem, Python Index.watch(), and watch-stream benchmark job registration. Persistence landed as a throttled save from both the batch and idle paths rather than a signal handler, pinned cold and warm by watch_persistence.rs and table-tested by fdu-w8af. Stream goldens closed as fdu-t9nv. The benchmark runner (fdu-g8ks) needs a harness-shape decision and moved to the epic."
---
Session composing IndexHandle + Watcher + Query with Selection-filtered batches; CLI watch loop (initial report, streamed files/recent records, dirty-gated aggregate re-render at --interval, explicit invalidate records, signal handling + final save); usage error for --watch with --scan-depth/--one-filesystem; Python Index.watch() iterator with deterministic shutdown tests; watch-stream benchmark job.
