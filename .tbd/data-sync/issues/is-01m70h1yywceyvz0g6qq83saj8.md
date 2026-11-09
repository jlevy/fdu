---
type: is
id: is-01m70h1yywceyvz0g6qq83saj8
title: "Engine: keep both clocks in Times and the index roll-ups"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m70h1y3e66qv3grdtmfcr72b
hold: null
hold_until: null
created_at: 2026-11-09T16:07:58.172Z
updated_at: 2026-11-09T18:34:04.404Z
started_at: 2026-11-09T16:09:06.769Z
closed_at: 2026-11-09T18:34:04.404Z
close_reason: Done on PR 209 (jlevy/fdu#209), reviewed in review E and fixed in 70b55219, f4e77d14, and 4d688217
resolution: null
duplicate_of: null
---
TimeBasis (Changed default, Modified) and Times {modified, changed} in engine_contract; Attrs::times, with an unobserved status change timed by modification. InternedRollUp's newest file and InternedPartitionRollUp's newest activity hold Times; merge, recompute_newest_upward, raise_activity_upward, the folded scan, the summary fold (execution.rs), and snapshot load maintain both. Public RollUp and RollUpSummary gain newest_changed_ns beside newest_mtime_ns.
