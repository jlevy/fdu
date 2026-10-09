---
type: is
id: is-01m4fxar5gf3qrax2k0rwfczn0
title: "Age: text column, machine fields, fdu.report/11, Python models"
kind: task
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies:
  - type: blocks
    target: is-01m4fxay0x5wxrjarhc892gfhp
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-09T08:44:00.045Z
updated_at: 2026-10-09T09:45:35.988Z
started_at: 2026-10-09T09:45:35.987Z
---
Text: age cell between size and name, right-aligned to the section's widest, 2-space gutters; gray 'unknown' when incomplete, gray em dash when the row counts nothing, blank on remainder rows. Machine: tree nodes gain mtime_ns, complete, age_ns, modified_at; list rows gain modified_at; envelope gains age_reference_at; modified_at null whenever age is null for incompleteness; fdu.report/11 with the SchemaCheck test. Python models/stubs: modified_at as tz-aware UTC datetime derived from mtime_ns (floored to microseconds, correct before epoch).
