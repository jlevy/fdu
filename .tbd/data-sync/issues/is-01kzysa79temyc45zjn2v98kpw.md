---
type: is
id: is-01kzysa79temyc45zjn2v98kpw
title: Content sidecar load is the layer-3 warm cost on Linux
kind: task
status: in_progress
priority: 1
version: 20
labels:
  - campaign-2
  - macos-agenda
dependencies: []
child_order_hints:
  - is-01m2w9fjprg6vnzee5686g93j6
  - is-01m31gvrxj2qkm4c256dp7mss5
created_at: 2026-08-14T00:03:55.833Z
updated_at: 2026-09-21T08:21:17.276Z
delegate: unknown@spud10
hold: null
hold_until: null
started_at: 2026-09-19T06:07:37.642Z
---
The content sidecar load costs about 370 ms for 14,542 files, roughly 25 microseconds per file, against about 3 microseconds per record for the metadata snapshot. It dominates every warm content run: with a sidecar hit, all three analysis profiles converge on the same warm floor regardless of how much analysis they avoided. Same class of problem as H78 for the metadata snapshot and probably wants the same answer, a layout usable without rebuilding per-record state. Measured in a virtualized-warm Linux regime; see research-2026-08-13-linux-three-tier-baseline.md.

## Notes

2026-09-19 morning remaining queue: H121 (fdu-vf4b) re-profiles cache-hit restore after H115+H120 before any further apply/install cut. H83 waits on that profile. H113 quiet confirmatory (fdu-rfr6) still needs a quiet host; morning is the intended cell; do not run uncontrolled.

2026-09-19 H120 / exp-117 accepted. Streaming parse-into-apply kept. Peak RSS -10.13%. Wall non-inferior.

2026-09-19 H113 quiet confirmatory (fdu-rfr6) after H115: official PERF_HOST_REGIME=quiet start gate refused at 46.7% CPU busy. No pair ran. exp-113 unused. File-count shortcut not in the engine. H113 still needs a quiet host. Do not retry H113 uncontrolled.

2026-09-19 exp-112 / H115 accepted. Restore-only bottom-up rebuild kept (7798fdc1). Wall -9.69% [-26.02%, -7.13%]. Do not retry H115 or restart fdu-jxhk from that accept.

2026-09-19 exp-111 / H114 rejected. Apply-path type-id String alloc is not the wall win. Do not retry parse, install_controls, or H114.
