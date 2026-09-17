---
type: is
id: is-01m2pye4c1h1kfcnsq59yfsnbt
title: "P1.3.3: report, report_in, and prepare_report* take &Request; delete the old validators"
kind: task
status: in_progress
priority: 0
version: 6
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: claude-code@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye4pgvg11q2txaafk5y0y
  - type: blocks
    target: is-01m2pye5cp0wgy9w2acq4f5v2z
  - type: blocks
    target: is-01m2pye5q72fsbb1n3y19mz0ma
  - type: blocks
    target: is-01m2pye8ptnr6019rcb15g4w69
parent_id: is-01m2pmr9ytx0ye8d701mr5vp9s
hold: null
hold_until: null
created_at: 2026-09-17T05:46:36.800Z
updated_at: 2026-09-17T16:27:30.487Z
started_at: 2026-09-17T16:27:30.484Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 3: The Request Model", commit 3. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `query/query_report.rs`: delete `Query::validate_analysis` (`:400`) and `validate_controls` (`:429`) into `Request`; extend `AxisNames` (`:298`); `report` (`:957`) and `report_in` (`:965`) take `&Request`, validate the read against the index's basis (`Index::content_set()` from P1.2.4), and read `analysis` and the share metric (`metric_summary`, `:1514-1522`) from the request.
- `execution.rs`: `prepare_report` (`:210`), `prepare_report_with_scan_diagnostics` (`:227`), `prepare_report_internal` (`:235`) take `(&Request, &Delivery)`, the root being in `Basis`; build today's `OpenConfig` from them internally until P2.3.4 deletes it; delete the check at `:242-244`.
- `engine_contract.rs`: `ReportRequest` (`:923-930`) becomes a read spec whose `now` is `generated_at`; add `Error::InvalidRequest(RequestError)`.
- Call sites: `query::report` at `execution.rs:324`, `watch_session.rs:155`, `crates/fdu-py/src/lib.rs:598`, `examples/perf_probe.rs:564`, and tests in `query/query_report.rs`, `report_format.rs`, and `content/content_analysis.rs`; `report_in` at `opened/read.rs:269`; `prepare_report*` at `crates/fdu/src/cli.rs:669` and `:671`, `crates/fdu-py/src/lib.rs:1382`, three calls in `examples/perf_probe.rs`, and 17 in `execution.rs` tests; `ReportRequest` at `opened.rs:3403`, `:5067`, `:5221`, `:5239`, `opened/golden_tests.rs:245`, `crates/fdu-py/src/opened_binding.rs:281`; the seven `validate_controls` and two `validate_analysis` sites.

**Tests**

- `execution.rs:713` expects `InvalidRequest(IgnoredWithoutObservation)`.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

Also advances fdu-vwkg's first half (the reader receives the requested analyzer set); its projection half is the content-subset deferral.
