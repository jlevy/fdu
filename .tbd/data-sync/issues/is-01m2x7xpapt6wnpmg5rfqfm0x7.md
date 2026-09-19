---
type: is
id: is-01m2x7xpapt6wnpmg5rfqfm0x7
title: "H123: opened-root or refresh product path vs one-shot"
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
delegate: unknown@spud10
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m2x7xbt1c4we0wffeth7jrd6
hold: null
hold_until: null
created_at: 2026-09-19T16:27:50.485Z
updated_at: 2026-09-19T18:33:03.480Z
started_at: 2026-09-19T18:04:38.610Z
closed_at: 2026-09-19T18:33:03.479Z
close_reason: "Confirmed exp-119: product query::report 1.7ms versus default-tree 2078ms (~1222x) on system-private-frameworks. Probe index-second-report kept. No serving-policy change and no CLI flag. Not a snapshot load on fdu PATH."
resolution: null
duplicate_of: null
---
Follow-on to H117 (exp-116 probe only). A product opened-root, watch, or refresh path that retains the index is at least 3% faster than repeating one-shot fdu PATH for the same request. Not a license to load a snapshot on one-shot fdu PATH (H108 / H9).

Metric: product opened-root or refresh wall below one-shot fdu PATH / default-tree wall by at least 3% on system-private-frameworks or metabrowser-clone. One-shot footer stays cold scan.

What refutes: no product surface can retain and re-report without changing one-shot cache policy, or the product path misses 3%.

Why significant: H108 left the default CLI as a cold walk; H117 showed the engine already has the cheaper retained read.

Protocol: docs/project/guides/performance-loop.md. Pickup: runbook Current Standing. Parent epic: fdu-8ya1. H117 probe: fdu-7wiq (closed).

## Notes

2026-09-19 stacked #92: measuring product Index.report() path. New probe mode index-second-report times the second query::report on a retained detached Index (same API as Python Index.report and CLI watch Session.report). Compare to default-tree one-shot on system-private-frameworks. Not a snapshot load on fdu PATH. Experiment id exp-119.
