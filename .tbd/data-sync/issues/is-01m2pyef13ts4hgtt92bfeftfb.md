---
type: is
id: is-01m2pyef13ts4hgtt92bfeftfb
title: "P2.4.2: open_for_report loads through load_serving; delete SnapshotUse and snapshot_scope_serves"
kind: task
status: in_progress
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyefbdg5fgka94j1bvwpbd
parent_id: is-01m2pmrcb8he4a8a54zt957vcs
hold: null
hold_until: null
created_at: 2026-09-17T05:46:47.714Z
updated_at: 2026-09-20T05:15:46.616Z
started_at: 2026-09-20T05:15:46.616Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 4: The `.gitignore` Observation Projection on Every Route", commit 2. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `lib.rs`: delete `snapshot_scope_serves` (`:377-395`), `SnapshotUse` (`:368-374`), and `RefusedSnapshot` (`:398-402`); the `open_for_report` load filter (`:501-531`) loads through `load_serving`; `unusable_snapshot_message` (`:415-454`) takes the refused identity; add `OpenReport.projected` (`:282-291`).
- If P2.3 has landed, `Plan::admit` calls `serves_snapshot`.

**Tests**

- The `lib.rs` tests at `:966` and `:1019` become projection tests.
- Extend `execution.rs:584` across policies and routes and flip `:630`.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
