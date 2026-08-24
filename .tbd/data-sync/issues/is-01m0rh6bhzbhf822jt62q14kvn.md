---
type: is
id: is-01m0rh6bhzbhf822jt62q14kvn
title: Report views still cannot tell a symlink-only directory from an empty one
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0racd5dxjfx1g5e0dsfay8q
created_at: 2026-08-24T00:02:15.743Z
updated_at: 2026-08-24T00:02:15.743Z
---
fdu-5hip added a non-file leaf count to roll-up state, so RollUp, RollUpScalars and a
listing row can now decide emptiness exactly. The report views cannot: SummaryRow and
TreeNode carry files, dirs and bytes, all three of which are zero for a directory holding
only symlinks, so `--view tree` renders it identically to an empty one.

That is the same bug at the CLI surface, and the surfaces are supposed to agree.

Not done in fdu-5hip on purpose: adding a column to the text table is a display decision
about the command line rather than an engine one, and it moves every text and JSON
golden. Worth doing deliberately, with the column's name and placement chosen rather than
inherited.

WHAT IT NEEDS: `others` on SummaryRow and TreeNode, carried through
summary_from_scalars and the tree builder, emitted in the JSON and YAML shapes, and a
decision about the text views -- a column, a suffix on the count, or nothing at all with
the fact reachable only in machine formats. Goldens follow whichever is chosen.
