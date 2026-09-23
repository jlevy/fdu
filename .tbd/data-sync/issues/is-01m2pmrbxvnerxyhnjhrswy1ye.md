---
type: is
id: is-01m2pmrbxvnerxyhnjhrswy1ye
title: "Phase 2 item 3: execution plan model: one planner and one write rule for every route"
kind: epic
status: in_progress
priority: 0
version: 19
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - design
  - cache
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
child_order_hints:
  - is-01m2pyec35d01hhcj2n349he3v
  - is-01m2pyecdev23a5knrxa2kmyh2
  - is-01m2pyecqwam3e928fc2jn2xjm
  - is-01m2pyed27kmqd4z20e9kqddh1
  - is-01m2pyedcc6gz25wqwmapyvp2h
  - is-01m2pyedpswpa393q9y6req1gz
  - is-01m2pyee0zf4meqknshjmzbz2k
  - is-01m2pyeeb53q0tcgnkayb0wv56
  - is-01m360rycaafcmq5rgkatcvmjm
created_at: 2026-09-17T02:57:26.458Z
updated_at: 2026-09-23T02:16:21.248Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 3: The Execution Plan Model", moved here when the item became beads (PR #78 at `e52383d4`; locators verified at `5f2d36d`). The commits are this bead's children, P2.3.1 to P2.3.8; their blockers carry the ordering, so this bead only groups them and closes when they do.

```rust
pub struct Delivery { pub cache: CachePolicy, pub cache_path: Option<PathBuf>, pub workers: Workers,
  pub accept_partial: bool, pub watch: Option<WatchDelivery> }
pub struct Workers { pub scan: Option<usize>, pub analysis: usize }
pub enum Route { OneShot, Retained, Refresh, Watch, Opened }
pub struct Plan { route: Route, retained: RetainedState, load: Load, verify: Verify, delivery: Delivery }
pub fn plan(request: &Request, delivery: &Delivery, route: Route) -> Result<Plan, RequestError>; // the root is request.basis.root
impl Plan {
  pub(crate) fn admit(&self, stored: &StoreHeader, basis: &Basis) -> Admission;
  pub(crate) fn writes(&self, run: &RunFacts) -> SaveTargets;   // the only write decision
  pub fn outcome(&self, status: &TreeStatus) -> OutcomeClass;
}
impl Delivery { pub fn enumerate() -> impl Iterator<Item = Delivery> }
```

Plans for the same request may differ only in what they load and in provenance.
`Delivery::enumerate()` yields representative values for the axes that can be enumerated
(cache policy, `accept_partial`, and watch), with worker counts and `cache_path` fixed.

| File | Function or type | Change |
| --- | --- | --- |
| `execution.rs` | `ReportPlan` (`:39`), `plan_report` (`:147-182`), `prepare_report_internal` (`:235-338`) | Replace with `Plan` and `plan(.., OneShot)`, keeping `RetainedState` (`:27`); execute the plan |
| `lib.rs` | `OpenConfig` (`:147-159`), `open` (`:337`), `open_with_pending_save` (`:359`), `open_for_report` (`:488-629`), `SaveTargets` and warm targets (`:579-586`, `:636-653`), `SNAPSHOT_MIN_ENTRIES` (`:687`), `cold_scan_save_targets` (`:693-717`), `load_content` (`:719`), `spawn_save` (`:730-777`) | Delete `OpenConfig` in favor of `Basis` and `Delivery`; `open(&Basis, &Delivery)`; `open_for_report` becomes `execute(&Plan, &Basis)`, with the cache-only content check moving into `admit`; `Plan::writes` replaces the save-target logic; `load_content` and `spawn_save` become pure executors |
| `lib.rs` | new `refresh(&mut Index, &Basis, &Delivery)` | `Route::Refresh`: reconcile, load the sidecar, analyze, and write per the plan |
| `watch_session.rs` | `Session` (`:113`) | Hold the plan; add `Session::start(request, delivery)` and `persist_due(now) -> SaveOutcome` |
| `crates/fdu/src/cli.rs` | `SaveOutcome` (`:297`), `save_is_due` (`:313`), `pending_after` (`:322`), `save_if_pending` (`:889`), `save_live` (`:931-948`), `run_watch` (`:760-879`), `allow_partial` (`:530`), `run` (`:742`), `finish` (`:1737-1746`) | Move throttling into `Session`; `accept_partial` in `Delivery`; exit status from `Plan::outcome` |
| `crates/fdu-py/src/lib.rs` | `refresh` (`:456-489`), `watch` (`:336`), `PyWatch.__next__` (`:1115`), `open` (`:1610`), `scan` (`:1685`), `report_once` (`:1319`) | Build a `Delivery`; `refresh` calls core `refresh`; `__next__` calls `persist_due`; the `Index.refresh` and `Index.watch` docstrings and the CHANGELOG say both write under `auto` |
| `opened.rs` | `OpenedIndex::open` | Take a plan with `Route::Opened` |
| `content/content_model.rs`, `content/content_analysis.rs`, `content/content_cache.rs` | `AnalysisRequest.workers` (`:193`), `analyze_index` (`:74`), `save_content_cache` (`:56`) | Workers move to `Delivery.workers` |

