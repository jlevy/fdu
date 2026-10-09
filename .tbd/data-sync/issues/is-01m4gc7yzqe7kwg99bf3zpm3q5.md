---
type: is
id: is-01m4gc7yzqe7kwg99bf3zpm3q5
title: "PR #191 A2: two tests assume nanosecond SystemTime; Windows CI red"
kind: bug
status: closed
priority: 1
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4gc7na3mkr44xafhsfkk6sm
hold: null
hold_until: null
created_at: 2026-10-09T13:04:37.366Z
updated_at: 2026-10-09T13:56:23.437Z
started_at: 2026-10-09T13:04:47.622Z
closed_at: 2026-10-09T13:56:23.436Z
close_reason: "fixed in d4d5d0c0: request instants on 100 ns multiples (epoch; newest + 1 us)"
resolution: null
duplicate_of: null
---
High. query_report.rs:4311 and 6549-6550. Use instants at multiples of 100 ns. Review: https://github.com/jlevy/fdu/pull/191#issuecomment-6081386402
