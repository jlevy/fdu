---
type: is
id: is-01m70h1ztph4z5jdq8q1rrq6d9
title: "Query: one window and a time basis in Selection"
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m70h1y3e66qv3grdtmfcr72b
created_at: 2026-11-09T16:07:59.062Z
updated_at: 2026-11-09T18:34:05.242Z
closed_at: 2026-11-09T18:34:05.241Z
close_reason: Done on PR 209 (jlevy/fdu#209), reviewed in review E and fixed in 70b55219, f4e77d14, and 4d688217
resolution: null
duplicate_of: null
---
Selection.window and Selection.time replace modified and changed; Candidate, SubtreeValues, ActivityTable, and maintained activity carry Times and read the basis. SortKey::Time replaces Mtime and Changed and is valid in every view (the tree refusal goes); recent, ages, tree, grouped rows, list rows, and the summary read the basis. Ages drops the unknown band; a note counts files timed by modification when a window or ages counted one. ReadSpec since/before/time, AxisNames, parse_time_basis, parse_sort time.
