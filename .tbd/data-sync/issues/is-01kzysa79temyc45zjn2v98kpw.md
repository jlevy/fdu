---
type: is
id: is-01kzysa79temyc45zjn2v98kpw
title: Content sidecar load is the layer-3 warm cost on Linux
kind: task
status: in_progress
priority: 1
version: 15
delegate: unknown@spud10
labels:
  - campaign-2
  - macos-agenda
dependencies: []
hold: null
hold_until: null
created_at: 2026-08-14T00:03:55.833Z
updated_at: 2026-09-19T07:03:57.661Z
started_at: 2026-09-19T06:07:37.642Z
---
The content sidecar load costs about 370 ms for 14,542 files, roughly 25 microseconds per file, against about 3 microseconds per record for the metadata snapshot. It dominates every warm content run: with a sidecar hit, all three analysis profiles converge on the same warm floor regardless of how much analysis they avoided. Same class of problem as H78 for the metadata snapshot and probably wants the same answer, a layout usable without rebuilding per-record state. Measured in a virtualized-warm Linux regime; see research-2026-08-13-linux-three-tier-baseline.md.

## Notes

2026-09-19 exp-111 / H114 rejected. Apply-path type-id String alloc is not the
wall win. H83 stays open as H115 / fdu-wx15 / fdu-jxhk (one bottom-up pass).
Do not retry parse, candidates, install_controls, H113, or H114.
