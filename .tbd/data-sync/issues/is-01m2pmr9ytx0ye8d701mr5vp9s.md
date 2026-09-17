---
type: is
id: is-01m2pmr9ytx0ye8d701mr5vp9s
title: "Phase 1 item 3: request model with one defaults table, grammars, and validation"
kind: epic
status: open
priority: 0
version: 19
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - design
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
child_order_hints:
  - is-01m2pye3pjzns8w3bcwyxty7tp
  - is-01m2pye4192h9z6a58v1t1vh2p
  - is-01m2pye4c1h1kfcnsq59yfsnbt
  - is-01m2pye4pgvg11q2txaafk5y0y
  - is-01m2pye522p05a7rd9aj4390tk
  - is-01m2pye5cp0wgy9w2acq4f5v2z
created_at: 2026-09-17T02:57:24.441Z
updated_at: 2026-09-17T05:47:43.579Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 3: The Request Model", moved here when the item became beads (PR #78 at `e52383d4`; locators verified at `5f2d36d`). The commits are this bead's children, P1.3.1 to P1.3.6; their blockers carry the ordering, so this bead only groups them and closes when they do.

The request model lives in `query/query_request.rs`, re-exported from `query.rs`:

```rust
pub struct Basis { pub root: PathBuf, pub scope: ScanConfig, pub content: AnalysisSet }
pub struct Delivery { pub cache: CachePolicy, pub cache_path: Option<PathBuf>, pub accept_partial: bool, pub watch: Option<WatchDelivery> }
pub struct Request { pub basis: Basis, pub query: Query, pub now: SystemTime }
pub struct RequestSpec<'a> { /* surface-neutral raw values, Option<&'a str> per axis */ }
pub enum RequestError { InvalidValue { axis: &'static str, value: String, expected: String },
  ViewNeedsContent(ViewSpec), IgnoredWithoutObservation(IgnoredEntries),
  ContentMismatch { held: AnalysisSet, requested: AnalysisSet },
  WatchScope, WatchContent, WatchCacheOnly, ViewLimit { attempted: usize, limit: usize } }
impl Request {
  pub fn build(spec: &RequestSpec, now: SystemTime, axes: AxisNames) -> Result<Self, RequestError>;
  pub fn validate(&self) -> Result<(), RequestError>;
  pub fn validate_read(&self, held: &Basis) -> Result<(), RequestError>;
  pub fn validate_delivery(&self, delivery: &Delivery) -> Result<(), RequestError>;
}
```

`build` resolves relative time windows against `now`, so `Selection.modified` is
absolute; a watch builds its request once at session start.
Typed command-line values (for example `--scan-depth`) reach `RequestSpec` through their
`Display`, so a mismatch there shows up as a golden difference.

| Axis | Default | Where it differs today |
| --- | --- | --- |
| Size | Allocated | `SizeMetric` defaults to apparent (`query/query_selection.rs:18-19`); the opened-root binding uses `"apparent"` (`crates/fdu-py/src/opened_binding.rs:121`) |
| Views (report) | `ViewSpec::default_for(content)` | None |
| Views (watch) | `tree`, the default for no content | Python passes `Files` (`crates/fdu-py/src/lib.rs:357`); `WatchOptions` defaults to `files` (`crates/fdu-py/python/fdu/_models.py:430`) |
| `words_per_page` | 250 | Declared at `crates/fdu/src/cli.rs:505`, `query/query_report.rs:380`, and the binding signatures |
| Analysis and controls | None; `read_controls` on; `ControlLimits::default()` | None |

| File | Function or type | Change |
| --- | --- | --- |
| `query/query_request.rs` | new | The types above, and the value grammars moved in from both surfaces: `parse_kind` (`cli.rs:1519`, `crates/fdu-py/src/lib.rs:1012`), `parse_bound` (`:1549`/`:1066`), `parse_sort` (`:1561`/`:1025`), `parse_size_metric` (`:1574`/`:1055`), `bound_nanos` (`:335`/`:709`), `parse_cache_policy` (`:1145`/`:606`) |
| `query/query_report.rs` | `Query::validate_analysis` (`:400`), `validate_controls` (`:429`), `AxisNames` (`:298`), `report` (`:957`), `report_in` (`:965`), share metric in `metric_summary` (`:1514-1522`) | Delete both validators into `Request`; extend `AxisNames`; `report` and `report_in` take `&Request`, validate the read against the index’s basis, and read `analysis` and the share metric from the request |
| `query/query_selection.rs` | `SizeMetric` default (`:18-19`) | Allocated |
| `execution.rs` | `prepare_report` (`:210`), `prepare_report_with_scan_diagnostics` (`:227`), `prepare_report_internal` (`:235`) | Take `(&Request, &Delivery)`, the root being in `Basis`; build today’s `OpenConfig` from them internally until Phase 2 item 3 deletes it; delete the check at `:242-244` |
| `watch_session.rs` | `Session::new` (`:127-142`), `query()` (`:145`) | `new(handle, Request, WatchConfig)`; refuse when `content_set()` is not empty and run the delivery checks; `query()` becomes `request()` |
| `scan.rs` | `validate_for_watch_scope` (`:400-406`) | Keep scope equality for its callers; the depth and one-filesystem rule becomes `RequestError::WatchScope` |
| `engine_contract.rs` | `ReportRequest` (`:923-930`), `Error` | A read spec whose `now` is `generated_at`; add `Error::InvalidRequest(RequestError)` |
| `opened/read.rs` | `report_projection` (`:236`), `validate_report` (`:281`) | Validate reads against `Basis { content: NONE }`, so `documents` is refused |
| `crates/fdu/src/cli.rs` | `run` (`:612-651`), `scan_config` (`:1109`), `parse_query` (`:1186`), `parse_analysis` (`:1245`), `resolved_query` (`:1180`), `resolve_views` (`:1488`), `run_watch` (`:760`) | `Cli::spec()` and `Request::build(.., SystemTime::now(), AxisNames::FLAGS)`; delete `:627-634` and the watch guards `:638-651`; add the `--watch --cache only` refusal |
| `crates/fdu-py/src/lib.rs` | `PyIndex` (`:134-147`), `build_query_at` (`:1172-1246`), `build_report` (`:551`), `watch` (`:336`), `report_once` (`:1319`), `open` (`:1610`), `scan` (`:1685`), `to_py_err` (`:38`) | Hold `basis`; `build_request(now, basis, spec)`; delete validation at `:375`, `:590`, `:1380`, `:1244`; `watch` refuses an analyzed or cache-only index; map `InvalidRequest` to `ValueError` |
| `crates/fdu-py/src/opened_binding.rs`, `crates/fdu-py/python/fdu/_models.py` | `parse_selection` (`:98-145`), `parse_report` (`:260-286`); `WatchOptions.query` (`:430`) | Defaults from the model; `WatchOptions` defaults to `Query()` |

