---
type: is
id: is-01m0rw7d4h3t49rwvk11cmk5xb
title: The bundled read carries only three projections, not the query algebra
kind: feature
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T03:15:04.465Z
updated_at: 2026-08-24T17:44:46.449Z
closed_at: 2026-08-24T17:44:46.449Z
close_reason: |
  `ReadRequest` now carries a `report` projection and `ReadBundle` returns the `Report` it
  produced, evaluated inside the same read guard as the children, the roll-ups and the
  totals. A consumer drawing a directory listing beside a "recently changed" panel gets both
  from one call, sharing one clock, one freshness, one scope, and one work record.

  WHAT LANDED

  - `ReportRequest { query, provenance }` on `ReadRequest`; `ReadBundle.report:
    Option<Report>`. Not a second query language: it is the `Query` every other surface
    already takes, called through `crate::query::report` under the guard.
  - The provenance rides with the query rather than being derived inside the read. Those
    facts belong to the RUN, not to the index: a read cannot know when a scan started or
    which cache tier answered, and filling them in from whatever was in reach would put a
    number in the envelope that nothing measured. Every caller that renders a report
    already holds one; the Python binding reads it from the same `RunState` it stamps the
    bundle's own `complete`/`source`/`errors` from, so the report cannot disagree with the
    bundle it arrived in.
  - `ProjectionWork { children, total, rollups, report }` on the bundle, answering the
    bead's check directly. A bundle that reported only a total would answer "this read was
    slow" and never "which part of it" -- the question a serving loop has to act on. The
    parts run in sequence inside one guard, so their wall times are genuinely theirs;
    `lock_wait_ns` stays zero on each and lives on the bundle, because that one is shared
    and splitting it would be inventing a number. `ProjectionWork::sum()` is the bundle's
    counted work, and a test asserts the equality term by term so no projection's cost can
    go missing or land in the wrong bucket.
  - `Section::row_count`/`name_bytes` and `Report::row_count`/`name_bytes` charge the report
    projection what the RESULT carries rather than what the question considered: `largest`
    ranks every entry and emits twenty, and a counter reporting the ranking would be
    describing a different question than "how much is a consumer about to copy out".
    The report projection deliberately claims no `entries_visited` -- a report may serve
    from maintained roll-up state or re-aggregate by walking, and reporting a walk it did
    not do, or a zero for one it did, is worse than reporting neither.
    `Selection::is_unfiltered` is what decides which happened, and it is on the query the
    caller already holds.
  - Python: `Index.read(..., query=Query(...))` returns `Bundle.report` and
    `Bundle.projections`. The report's row bound arrives as `limit_rows` because `limit` on
    that call already means the child page's -- one name for two bounds is how a caller ends
    up bounding the listing when they meant the report.
  - `report_format::report_schema` and `generator` are now public, and the binding's report
    dict carries both. It was missing them, so `report_from_dict` -- the parser the
    standalone `report()` already uses -- could not read it. Fixing that is what lets the
    bundled report and the standalone one be parsed by one function from one wire shape,
    rather than by two parsers that agree until they do not.

  TESTS

  - `a_report_in_a_bundle_describes_the_same_instant_as_the_rows` runs under a live writer
    and asserts the summary section's file count equals the bundle's own totals, 300 times.
    An invariant a race breaks, not a literal -- a literal agrees with a stale answer as
    readily as a fresh one, which is the same reasoning
    `a_bundled_read_cannot_straddle_a_commit` uses one level down.
  - `a_bundle_says_which_projection_cost_what` pins that every projection is charged, that
    the bundle equals their sum term by term, that the guard wait is the bundle's alone, and
    that a bundle with no query charges nothing to that projection rather than charging an
    empty report's assembly to it.
  - `check_a_bundle_answers_a_query_at_the_same_instant_as_its_rows` in `public_smoke.py`
    pins the same three facts through the public Python API, including that the bundled
    report equals what `index.report(query)` returns for the same query.

  `make check` is green.
resolution: null
duplicate_of: null
---
The contract's point 2 is that "every result carries the exact version, resume cursor,
lifecycle and coverage facts, and work counters that describe THE SAME observation
boundary", over a closed algebra of nine query kinds: entry, directory, filtered_tree,
rollup, navigation, recent, catalog, metadata, diagnostics.

fdu's ReadRequest carries three: children_of (directory), rollups (rollup), total. Every
other kind is reachable only through Index.report(), which takes its own guard.

So a consumer wanting a directory listing AND a recent list at one instant cannot get one.
It makes two calls, a write lands between them, and the page is internally inconsistent in
exactly the way the bundled read was built to prevent -- the rows say one thing, the
sidebar another, and both are individually true. That is the same defect fdu-2ivi fixed
for the listing-plus-header case, still present one level up.

WHAT THIS IS NOT: reproducing MetaBrowser's query types in fdu. The engine already answers
every one of these kinds; what it lacks is the ability to answer SEVERAL under one guard
and return them with one version, cursor, state and work record. The likely shape is
ReadRequest gaining the projections report() already knows how to build, evaluated inside
the same read guard, with the work record summing across them.

Sequence after fdu-5yqb (coverage reason) so the state facts a bundle carries are the
final ones, and check the work counters still add up per projection rather than only in
total -- a bundle that hides which projection cost what is a counter that stopped being
useful.
