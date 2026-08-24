---
type: is
id: is-01m0tdy8b6h17fqk7mqge56svh
title: Complete the coherent read envelope and version-pinned paging
kind: bug
status: open
priority: 1
version: 9
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
updated_at: 2026-08-24T22:35:08.320Z
closed_at: null
close_reason: null
resolution: null
duplicate_of: null
---
At PR 47 head e658915, the core ReadBundle captures clock, scope, freshness, and projections under one guard, but PyIndex.read releases that guard and then locks RunState to attach complete, source, and errors. A refresh can therefore pair old data with new status or new data with old status. ReadRequest also has no requested clock or version, so a multi-page catalog can silently mix states after a mutation. Fix: return lifecycle, coverage, freshness, source, progress, and typed issues from the same versioned engine image; add an expected session and clock to a read and return VersionUnavailable on mismatch. A provider may retain only the current version: page two either sees the exact version or fails, never advances silently. Add forced interleaving and mutation-between-pages tests. This is follow-up to closed fdu-2ivi and should precede the wider algebra in fdu-samw. Review finding FDU47-R4.

## Notes

POST-LANDING REVIEW at a3960fb (2026-08-24). The shared read guard, session-scoped cursor, expected-version refusal, and empty checkpoint are fixed. Residual scope from this bead remains: ReadBundle run carries only complete, source, scan_started_at, and untyped string errors; it still has no lifecycle phase, progress, coverage reason, or typed OperationError, and Python maps the strings to generic pathless operation issues. Also, refresh mutates the tree through reconcile_subtree_handle and only later calls set_run_facts; set_run_facts does not advance Clock or emit a delta, so one cursor can name two state envelopes and a read between those calls can pair new rows with the prior run facts. Keep this bead open for the missing structured envelope. Coordinate the clock and change-feed portion with fdu-jxs0 rather than duplicating it. Close only after nonempty typed-error coverage and a forced refresh interleaving or same-cursor state test land.
