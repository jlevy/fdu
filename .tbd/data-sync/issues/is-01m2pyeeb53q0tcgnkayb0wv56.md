---
type: is
id: is-01m2pyeeb53q0tcgnkayb0wv56
title: "P2.3.8: Opened roots take a plan (Route::Opened)"
kind: task
status: in_progress
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies: []
parent_id: is-01m2pmrbxvnerxyhnjhrswy1ye
created_at: 2026-09-17T05:46:47.012Z
updated_at: 2026-09-23T02:04:50.174Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 3: The Execution Plan Model", commit 8. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `opened.rs`: `OpenedIndex::open` takes a plan with `Route::Opened`.
- `crates/fdu-py/src/lib.rs`: `open` (`:1610`), `scan` (`:1685`), and `report_once` (`:1319`) build a `Delivery`.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
