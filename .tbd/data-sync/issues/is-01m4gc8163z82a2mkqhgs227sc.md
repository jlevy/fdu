---
type: is
id: is-01m4gc8163z82a2mkqhgs227sc
title: "PR #191 A4: incomplete row with no entries shows a dash, not unknown"
kind: bug
status: in_progress
priority: 3
version: 2
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4gc7na3mkr44xafhsfkk6sm
hold: null
hold_until: null
created_at: 2026-10-09T13:04:39.617Z
updated_at: 2026-10-09T13:04:49.198Z
started_at: 2026-10-09T13:04:49.196Z
---
Nit. report_format.rs:2000-2008 tree_age_cell checks mtime_ns before completeness. Review: https://github.com/jlevy/fdu/pull/191#issuecomment-6081386402
