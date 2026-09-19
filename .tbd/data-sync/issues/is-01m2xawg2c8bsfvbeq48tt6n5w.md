---
type: is
id: is-01m2xawg2c8bsfvbeq48tt6n5w
title: "PR #91 review R2: opened-second-report completeness and close"
kind: bug
status: closed
priority: 2
version: 3
delegate: unknown@spud10
labels: []
dependencies: []
parent_id: is-01m2xawb8wrb4e550za5ggdcw8
hold: null
hold_until: null
created_at: 2026-09-19T17:19:37.035Z
updated_at: 2026-09-19T17:26:55.178Z
started_at: 2026-09-19T17:19:41.239Z
closed_at: 2026-09-19T17:26:55.178Z
close_reason: "Addressed on PR #91 in e667b739: R1 apply timer isolated from decode; R2 completeness and close paths; S1–S3 applied cheaply."
resolution: null
duplicate_of: null
---
PR #91. Medium.

crates/fdu-core/examples/perf_probe.rs:1149-1194

Settle treats Ready|Watching|Stopped|Failed as done, then sets complete = true after close, including Failed/Stopped, oracle-off, and missing projection. Empty projection list returns without close.

Fix (option 1): refuse Failed and Stopped as a valid cell. Copy opened_discovery complete/coverage/error accounting. Close on every error path including empty projection list.
