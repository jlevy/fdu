---
type: is
id: is-01m2pyedcc6gz25wqwmapyvp2h
title: "P2.3.5: Core refresh(&mut Index, &Basis, &Delivery), used by Python Index.refresh"
kind: task
status: closed
priority: 0
version: 5
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyedpswpa393q9y6req1gz
parent_id: is-01m2pmrbxvnerxyhnjhrswy1ye
created_at: 2026-09-17T05:46:46.027Z
updated_at: 2026-09-23T08:14:06.674Z
closed_at: 2026-09-23T08:14:06.674Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
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

## Notes

CI115 follow-up in codex/alpha-execution-ci-fixes: native wheel smoke now explicitly proves Refresh+Only refusal leaves cached complete/stale/invalid_utf8 coverage unchanged, then verifies Auto refresh preserves expected coverage. Opened restricted-scope test now requires early typed OneFilesystem capability refusal on non-Unix, while Unix still requires the observation scope refusal. These preserve the Plan contract and platform refusal order rather than accepting any error. Full smoke passed against known fba4730c native artifact; fresh owning-worktree targeted opened test passed. Windows compile check underway; final composed wheel/CI pending.
