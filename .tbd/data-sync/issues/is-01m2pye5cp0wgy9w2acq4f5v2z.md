---
type: is
id: is-01m2pye5cp0wgy9w2acq4f5v2z
title: "P1.3.6: Opened reads validate their read spec (documents without analysis is refused)"
kind: task
status: closed
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pmrcrmjrm62bm1x3mxwgvm
parent_id: is-01m2pmr9ytx0ye8d701mr5vp9s
created_at: 2026-09-17T05:46:37.845Z
updated_at: 2026-09-17T23:46:48.484Z
closed_at: 2026-09-17T23:46:48.484Z
close_reason: "Shipped in PR #83 and landed on main via stack merge 98379c76."
delegate: claude-code@spud10
hold: null
hold_until: null
started_at: 2026-09-17T16:27:31.857Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 3: The Request Model", commit 6. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `opened/read.rs`: `report_projection` (`:236`) and `validate_report` (`:281`) validate reads against `Basis { content: NONE }`, so `documents` is refused.

**Tests**

- Python: an opened `documents` read is refused.
- Regenerate `opened-root/coherent-projections-and-continuations.golden` if it changes, reading the diff.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
