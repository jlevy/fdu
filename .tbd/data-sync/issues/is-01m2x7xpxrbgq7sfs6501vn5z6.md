---
type: is
id: is-01m2x7xpxrbgq7sfs6501vn5z6
title: "H124: first-pass analyze I/O type/size gate or read-ahead"
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
created_at: 2026-09-19T16:27:51.095Z
updated_at: 2026-09-19T18:50:43.650Z
started_at: 2026-09-19T18:35:08.825Z
closed_at: 2026-09-19T18:50:43.649Z
close_reason: "exp-121 rejected: every admitted open required for lines; skippable share under 1% wall; read calls already one data chunk per file; no engine change"
resolution: null
duplicate_of: null
---
First-pass analyze wall is file I/O (read 59%, __open 17% on deciding-scale content-basic). Fewer opens via a type/size gate (do not open files that cannot contribute to the requested analyzer set) or read-ahead on admitted files cuts that wall at least 3%. Not H118 (apply shape). Not H119 (walk overlap / openat). Not a new unsafe block.

Metric: content-basic wall or product --analyze wall down at least 3% with the interval below zero on deciding-scale metabrowser. Digest identical. Worker parallelism retained.

What refutes: interval includes zero, or every admitted open is required for the requested metrics.

Why significant: H118/H119 showed apply and walk-overlap cannot move this job.

Protocol: docs/project/guides/performance-loop.md. Pickup: runbook Current Standing. Parent epic: fdu-8ya1.

## Notes

Pre-registered 2026-09-19 ~11:37 PT.

H124 / exp-121. Control = #92 HEAD 45727e1d (H115+H120 in; no engine speed patch).
Job: first-pass `content-basic` wall on deciding-scale metabrowser (frozen APFS clone; live checkout has writers).
Accept: wall down at least 3% with the 95% interval entirely below zero; content digest identical; worker parallelism retained.
Not H118 (apply shape). Not H119 (walk overlap / openat). No new unsafe.

Named mechanism: type/size gate (do not open files that cannot contribute to lines) or read-ahead on admitted files.

What refutes: interval includes zero, or every admitted open is required for the requested metrics (and remaining files are already one-chunk sequential reads, so a safe read-ahead cannot clear 3%).

Honest first step: admission/size inventory + one counters/report pass. Implement a smallest fdu-core gate only if the inventory names skippable opens or bytes that can reach the 3% bar. Do not invent a speculative rewrite. Do not raise README 200K/4M.
