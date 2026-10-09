---
type: is
id: is-01m4gc33xdg0re1zed4cjdtrmd
title: Keep a count of unlisted directories so the per-report activity pass can go
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T13:01:58.569Z
updated_at: 2026-10-09T13:01:58.569Z
---
Review A7 on #191: an index that tracks how many directories are not yet listed in full (and scan-depth boundaries) could serve completeness without query_subtrees::activity's per-report pass, removing one of the copies of the age definition. Partial and depth-bounded indexes then read the maintained newest_activity_ns too.
