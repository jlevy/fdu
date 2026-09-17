---
type: is
id: is-01m2pye1dfapb52r07ztah5nap
title: "P1.1.6: Retire the exploration scripts and update links"
kind: task
status: in_progress
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: claude-code@spud10
labels:
  - core-models
dependencies: []
parent_id: is-01m2pmr9n3mq4nb2r1pc328qpz
hold: null
hold_until: null
created_at: 2026-09-17T05:46:33.775Z
updated_at: 2026-09-17T06:17:19.750Z
started_at: 2026-09-17T06:17:19.747Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 1: The Path-Independence Harness", commit 6. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- Delete the scripts in `explorations/path-independence/`; keep `results/` and a README pointing to `tests/path_independence`.
- Update links in the plan, the design docs, and bead notes that cite the exploration scripts.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
