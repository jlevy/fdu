---
type: is
id: is-01m0tdy8b6h17fqk7mqge56svh
title: Complete the coherent read envelope and version-pinned paging
kind: bug
status: closed
priority: 1
version: 11
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
updated_at: 2026-08-24T23:31:10.933Z
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

POST-LANDING REVIEW at a3960fb (2026-08-24). The shared read guard, session-scoped cursor, expected-version refusal, and empty checkpoint are fixed. Residual scope from this bead remains: ReadBundle run carries only complete, source, scan_started_at, and untyped string errors; it still has no lifecycle phase, progress, coverage reason, or typed OperationError, and Python maps the strings to generic pathless operation issues. Also, refresh mutates the tree through reconcile_subtree_handle and only later calls set_run_facts; set_run_facts does not advance Clock or emit a delta, so one cursor can name two state envelopes and a read between those calls can pair new rows with the prior run facts. Keep this bead open for the missing structured envelope. Coordinate the clock and change-feed portion with fdu-jxs0 rather than duplicating it. Close only after nonempty typed-error coverage and a forced refresh interleaving or same-cursor state test land.

METABROWSER CONTRACT FOLLOW-UP at a3960fb / MetaBrowser 68eeaac (2026-08-24). Version pinning must also pin time-relative selection. PyIndex.read rebuilds Query on every call, and build_query resolves modified_since/modified_before against a fresh SystemTime::now(). Therefore two calls with the same expected Cursor can select different rows across an age boundary even though the tree version is identical. Add a caller-controlled exact as_of/reference instant to the read/query boundary, reuse it across every page of one assembly, and add a deterministic boundary-crossing test. This stays on fdu-91ru because it is part of coherent version-pinned paging, not a separate query feature.
