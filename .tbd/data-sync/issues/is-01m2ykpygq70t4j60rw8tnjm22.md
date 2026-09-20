---
type: is
id: is-01m2ykpygq70t4j60rw8tnjm22
title: Publish the directory-query plan as a separate stacked PR
kind: task
status: in_progress
priority: 1
version: 2
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
hold: null
hold_until: null
created_at: 2026-09-20T05:13:06.838Z
updated_at: 2026-09-20T05:14:11.059Z
started_at: 2026-09-20T05:14:11.058Z
---
Publish the accepted design as a self-contained tracked plan spec, with issue context, unchanged default output, list/tree/paths/long contracts, directory metric and exclusion semantics, compatibility, complete implementation bead breakdown, documentation/help inventory, and validation criteria. Use an isolated plan-only branch stacked on the latest branch (#94 above #92); preserve implementation edits in the original worktree. Link the spec from TODO.md and all related beads. Run documentation formatting and make check, review and commit only planning files, push/create the linked stack PR, and wait for CI. Keep all durable context in the PR, not temporary files.
