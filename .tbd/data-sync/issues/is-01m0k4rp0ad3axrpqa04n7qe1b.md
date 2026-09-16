---
type: is
id: is-01m0k4rp0ad3axrpqa04n7qe1b
title: files becomes a complete name-ordered enumeration
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md
labels: []
dependencies: []
parent_id: is-01m0k4qrz1rb300efa1s5z86w6
created_at: 2026-08-21T21:48:52.616Z
updated_at: 2026-09-16T16:50:14.756Z
closed_at: 2026-09-16T16:50:14.755Z
close_reason: "a6b670c (PR #39): files is a complete name-ordered listing; ViewSpec::default_limit gives Files Bound::All (crates/fdu-core/src/query/query_report.rs)."
resolution: null
duplicate_of: null
---
`files` becomes a complete enumeration: name ascending, no limit. It is the fd/find
replacement, and the sync-watermark query depends on it -- a watermark run that silently
returns 20 of 192,871 changed files is a data-loss bug, not a display nit.

The name-ascending default only ever made sense for a complete listing: it exists so the
output diffs cleanly, and a diff of an arbitrary prefix is meaningless.
