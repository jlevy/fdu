---
type: is
id: is-01m70qy0sfzmpbqxd9tk4vmx1f
title: Paired perf-compare of the both-clock roll-ups into the ledger
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-09T18:08:09.007Z
updated_at: 2026-11-09T18:08:09.007Z
---
Review E8 on PR 209. Phase 4 grows each directory's roll-up by 24 bytes (newest file and newest activity on both clocks) and adds a second maximum per partition to every merge. Spot measurements on a 261,876-file, 43,524-directory tree show no speed change and +1.2 to 1.7 MiB peak on warm and cache-only reads (43,524 x 24 B = 1.0 MiB), but no paired perf_probe run is recorded. Run make perf-compare (cold-scan-index with peak RSS, warm-snapshot-load) against a main control, then make perf-record, perf-ledger, and perf-report.
