---
type: is
id: is-01m70qy0sfzmpbqxd9tk4vmx1f
title: Paired perf-compare of the both-clock roll-ups into the ledger
kind: task
status: open
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-09T18:08:09.007Z
updated_at: 2026-11-11T22:03:05.706Z
---
Review E8 on PR 209. Phase 4 grows each directory's roll-up by 24 bytes (newest file and newest activity on both clocks) and adds a second maximum per partition to every merge. Spot measurements on a 261,876-file, 43,524-directory tree show no speed change and +1.2 to 1.7 MiB peak on warm and cache-only reads (43,524 x 24 B = 1.0 MiB), but no paired perf_probe run is recorded. Run make perf-compare (cold-scan-index with peak RSS, warm-snapshot-load) against a main control, then make perf-record, perf-ledger, and perf-report.

Also (review F5 on PR 209): give `--view ages` a benchmark job, as "Every Output Surface Is a Benchmark Job" asks. It needs a perf_probe mode and a component in explorations/benchmarks/index-suite.json that runs that mode (mapped in index-jobs.json); adding or reweighting a component is a new index version, so it needs new recorded evidence and could not go in the PR without it. Record F5's before and after with it. F5's spot measurement (release builds, `/ --one-filesystem`, 236,504 files and 40,928 directories, warm, median of 5, peak RSS from wait4, one uncontrolled virtualized Linux host, not a paired perf-loop run): unfiltered `--view ages` went from 0.50 s and 181.4 MiB peak to 0.26 s and 103.1 MiB once it banded from the index by entry id instead of building a FileRow and path per entry; `--view ages --kind file` (the walked route, 0.70 s / 243.8 MiB before, 0.68 s / 249.4 MiB after) and the default tree (0.20 s, 31.5 to 33.5 MiB) were unchanged.

Review K5 on #209 extends this: also a paired main-vs-head run of the walked route (--kind file --since 30d on both clocks; K measured +2.4% [+0.2%, +6.4%] wall and +3.0% RSS on an uncontrolled VM), a paired warm-snapshot-load run (K: +5.5% [-1.3%, +6.8%]), and a volumes benchmark job. Record each with its regime.
