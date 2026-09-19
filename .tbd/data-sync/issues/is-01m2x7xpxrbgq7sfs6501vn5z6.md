---
type: is
id: is-01m2x7xpxrbgq7sfs6501vn5z6
title: "H124: first-pass analyze I/O type/size gate or read-ahead"
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
created_at: 2026-09-19T16:27:51.095Z
updated_at: 2026-09-19T16:27:51.095Z
---
First-pass analyze wall is file I/O (read 59%, __open 17% on deciding-scale content-basic). Fewer opens via a type/size gate (do not open files that cannot contribute to the requested analyzer set) or read-ahead on admitted files cuts that wall at least 3%. Not H118 (apply shape). Not H119 (walk overlap / openat). Not a new unsafe block.

Metric: content-basic wall or product --analyze wall down at least 3% with the interval below zero on deciding-scale metabrowser. Digest identical. Worker parallelism retained.

What refutes: interval includes zero, or every admitted open is required for the requested metrics.

Why significant: H118/H119 showed apply and walk-overlap cannot move this job.

Protocol: docs/project/guides/performance-loop.md. Pickup: runbook Current Standing. Parent epic: fdu-8ya1.
