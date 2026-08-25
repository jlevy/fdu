---
type: is
id: is-01m0tdy8b6h17fqk7mqge56svh
title: Complete the coherent read envelope and version-pinned paging
kind: bug
status: open
priority: 1
version: 14
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels:
  - pr47-review
  - metabrowser
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
  - type: blocks
    target: is-01m0tdy9ceep2byvbtyvwc2vky
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T17:43:53.445Z
updated_at: 2026-08-25T00:59:44.751Z
closed_at: 2026-08-24T23:31:10.933Z
close_reason: |
  Shipped. `make check` green.

  THE DEFECT, verified. `PyIndex.read` sampled `RunState` under its own mutex *before* the
  engine bundle, and the comment there argued the report and the bundle would agree with
  each other -- true, and beside the point. Both could disagree with the engine image
  captured after. `build_report` and `status_dict` did the same thing, so three read paths
  each paired one instant's rows with another instant's claim about how far to trust them,
  both halves individually true.

  THE FIX, and it is a placement rather than a mechanism. `RunFacts { scan_started_at,
  source, complete, errors }` moved from beside the index into it, under the same
  `RwLock` as the tree. `IndexHandle::read` returns them in the bundle;
  `ReportRequest.provenance` is gone, replaced by a bare `generated_at` -- the one run fact
  that genuinely belongs to a rendering rather than to an index. `RunFacts::provenance()`
  builds the envelope from inside the guard. The binding's `RunState` is down to
  `telemetry`, which is about the call rather than about the index and correctly stays.

  `open_for_report` records the facts on all three paths through one `record_run` helper, so
  they cannot disagree about what they claim; cache-only records a default `ScanReport`,
  because a tier that walks nothing has nothing that could have failed. `refresh` pushes its
  own -- including analysis failures, which the engine has no view of -- through the new
  `IndexHandle::set_run_facts`. `status_dict` now takes freshness under the same guard too:
  a status envelope assembled from four instants was exactly the thing it described itself
  as not being.

  PINNING. `ReadRequest.expected: Option<Cursor>`, checked under the guard before any
  projection runs, returning `Error::VersionUnavailable { requested, current }`. Retaining
  only the current image *is* the retention policy, so an aged-out pin failing is the
  designed answer: the caller restarts a bounded assembly, which is cheap, rather than the
  engine holding history, which is not. `ReadBundle` also carries `cursor` beside `clock`,
  so one value serves the cache key, the next replay, and the next page's pin.

  TESTS.
  - `a_pinned_read_refuses_a_version_the_index_has_moved_past`: a pin at the current version
    reads; after a write the same pin fails; unpinned, the same request reads the new tree.
  - `an_empty_read_is_a_checkpoint_that_visits_nothing`: zero entries and dirs visited, no
    projections, and the envelope still present -- the constant-work form the provider
    contract requires, checked rather than asserted.
  - `the_run_envelope_arrives_with_the_rows_it_describes`: source, completeness and errors
    come back from the same read as the totals.
  - Python: a successful pin before the tree moves, a refused one after, and a checkpoint
    read asserting `work.entries_visited == 0`.

  NOTE FOR fdu-hfdw. The checkpoint test asserts zero visits, which is currently true partly
  because the report projection charges none. When `fdu-hfdw` threads a visit sink through
  `query::report`, that test keeps its meaning and gains teeth: an empty request must still
  visit nothing once a filtered report starts charging honestly.
resolution: null
duplicate_of: null
---
At PR 47 head e658915, the core ReadBundle captures clock, scope, freshness, and projections under one guard, but PyIndex.read releases that guard and then locks RunState to attach complete, source, and errors. A refresh can therefore pair old data with new status or new data with old status. ReadRequest also has no requested clock or version, so a multi-page catalog can silently mix states after a mutation. Fix: return lifecycle, coverage, freshness, source, progress, and typed issues from the same versioned engine image; add an expected session and clock to a read and return VersionUnavailable on mismatch. A provider may retain only the current version: page two either sees the exact version or fails, never advances silently. Add forced interleaving and mutation-between-pages tests. This is follow-up to closed fdu-2ivi and should precede the wider algebra in fdu-samw. Review finding FDU47-R4.

## Notes

ITEM 2 RESOLVED by fdu-jxs0. `set_run_facts` is now a clocked commit that enters the
journal, so a cursor names exactly one envelope: the refresh's rows commit, then the
envelope commits at a distinct clock, and a consumer between them holds a position whose
envelope is the prior one -- coherent, and it learns of the transition from the feed.

Worth recording because it changes what the "narrow fix" can be. Run facts cannot
literally share the rows' commit on the Python path: analysis runs after reconciliation
and contributes errors the engine has no view of. Since freshness is `Reconciling` from
`begin_reconcile` until `finish_reconcile`, the interim window is honestly labelled
in-flight rather than mislabelled current, which is what item 2 was actually about.

STILL OPEN: item 1 (lifecycle phase, progress, coverage reason, and typed issues in the
envelope -- `errors: Vec<String>` is not a vocabulary a consumer can branch on) and item 3
(`build_query` resolves relative `modified_since`/`modified_before` against a fresh
`SystemTime::now()` per call, so a version-pinned multi-page recency assembly can change
membership without the version moving).
