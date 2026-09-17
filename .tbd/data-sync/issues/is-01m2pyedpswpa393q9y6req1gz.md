---
type: is
id: is-01m2pyedpswpa393q9y6req1gz
title: "P2.3.6: Session::start and persist_due, used by the command line and Python watch"
kind: task
status: open
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyee0zf4meqknshjmzbz2k
parent_id: is-01m2pmrbxvnerxyhnjhrswy1ye
created_at: 2026-09-17T05:46:46.360Z
updated_at: 2026-09-17T05:47:00.464Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 3: The Execution Plan Model", commit 6. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `watch_session.rs`: `Session` (`:113`) holds the plan; add `Session::start(request, delivery)` and `persist_due(now) -> SaveOutcome`, taking the clock as a parameter.
- `crates/fdu/src/cli.rs`: move throttling into `Session`: `SaveOutcome` (`:297`), `save_is_due` (`:313`), `pending_after` (`:322`), `save_if_pending` (`:889`), `save_live` (`:931-948`), `run_watch` (`:760-879`).
- `crates/fdu-py/src/lib.rs`: `watch` (`:336`) builds a `Delivery`; `PyWatch.__next__` (`:1115`) calls `persist_due`. The `Index.watch` docstring and the CHANGELOG say it writes under `auto`.

**Tests**

- Move the throttle tests (`crates/fdu/src/cli.rs:2068`, `:2089`, `:2103`) into `watch_session.rs`.
- A Python watch persists under `auto`.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
