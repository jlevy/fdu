---
type: is
id: is-01m6yn405v7pxcye1mfah3yxhe
title: Report change-time fields and schema fdu.report/12
kind: task
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies:
  - type: blocks
    target: is-01m6yn41za441ay8y4vnmkd502
  - type: blocks
    target: is-01m6yn46vk6e86k68fbyx8e4yc
parent_id: is-01m6ycafymrea5sgq8r7n621re
created_at: 2026-11-08T22:40:30.395Z
updated_at: 2026-11-08T22:40:37.235Z
---
Rows and tree nodes that carry mtime_ns also carry changed_ns and changed_at (null where unknown or a lower bound); JSON, JSONL, YAML through the shared emitter; Python FileRow/TreeNode models decode them; REPORT_SCHEMA moves to fdu.report/12 once for this release (goldens, Python tests, docs/machine-output.md, skill).