**Call sites:** `query::report` at `execution.rs:324`, `watch_session.rs:155`,
`crates/fdu-py/src/lib.rs:598`, `examples/perf_probe.rs:564`, and tests in
`query/query_report.rs`, `report_format.rs`, and `content/content_analysis.rs`;
`report_in` at `opened/read.rs:269`; `Session::new` at `crates/fdu/src/cli.rs:798`,
`crates/fdu-py/src/lib.rs:380`, `crates/fdu-core/tests/watch_session_integration.rs:24`;
`prepare_report*` at `crates/fdu/src/cli.rs:669` and `:671`,
`crates/fdu-py/src/lib.rs:1382`, three calls in `examples/perf_probe.rs`, and 17 in
`execution.rs` tests; `ReportRequest` at `opened.rs:3403`, `:5067`, `:5221`, `:5239`,
`opened/golden_tests.rs:245`, `crates/fdu-py/src/opened_binding.rs:281`; the seven
`validate_controls` and two `validate_analysis` sites.

**Tests:** in `query/query_request.rs`, the defaults table, every `RequestError` with
flag and field wording, `now` resolution against a fixed instant, the watch refusals,
and `ContentMismatch`; in `watch_session_integration.rs`, refusal of an analyzed index;
move `query/query_report.rs:1951-1980` and `:2680-2707` into the module; repoint the
grammar tests in `crates/fdu/src/cli.rs`; `execution.rs:713` expects
`InvalidRequest(IgnoredWithoutObservation)`. Goldens: add `fdu --watch --cache only .`
near `cli-surface.tryscript.md:608-620`; regenerate
`opened-root/coherent-projections-and-continuations.golden`. Python: assert
`WatchOptions().query == Query()`, and add refusals for `Index.watch()` and an opened
`documents` read.

**Commits:**
1. The model, including the `Delivery` struct without `workers`, grammars, defaults, and
   `validate`, with unit tests.
2. Allocated as the size default; regenerate the opened-root golden.
3. `report`, `report_in`, and `prepare_report*` take `&Request`; migrate every caller
   and delete the old validators.
4. The command line and the binding build through `RequestSpec`.
5. `validate_delivery`: the watch refusals and the tree default.
6. Opened reads validate their read spec.

**Risks:** the allocated default reaches opened-root selection through `EntrySelection`,
which MetaBrowser sees; `RequestError` must reproduce each surface’s current wording,
which goldens and the parity label class check; `ScanConfig` still holds delivery fields
(`threads`, `batch_size`, `order`), and `ScanConfig.threads` stays the authoritative
scan worker count until Phase 2 item 3 moves those fields into `Delivery.workers`.

## Original scope (before the implementation detail)

Root, scope, content (analyzer set and options), selection, views and view options; one defaults table; value
grammars; typed validation (analysis-requiring views, ignored-state selection without observation, watch
compatibility, limits); derived choices (default view, views `full` omits); scope and content identities.
The command line and Python construct it; the report reader receives the whole request. Removes separate
parsing in cli.rs and fdu-py, the seven validate_controls sites, surface-only validate_analysis, and the
size and watch-view default mismatches.

## Notes

2026-09-17 (PR #78 review): Phase 1 item 3. Compose Request {scope, content, selection, views, now} from existing typed parts; (scope, content) held by retained indexes and opened roots, (selection, views, now) per read; one constructor and validate; defaults: allocated sizes, tree view for watch on every surface; opened reads and Python Index validate through it; refuse --watch --cache only; Rust Session and Python Index.watch refuse an analyzed index (CLI already refuses --watch --analyze).
