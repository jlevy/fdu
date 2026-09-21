---
type: is
id: is-01kzysa79temyc45zjn2v98kpw
title: Content sidecar load is the layer-3 warm cost on Linux
kind: task
status: in_progress
priority: 1
version: 22
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
child_order_hints:
  - is-01m2w9fjprg6vnzee5686g93j6
  - is-01m31gvrxj2qkm4c256dp7mss5
created_at: 2026-08-14T00:03:55.833Z
updated_at: 2026-09-21T08:39:39.586Z
delegate: unknown@spud10
hold: null
hold_until: null
started_at: 2026-09-19T06:07:37.642Z
---
The content sidecar load costs about 370 ms for 14,542 files, roughly 25 microseconds per file, against about 3 microseconds per record for the metadata snapshot. It dominates every warm content run: with a sidecar hit, all three analysis profiles converge on the same warm floor regardless of how much analysis they avoided. Same class of problem as H78 for the metadata snapshot and probably wants the same answer, a layout usable without rebuilding per-record state. Measured in a virtualized-warm Linux regime; see research-2026-08-13-linux-three-tier-baseline.md.

## Notes

2026-09-21 Linux H149 / exp-155 (PR #105, c7251d26): after leftover apply-timer expansion, apply is 60-62% of restore on linux-v6.12 because HashMap remove + fingerprint now sit in apply. Same leftover identity as H144. No new compileable cut. Do not retry H116. Remaining H83 work is H78/H92 format, not another apply/install increment.

H144 (exp-144) already named the Linux cache-hit leftover under the old apply bucket. H121 apply-cut rule is not a compile on this mix.

Do not retry H115, H114, H116, H113 uncontrolled, or H85/mimalloc (fdu-cckr deferred).
