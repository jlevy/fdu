---
type: is
id: is-01m2h2wtj22ccdbbnqhg2xdb05
title: "PR #58 review PR58-REC-6: evidence page says Prepared 2026-09-07 but contains a 2026-09-14 experiment"
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m2h2t7k65srszyhv02tag8ba
created_at: 2026-09-14T23:09:05.985Z
updated_at: 2026-09-14T23:27:41.282Z
closed_at: 2026-09-14T23:27:41.281Z
close_reason: "bd8ed0b: make perf-report PREPARED=2026-09-14; timeline.json and index.html now say Prepared 2026-09-14."
resolution: null
duplicate_of: null
---
PR #58, P3. `docs/project/reports/performance-evidence/timeline.json` has `"prepared": "2026-09-07"` and `index.html` prints "Prepared 2026-09-07", but the page contains exp-104, dated 2026-09-14.

The date is carried in the projection: `explorations/benchmarks/realtree/timeline.py:495` takes `--prepared` or preserves the committed value, and `make perf-report PREPARED=...` sets it (Makefile:620-625; `performance-loop.md` "Publishing the evidence"; runbook RECORD step).

Fix: `make perf-report PREPARED=2026-09-14`.
