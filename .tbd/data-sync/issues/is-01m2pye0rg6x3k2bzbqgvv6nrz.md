---
type: is
id: is-01m2pye0rg6x3k2bzbqgvv6nrz
title: "P1.1.4: Add the unreadable-subtree mutation and its registry entries"
kind: task
status: closed
priority: 0
version: 5
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye70x3nc8p6t827sywb23
parent_id: is-01m2pmr9n3mq4nb2r1pc328qpz
created_at: 2026-09-17T05:46:33.103Z
updated_at: 2026-09-17T23:46:47.265Z
closed_at: 2026-09-17T23:46:47.265Z
close_reason: "Shipped in PR #79 and landed on main via stack merge 98379c76."
delegate: claude-code@spud10
hold: null
hold_until: null
started_at: 2026-09-17T06:17:18.545Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 1: The Path-Independence Harness", commit 4. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `tests/path_independence/matrix.py`: add the `unreadable` mutation: `chmod 000` on `src/nested`, restored in `finally`, skipped where permission bits are not enforced (Windows, root).
- Registry entries for it under `unverified-subtree`, from a full Linux run.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

## Notes

Layer 1 of the core-models stack: branch claude/core-models-1-harness, PR https://github.com/jlevy/fdu/pull/79 (stack #80 on #78). Committed in 6c1c9f67 (harness, registry, targets) and 3106c945 (CI). Close when the layer merges.
