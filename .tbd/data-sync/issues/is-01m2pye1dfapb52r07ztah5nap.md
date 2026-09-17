---
type: is
id: is-01m2pye1dfapb52r07ztah5nap
title: "P1.1.6: Retire the exploration scripts and update links"
kind: task
status: closed
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies: []
parent_id: is-01m2pmr9n3mq4nb2r1pc328qpz
created_at: 2026-09-17T05:46:33.775Z
updated_at: 2026-09-17T23:46:47.272Z
closed_at: 2026-09-17T23:46:47.272Z
close_reason: "Shipped in PR #79 and landed on main via stack merge 98379c76."
delegate: claude-code@spud10
hold: null
hold_until: null
started_at: 2026-09-17T06:17:19.747Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 1: The Path-Independence Harness", commit 6. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- Delete the scripts in `explorations/path-independence/`; keep `results/` and a README pointing to `tests/path_independence`.
- Update links in the plan, the design docs, and bead notes that cite the exploration scripts.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

## Notes

Layer 1 of the core-models stack: branch claude/core-models-1-harness, PR https://github.com/jlevy/fdu/pull/79 (stack #80 on #78). Committed in 6c1c9f67 (harness, registry, targets) and 3106c945 (CI). Close when the layer merges.
