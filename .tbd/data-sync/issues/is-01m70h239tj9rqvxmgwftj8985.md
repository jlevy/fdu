---
type: is
id: is-01m70h239tj9rqvxmgwftj8985
title: "Tests: goldens, parity, path independence, engine tests on one time"
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m70h1y3e66qv3grdtmfcr72b
created_at: 2026-11-09T16:08:02.618Z
updated_at: 2026-11-09T18:34:08.525Z
closed_at: 2026-11-09T18:34:08.524Z
close_reason: Done on PR 209 (jlevy/fdu#209), reviewed in review E and fixed in 70b55219, f4e77d14, and 4d688217
resolution: null
duplicate_of: null
---
Goldens move to --since/--before, with --time modified where a fixture is backdated (touch -d cannot backdate a change time). Parity re-recorded. Path independence compares change-time requests by age residual and masks copy-dependent clocks, and adds --time modified requests for exact comparison. Engine tests for both clocks in roll-ups, fallback timing, sort time in trees.
