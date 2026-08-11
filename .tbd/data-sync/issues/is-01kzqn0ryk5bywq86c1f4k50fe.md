---
type: is
id: is-01kzqn0ryk5bywq86c1f4k50fe
title: "P1: Query/Report core and pure report() over four views"
kind: task
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzqn1e57thy4skv9yjtcpp2h
  - type: blocks
    target: is-01kzqn23etqsjxe0pnn1hx1jng
  - type: blocks
    target: is-01kzqn4eh461jy13mvs25bmwvn
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:34:10.898Z
updated_at: 2026-08-11T07:03:42.302Z
closed_at: 2026-08-11T07:03:42.301Z
close_reason: "query/report.rs implements ViewSpec, Query, Provenance, Report, Section, and pure report(). Both performance tiers implemented with a test pinning their equivalence; one traversal serves all filtered views in a request; per-view sort defaults with name tiebreak for golden stability; depth 0 keeps du's root-totals meaning and marks truncation. 14 tests. Deviation recorded on the function: provenance is a third argument because generated_at cannot be sampled purely."
---
query module: ViewSpec (Tree|Types|Files|Summary), Query { selection, views }, Report { scan_started_at, generated_at, source, freshness, complete, scope, one section per requested view in request order }, and report(&Index, &Query) -> Report: pure, never scans, never mutates - views are readers and the delta contract stands. View semantics: tree = per-directory rollups (files, dirs, bytes, allocated, newest mtime) bounded by depth/limit, default sort size desc; types = flat per derived extension; files = flat matching entries, default name asc; summary = one aggregate row. Two performance tiers, tested separately and documented in help: an unfiltered tree/types/summary reads pre-computed RollUp state directly, while any selection filter (and the files view always) traverses the retained index in memory; both are milliseconds warm and neither touches the filesystem. Thread scan_started_at from the walk/revalidation start through open() and scan_into_index; stamp generated_at at render. Tests: per view x selection x tier over a fixed synthetic index, plus a property test that adding a view never changes another view's section.