**Call sites:** `OpenConfig` literals (36 in `lib.rs` tests, 10 in `execution.rs`,
`cache.rs:632`, `:671`, `:758`, `opened.rs:2193`,
`crates/fdu-core/tests/watch_session_integration.rs:22`, five in
`examples/perf_probe.rs`, `crates/fdu-py/src/lib.rs:1346`, `:1625`, `:1698`,
`crates/fdu/src/cli.rs:635`); `open` calls in `lib.rs`, `execution.rs`, `cache.rs`,
`opened.rs:2191`, `examples/perf_probe.rs`, `watch_session_integration.rs:23`,
`crates/fdu-py/src/lib.rs:1638` and `:1710`, and `crates/fdu/src/cli.rs:778`;
`plan_report` in `execution.rs` tests (`:386-520`).

**Tests:** for every `Delivery::enumerate()` value and route, `writes` is identical for
the same run facts; content is written after a partial scan when verified and a snapshot
of its entry identity exists, entries never; Python `Index.refresh()` writes so a later
cache-only open succeeds; a Python watch persists under `auto`. Retarget the planner
tests, turn `save_tests` (`lib.rs:1623`) and `cold_scan_persistence_tests` (`:1730`)
into `Plan::writes` tests, move the throttle tests (`crates/fdu/src/cli.rs:2068`,
`:2089`, `:2103`) into `watch_session.rs`, and use `OutcomeClass` in
`crates/fdu/src/cli.rs:2931` and `crates/fdu/tests/cli_exit.rs`.

**Commits:**
1. `Plan` for the one-shot route with no behavior change.
2. `open_for_report` consumes the plan.
3. `Plan::writes` as the only write decision, with per-tier rules and their tests.
4. Delete `OpenConfig` and migrate callers.
5. Core `refresh`, used by Python.
6. `Session::start` and `persist_due`, used by the command line and Python.
7. `accept_partial` through `Plan::outcome`.
8. Opened roots take a plan.

**Risks:** `open_for_report` is the engine’s densest function, so commits 2 and 3 land
only with the harness subset passing; Python refresh and watch start writing snapshots
under `auto`; `persist_due` takes the clock as a parameter so tests stay deterministic.

## Original scope (before the implementation detail)

Given request, delivery, and available stored state: tiers needed, stored entries that serve (through the
stored-state model), what is verified, computed, written, and the resulting provenance. One-shot reports, open,
Python Index refresh and watch, CLI watch, and opened roots all use it. Alternative plans differ in cost, never
answer. Write rule: write only tiers verified completely, keyed by identity, never replacing another identity's
entry with narrower data.

## Notes

2026-09-17 (PR #78 review): Phase 2. Typed Delivery {cache, workers, accept_partial, watch} is this model's second input; format and colour belong to the answer model. The harness iterates deliveries through the type.
