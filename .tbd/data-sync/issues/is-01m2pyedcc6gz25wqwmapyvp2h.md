---
type: is
id: is-01m2pyedcc6gz25wqwmapyvp2h
title: "P2.3.5: Core refresh(&mut Index, &Basis, &Delivery), used by Python Index.refresh"
kind: task
status: open
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyedpswpa393q9y6req1gz
parent_id: is-01m2pmrbxvnerxyhnjhrswy1ye
created_at: 2026-09-17T05:46:46.027Z
updated_at: 2026-09-17T05:47:00.152Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 3: The Execution Plan Model", commit 5. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `lib.rs`: new `refresh(&mut Index, &Basis, &Delivery)` for `Route::Refresh`: reconcile, load the sidecar, analyze, and write per the plan.
- `crates/fdu-py/src/lib.rs`: `refresh` (`:456-489`) calls core `refresh` with a built `Delivery`.
- The `Index.refresh` docstring and the CHANGELOG say it writes under `auto`.

**Tests**

- Python `Index.refresh()` writes, so a later cache-only open succeeds.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

Risk: Python refresh starts writing snapshots under `auto`.
