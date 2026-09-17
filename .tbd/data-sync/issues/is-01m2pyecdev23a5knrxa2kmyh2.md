---
type: is
id: is-01m2pyecdev23a5knrxa2kmyh2
title: "P2.3.2: open_for_report executes the plan; the cache-only content check moves into Plan::admit"
kind: task
status: open
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyecqwam3e928fc2jn2xjm
parent_id: is-01m2pmrbxvnerxyhnjhrswy1ye
created_at: 2026-09-17T05:46:45.037Z
updated_at: 2026-09-17T05:46:59.277Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 3: The Execution Plan Model", commit 2. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `lib.rs`: `open_for_report` (`:488-629`) becomes `execute(&Plan, &Basis)`, with the cache-only content check moving into `Plan::admit(&StoreHeader, &Basis) -> Admission`.
- `execution.rs`: `prepare_report_internal` (`:235-338`) executes the plan.

**Done when**

- Lands only with the path-independence subset passing (`open_for_report` is the engine's densest function).
- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
