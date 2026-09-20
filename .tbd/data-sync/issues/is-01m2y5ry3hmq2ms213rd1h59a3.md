---
type: is
id: is-01m2y5ry3hmq2ms213rd1h59a3
title: "PR #91 review R1: retain fractional restore timing"
kind: bug
status: in_progress
priority: 2
version: 4
delegate: unknown@spud10
labels: []
dependencies: []
parent_id: is-01m2y58e5s7yzpstkcqmheymy5
hold: null
hold_until: null
created_at: 2026-09-20T01:09:31.888Z
updated_at: 2026-09-20T01:17:05.344Z
started_at: 2026-09-20T01:10:13.050Z
closed_at: 2026-09-20T01:15:23.758Z
close_reason: Accumulate parse/apply nanoseconds locally and convert once via micros_from_nanos/add_nanos; unit test keeps 1000x900ns as 900us.
resolution: null
duplicate_of: null
---

## Notes

PR91 R1 review https://github.com/jlevy/fdu/pull/91#pullrequestreview-5258707801; content_cache.rs195/212 rounded each per-record duration. Implemented c2487f38; targeted tests pass, root reviewed, gate/push pending.
