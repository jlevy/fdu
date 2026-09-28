---
type: is
id: is-01m3jvzx2gd1wvcnstvfmzw959
title: Split code overview into composable summary and language-line reports
kind: feature
status: in_progress
priority: 2
version: 4
delegate: codex@spud10.local
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-28T02:02:37.517Z
updated_at: 2026-09-28T02:08:21.092Z
started_at: 2026-09-28T02:03:07.793Z
---

## Notes

WIP checkpoint: separate core CodeSummary/CodeLines view types, shared projection, human headings, Python models and view smoke expectations, CLI help/docs/human goldens. Formatting checks clean; Python targeted Ruff passes. Not buildable or validated yet: machine serializer and old core tests intentionally await explicit schema /11 approval after automatic approval review rejected schema change. No commit/push/install of WIP. Existing installed0433bcc09 unchanged. Also pending preference on --analyze subreport aliases vs --view selection.
