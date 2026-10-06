---
type: is
id: is-01m480tgkyfd1rhkmktkzjcw3j
title: "PR #177 A7: perf probe duplicates a helper and silently re-baselines content-query"
kind: bug
status: closed
priority: 2
version: 3
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m480t2frm92ses02tfr36tgv
hold: null
hold_until: null
created_at: 2026-10-06T07:11:06.877Z
updated_at: 2026-10-06T09:04:28.704Z
started_at: 2026-10-06T07:40:02.353Z
closed_at: 2026-10-06T09:04:28.703Z
close_reason: "Fixed in 14497eee: helper deduplicated, regime change recorded in runbook; re-baseline tracked as fdu-mvnp"
resolution: null
duplicate_of: null
---
Low. crates/fdu-core/examples/perf_probe.rs:667-675; explorations/benchmarks/scenarios.json:966-1015. PR #177, review: https://github.com/jlevy/fdu/pull/177#issuecomment-6011241855
