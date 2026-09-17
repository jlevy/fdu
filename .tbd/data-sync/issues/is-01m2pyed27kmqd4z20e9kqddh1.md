---
type: is
id: is-01m2pyed27kmqd4z20e9kqddh1
title: "P2.3.4: Delete OpenConfig for Basis and Delivery; worker counts move into Delivery.workers"
kind: task
status: open
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyedcc6gz25wqwmapyvp2h
parent_id: is-01m2pmrbxvnerxyhnjhrswy1ye
created_at: 2026-09-17T05:46:45.703Z
updated_at: 2026-09-17T05:46:59.856Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 3: The Execution Plan Model", commit 4. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `lib.rs`: delete `OpenConfig` (`:147-159`); `open` (`:337`) becomes `open(&Basis, &Delivery)`; `open_with_pending_save` (`:359`).
- `Delivery` gains `workers: Workers { scan: Option<usize>, analysis: usize }`; `ScanConfig`'s `threads`, `batch_size`, and `order` move into it, and `AnalysisRequest.workers` (`content/content_model.rs:193`), `analyze_index` (`content/content_analysis.rs:74`), and `save_content_cache` (`content/content_cache.rs:56`) read `Delivery.workers`. (The plan's table assigns this move to Phase 2 item 3 without naming a commit; it lands with the `OpenConfig` migration.)
- Call sites: `OpenConfig` literals (36 in `lib.rs` tests, 10 in `execution.rs`, `cache.rs:632`, `:671`, `:758`, `opened.rs:2193`, `crates/fdu-core/tests/watch_session_integration.rs:22`, five in `examples/perf_probe.rs`, `crates/fdu-py/src/lib.rs:1346`, `:1625`, `:1698`, `crates/fdu/src/cli.rs:635`); `open` calls in `lib.rs`, `execution.rs`, `cache.rs`, `opened.rs:2191`, `examples/perf_probe.rs`, `watch_session_integration.rs:23`, `crates/fdu-py/src/lib.rs:1638` and `:1710`, and `crates/fdu/src/cli.rs:778`.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
