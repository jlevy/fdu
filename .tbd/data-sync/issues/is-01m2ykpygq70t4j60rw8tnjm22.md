---
type: is
id: is-01m2ykpygq70t4j60rw8tnjm22
title: Publish the directory-query plan as a separate stacked PR
kind: task
status: closed
priority: 1
version: 8
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
updated_at: 2026-09-20T05:37:57.838Z
started_at: 2026-09-20T05:14:11.058Z
closed_at: 2026-09-20T05:37:57.837Z
close_reason: "Plan-only PR #96 published above #94 with the complete design and bead breakdown; local make check and all 19 GitHub CI checks passed. Implementation remains a separate follow-up."
resolution: null
duplicate_of: null
---
Publish the accepted design as a self-contained tracked plan spec, with issue context, unchanged default output, list/tree/paths/long contracts, directory metric and exclusion semantics, compatibility, complete implementation bead breakdown, documentation/help inventory, and validation criteria. Use an isolated plan-only branch stacked on the latest branch (#94 above #92); preserve implementation edits in the original worktree. Link the spec from TODO.md and all related beads. Run documentation formatting and make check, review and commit only planning files, push/create the linked stack PR, and wait for CI. Keep all durable context in the PR, not temporary files.

## Notes

Published plan-only PR https://github.com/jlevy/fdu/pull/96, commit 16e1ded1, based on PR #94 at c234da2b. Stack #95: #91 -> #92 -> #94 -> #96. The PR contains only TODO.md and the complete tracked plan spec, including all six implementation steps. make check passed; documentation formatting and diff checks passed after the docs-only base refresh. All 19 CI checks passed on 16e1ded1: https://github.com/jlevy/fdu/actions/runs/35491851483. The plan is ready for review; runtime implementation, its six beads, and issue #93 remain open.
