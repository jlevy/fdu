---
type: is
id: is-01m2ykpygq70t4j60rw8tnjm22
title: Publish the directory-query plan as a separate stacked PR
kind: task
status: in_progress
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-09-20-directory-query-formats.md
delegate: claude-code@spud10
labels: []
dependencies:
  - type: blocks
    target: is-01m2y7by0xkr9zre4es53fwjm0
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
hold: null
hold_until: null
created_at: 2026-09-20T05:13:06.838Z
updated_at: 2026-09-20T05:32:36.053Z
started_at: 2026-09-20T05:14:11.058Z
---
Publish the accepted design as a self-contained tracked plan spec, with issue context, unchanged default output, list/tree/paths/long contracts, directory metric and exclusion semantics, compatibility, complete implementation bead breakdown, documentation/help inventory, and validation criteria. Use an isolated plan-only branch stacked on the latest branch (#94 above #92); preserve implementation edits in the original worktree. Link the spec from TODO.md and all related beads. Run documentation formatting and make check, review and commit only planning files, push/create the linked stack PR, and wait for CI. Keep all durable context in the PR, not temporary files.

## Notes

Published https://github.com/jlevy/fdu/pull/96 from codex/directory-query-plan, commit 16e1ded1, based on PR #94 at c234da2b. GitHub stack #95 is #91 -> #92 -> #94 -> #96. Only TODO.md and the full tracked plan spec are in the PR. make check passed (165 CLI golden checks, CLI/Python parity, 1615 path-independence cases); after the docs-only base refresh, docs-format, docs-format-check, and diff checks passed. CI is running; implementation remains separate.
