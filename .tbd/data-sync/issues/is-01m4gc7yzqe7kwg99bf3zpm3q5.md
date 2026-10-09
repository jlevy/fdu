---
type: is
id: is-01m4gc7yzqe7kwg99bf3zpm3q5
title: "PR #191 A2: two tests assume nanosecond SystemTime; Windows CI red"
kind: bug
status: in_progress
priority: 1
version: 2
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4gc7na3mkr44xafhsfkk6sm
hold: null
hold_until: null
created_at: 2026-10-09T13:04:37.366Z
updated_at: 2026-10-09T13:04:47.626Z
started_at: 2026-10-09T13:04:47.622Z
---
High. query_report.rs:4311 and 6549-6550. Use instants at multiples of 100 ns. Review: https://github.com/jlevy/fdu/pull/191#issuecomment-6081386402
