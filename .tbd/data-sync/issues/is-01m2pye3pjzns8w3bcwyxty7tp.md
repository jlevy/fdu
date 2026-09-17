---
type: is
id: is-01m2pye3pjzns8w3bcwyxty7tp
title: "P1.3.1: Request model: Basis, Request, Delivery (no workers), RequestError, grammars, defaults, validate"
kind: task
status: in_progress
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: claude-code@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye4192h9z6a58v1t1vh2p
  - type: blocks
    target: is-01m2pye4c1h1kfcnsq59yfsnbt
parent_id: is-01m2pmr9ytx0ye8d701mr5vp9s
hold: null
hold_until: null
created_at: 2026-09-17T05:46:36.114Z
updated_at: 2026-09-17T15:27:19.155Z
started_at: 2026-09-17T15:27:19.154Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 3: The Request Model", commit 1. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `query/query_request.rs` (new), re-exported from `query.rs`: `Basis { root, scope: ScanConfig, content: AnalysisSet }`, `Delivery { cache, cache_path, accept_partial, watch: Option<WatchDelivery> }` (no `workers` until P2.3), `Request { basis, query: Query, now: SystemTime }`, `RequestSpec<'a>` (surface-neutral raw values), `RequestError` (`InvalidValue`, `ViewNeedsContent`, `IgnoredWithoutObservation`, `ContentMismatch`, `WatchScope`, `WatchContent`, `WatchCacheOnly`, `ViewLimit`), and `Request::{build, validate, validate_read, validate_delivery}` as in the plan. `validate_delivery` is declared here and filled in by P1.3.5.
- Value grammars moved in from both surfaces: `parse_kind` (`crates/fdu/src/cli.rs:1519`, `crates/fdu-py/src/lib.rs:1012`), `parse_bound` (`:1549`/`:1066`), `parse_sort` (`:1561`/`:1025`), `parse_size_metric` (`:1574`/`:1055`), `bound_nanos` (`:335`/`:709`), `parse_cache_policy` (`:1145`/`:606`).
- `build` resolves relative time windows against `now`, so `Selection.modified` is absolute; a watch builds its request once at session start.
- Defaults table: size allocated (wired in P1.3.2); report views `ViewSpec::default_for(content)`; watch views `tree`; `words_per_page` 250 (today declared at `crates/fdu/src/cli.rs:505`, `query/query_report.rs:380`, and the binding signatures); analysis none, `read_controls` on, `ControlLimits::default()`.

**Tests**

- The defaults table; every `RequestError` with flag and field wording; `now` resolution against a fixed instant; `ContentMismatch`.
- Move `query/query_report.rs:1951-1980` and `:2680-2707` into the module.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
