---
type: is
id: is-01m2pmra8yqrcxg27kc6ezg9vd
title: "Phase 1 item 4: provenance and tree status computed on every route"
kind: epic
status: open
priority: 0
version: 15
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - design
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
child_order_hints:
  - is-01m2pye5q72fsbb1n3y19mz0ma
  - is-01m2pye61m0r7gny0f5nm361e3
  - is-01m2pye6byr0vp0t4v8k2nk8vb
  - is-01m2pye6pak55wktt2d0ge1je2
  - is-01m2pye70x3nc8p6t827sywb23
  - is-01m2pye7bt83a2erfcav6hhkfc
created_at: 2026-09-17T02:57:24.765Z
updated_at: 2026-09-17T05:47:43.585Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 4: Provenance and Tree Status", moved here when the item became beads (PR #78 at `e52383d4`; locators verified at `5f2d36d`). The commits are this bead's children, P1.4.1 to P1.4.6; their blockers carry the ordering, so this bead only groups them and closes when they do.

```rust
pub struct TreeStatus { pub complete: bool, pub coverage: Coverage, pub errors: Vec<String>, pub errors_omitted: u64 }
pub struct ReportProvenance { pub source: ReportSource, pub freshness: Freshness,
  pub scan_started_at: Option<SystemTime>, pub generated_at: SystemTime, pub tiers: TierProvenance }
pub struct TierProvenance { pub entries: TierState, pub content: Option<TierState> }
pub struct TierState { pub source: Source, pub freshness: Freshness, pub observed_at_ns: i64 }
impl TreeStatus { pub fn of(index: &Index, request: &Request) -> Self; pub(crate) fn of_walk(scan: &ScanReport) -> Self }
impl ReportProvenance { pub fn of(index: &Index, generated_at: SystemTime) -> Self;
  pub(crate) fn of_walk(started: SystemTime, generated_at: SystemTime, complete: bool) -> Self }
```

The types live in `query/query_status.rs`; the report type is renamed from `Provenance`,
which also names the per-entry type at the crate root (`lib.rs:120`).
`report(index, request, generated_at)` computes both, so no caller builds provenance.
`scan_started_at` means the start of the oldest verification pass whose facts the answer
serves: for a stale answer, the pass that wrote the snapshot, read from the format-5
header’s `verified_started_at_ns`.

| File | Function or type | Change |
| --- | --- | --- |
| `query/query_report.rs` | `Provenance` (`:463-481`), `Report` (`:779-829`), `report_in` (`:965-1012`), `report_summary` (`:1053`) | Split into `status` and `provenance`; compute both; stop reading `index.freshness()` directly (`:998`) |
| `scan.rs` | `consolidate_detached_index` (`:3739-3755`); error branches in `reconcile_target_inner` (for example `:4352-4355`) | `index.record_walk(&errors, started_at)` marks each failed path `Partial` and retains its issue; a pass that cannot list a directory removes its retained descendants through `remove_known_children` (`:5178`) |
| `index.rs` | `set_initial_freshness` (`:2671`), `begin_reconcile` (`:2692`), `finish_reconcile` (`:2728`, `:2776`), `IndexState.source` (set at `snapshot.rs:650`, `:710`) | Mark failed paths rather than the pass root; stamp `verified_started_at_ns`; producers keep `source` current (`Scanned`, `Revalidated`, `Cached`) |
| `watch_session.rs` | `live_provenance` (`:396-404`), `report` (`:153`) | Delete `live_provenance`; `report(generated_at)` |
| `opened/read.rs` | `:254-264` | Replaced by `TreeStatus::of` and `ReportProvenance::of` |
| `execution.rs` | `:279-285`, `:310-322` | Delete; the reader computes both |
| `crates/fdu/src/cli.rs` | `:777`, `:801-811`, `:962`; `run` (`:729-742`) | Delete construction; read `report.status` |
| `crates/fdu-py/src/lib.rs` | `PyIndex` fields `errors`, `operation_complete`, `scan_started_at`, `source` (`:138-146`), set in `open` (`:1640-1653`), `scan` (`:1696-1727`), `refresh` (`:457-472`); `status_dict` (`:646`); getters (`:165`, `:177`); `build_report` (`:591-597`) | Delete the fields; compute from the index |
| `crates/fdu-py/python/fdu/_api.py` | `Index.report` (`:260-270`) | Delete the errors override |
| `report_format.rs` | JSON envelope (`:575-600`), YAML (`:976-996`) | Read the split fields with the same output |

**Call sites:** the six production constructions (`execution.rs:279`, `:310`;
`crates/fdu/src/cli.rs:801`; `watch_session.rs:397`; `crates/fdu-py/src/lib.rs:591`;
`opened/read.rs:254`), `examples/perf_probe.rs:555-561`, test constructions in
`query/query_report.rs`, `report_format.rs`, `content/content_analysis.rs`, and
`execution.rs`, and `live_provenance` callers at `crates/fdu/src/cli.rs:962`,
`crates/fdu-py/src/lib.rs:1105`, and `watch_session_integration.rs:246` and `:264`.

**Tests:** `TreeStatus::of` names each failed path on a partial one-shot index; a watch
repaint over an unreadable subtree reports `complete: false`; cold and warm answers are
equal with an unreadable subtree; cache-only `scan_started_at` is the writing pass’s
start; content freshness is stale under cache-only; a tree with more than 64 failed
paths gives equal cold and warm `errors` and `errors_omitted`. Update `scan.rs:8218` and
`:8580` for dropped descendants, review `index.rs:8524-8608`, and regenerate the
opened-root golden. Envelopes are unchanged, so machine goldens should not change.

**Commits:**
1. Split the `Report` fields; writers emit the same bytes.
2. Record walk failures per path and add `TreeStatus::of`.
3. Producers maintain `IndexState.source`; add `ReportProvenance::of`; delete
   `live_provenance` and the Python fields.
4. One `scan_started_at`, reading `verified_started_at_ns` from the snapshot header item
   2 writes.
5. Reconciliation drops facts under unverified directories; clear `unverified-subtree`.
6. Per-tier content provenance.

**Order of errors:** `errors` holds the 64 failures smallest by path and
`errors_omitted` counts the rest.
One-shot scan errors are already sorted (`scan.rs:2414`), while retained issues are
capped at insertion in walk order (`index.rs:2486`), so `record_walk` keeps retained
failures in path order regardless of walk order.

**Risks:** issues are capped at 64 while scan errors are not, so `errors` becomes
bounded with `errors_omitted`; dropping descendants matches a cold walk and keeps
roll-ups exact, but an opened root loses last-known children it could have shown as
unknown, and a transient permission error forces a re-walk.

## Original scope (before the implementation detail)

Per tier: source, freshness, observation time, completeness, coverage, errors; composed into the report envelope.
Every route computes it: one-shot, open, Python Index, watch repaints (no hard-coded complete/errors/source),
opened reads. One meaning for scan_started_at. Content gains freshness.

## Notes

2026-09-17 (PR #78 review): Phase 1 item 4. Split tree status (complete, errors, coverage; part of the answer, compared by the invariant) from provenance (source, freshness, timestamps; excluded). Compute both on every route from the index, generalizing opened/read.rs:254-263; one meaning for scan_started_at; never serve retained facts under an unverified subtree (partial answers contain exactly the verified part).
