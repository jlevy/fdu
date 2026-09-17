---
type: is
id: is-01m2pye0rg6x3k2bzbqgvv6nrz
title: "P1.1.4: Add the unreadable-subtree mutation and its registry entries"
kind: task
status: in_progress
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: claude-code@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye70x3nc8p6t827sywb23
parent_id: is-01m2pmr9n3mq4nb2r1pc328qpz
hold: null
hold_until: null
created_at: 2026-09-17T05:46:33.103Z
updated_at: 2026-09-17T06:17:18.547Z
started_at: 2026-09-17T06:17:18.545Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 1: The Path-Independence Harness", commit 4. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `tests/path_independence/matrix.py`: add the `unreadable` mutation: `chmod 000` on `src/nested`, restored in `finally`, skipped where permission bits are not enforced (Windows, root).
- Registry entries for it under `unverified-subtree`, from a full Linux run.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
