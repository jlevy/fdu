---
type: is
id: is-01m2pyec35d01hhcj2n349he3v
title: "P2.3.1: Plan for the one-shot route, with no behavior change"
kind: task
status: closed
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyecdev23a5knrxa2kmyh2
parent_id: is-01m2pmrbxvnerxyhnjhrswy1ye
created_at: 2026-09-17T05:46:44.709Z
updated_at: 2026-09-23T08:14:06.648Z
closed_at: 2026-09-23T08:14:06.648Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 3: The Execution Plan Model", commit 1. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `execution.rs`: replace `ReportPlan` (`:39`) and `plan_report` (`:147-182`) with `Plan` and `plan(request, delivery, Route::OneShot)`, keeping `RetainedState` (`:27`).
- Types: `Route { OneShot, Retained, Refresh, Watch, Opened }`; `Plan { route, retained, load, verify, delivery }`; `plan(&Request, &Delivery, Route) -> Result<Plan, RequestError>`; `Plan::{admit, writes, outcome}` signatures; `Delivery::enumerate()` yielding representative values for cache policy, `accept_partial`, and watch, with worker counts and `cache_path` fixed.
- Plans for the same request may differ only in what they load and in provenance.

**Tests**

- Retarget the planner tests (`execution.rs:386-520`).

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
