---
type: is
id: is-01m2x7xpapt6wnpmg5rfqfm0x7
title: "H123: opened-root or refresh product path vs one-shot"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m2x7xbt1c4we0wffeth7jrd6
created_at: 2026-09-19T16:27:50.485Z
updated_at: 2026-09-19T16:27:50.485Z
---
Follow-on to H117 (exp-116 probe only). A product opened-root, watch, or refresh path that retains the index is at least 3% faster than repeating one-shot fdu PATH for the same request. Not a license to load a snapshot on one-shot fdu PATH (H108 / H9).

Metric: product opened-root or refresh wall below one-shot fdu PATH / default-tree wall by at least 3% on system-private-frameworks or metabrowser-clone. One-shot footer stays cold scan.

What refutes: no product surface can retain and re-report without changing one-shot cache policy, or the product path misses 3%.

Why significant: H108 left the default CLI as a cold walk; H117 showed the engine already has the cheaper retained read.

Protocol: docs/project/guides/performance-loop.md. Pickup: runbook Current Standing. Parent epic: fdu-8ya1. H117 probe: fdu-7wiq (closed).
