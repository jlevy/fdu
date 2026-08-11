---
type: is
id: is-01kzq1vzdeychrseqy1t2qftr9
title: "Phase 3: Watch mode — Session API, --watch/--interval, fdu.stream/1"
kind: feature
status: open
priority: 2
version: 4
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
updated_at: 2026-08-11T19:53:48.242Z
---
Session composing IndexHandle + Watcher + Query with Selection-filtered batches; CLI watch loop (initial report, streamed files/recent records, dirty-gated aggregate re-render at --interval, explicit invalidate records, signal handling + final save); usage error for --watch with --scan-depth/--one-filesystem; Python Index.watch() iterator with deterministic shutdown tests; watch-stream benchmark job.
