---
type: is
id: is-01m70h20ncstrkzcc17cbcrx75
title: "Report schema: name every time by the request's basis"
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m70h1y3e66qv3grdtmfcr72b
created_at: 2026-11-09T16:07:59.916Z
updated_at: 2026-11-09T18:34:06.054Z
closed_at: 2026-11-09T18:34:06.054Z
close_reason: Done on PR 209 (jlevy/fdu#209), reviewed in review E and fixed in 70b55219, f4e77d14, and 4d688217
resolution: null
duplicate_of: null
---
Rows and nodes give time_ns, time_at, age_ns; newest_file_time_ns on summary and tree rows; list rows keep raw mtime_ns and changed_ns; the request echo gains time; the ages section drops basis and the unknown band; Report gains time. Schema stays fdu.report/12, unreleased.
