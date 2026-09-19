---
type: is
id: is-01m2xawfmxqrz3kxtc446exw0y
title: "PR #91 review R1: isolate content_sidecar_apply_us from decode"
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
created_at: 2026-09-19T17:19:36.604Z
updated_at: 2026-09-19T17:26:55.165Z
started_at: 2026-09-19T17:19:41.227Z
closed_at: 2026-09-19T17:26:55.162Z
close_reason: "Addressed on PR #91 in e667b739: R1 apply timer isolated from decode; R2 completeness and close paths; S1–S3 applied cheaply."
resolution: null
duplicate_of: null
---
PR #91. Medium.

crates/fdu-core/src/content/content_cache.rs:185-228

After H120, apply_started wraps the decode/apply loop, so content_sidecar_apply_us includes per-record decode already counted in parse_us. Summing the four sidecar timers double-counts decode.

Fix: time apply as apply_restored_analysis + rebuild_content_rollups only. Keep decode on parse_us. Do not start H121.
