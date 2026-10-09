---
type: is
id: is-01m4gpwpyt8amak04a7we0xqkz
title: Measure a large Python report's cost from eager modified_at datetimes
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T16:10:43.033Z
updated_at: 2026-10-09T16:10:43.033Z
---
Review C7 fix 3 on #191 (exp-212, exp-213): Python's _models.py derives every row's modified_at datetime eagerly from mtime_ns. Time a 100k-row list report through fdu.report before and after the age fields, and if the derivation is a measurable share, make it lazy (a cached property) without changing the public type.
