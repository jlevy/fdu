---
type: is
id: is-01m2pyee0zf4meqknshjmzbz2k
title: "P2.3.7: accept_partial through Plan::outcome and OutcomeClass"
kind: task
status: closed
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyeeb53q0tcgnkayb0wv56
parent_id: is-01m2pmrbxvnerxyhnjhrswy1ye
created_at: 2026-09-17T05:46:46.686Z
updated_at: 2026-09-23T08:14:06.688Z
closed_at: 2026-09-23T08:14:06.688Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 3: The Execution Plan Model", commit 7. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `crates/fdu/src/cli.rs`: `allow_partial` (`:530`) becomes `Delivery.accept_partial`; `run` (`:742`) and `finish` (`:1737-1746`) take exit status from `Plan::outcome(&TreeStatus) -> OutcomeClass`.

**Tests**

- Use `OutcomeClass` in `crates/fdu/src/cli.rs:2931` and `crates/fdu/tests/cli_exit.rs`.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
