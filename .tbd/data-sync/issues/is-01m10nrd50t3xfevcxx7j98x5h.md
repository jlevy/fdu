---
type: is
id: is-01m10nrd50t3xfevcxx7j98x5h
title: Complete bounded tree, flat, aggregate, recent, and navigation projections
kind: task
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies: []
parent_id: is-01m0y1shykye8sc7h7e9rkk6kh
created_at: 2026-08-27T03:55:56.959Z
updated_at: 2026-08-28T00:15:48.372Z
---
Update opened/read.rs to traverse approved maintained structures in exact tree and flat orders, resume without root rescans, and return separate exact-or-capped totals and honest portable-path issues. Gate every page size against unpaged independent recomputation, work bounds, and the canonical opened-root sessions.

## Notes

## Notes

Checkpoint in progress: added dependency-free File Rollup v3 registry parsing, ordered
browsing taxonomy, on-demand opened-row name classification, and additive Selection
predicates for ignored state, logical extension/exact basename identity, terminal suffix,
ancestor name, and inclusive maximum size. Preserved the public compiled-registry
derive_ext/ext_bucket answers and opened-root-only serving allocation. Focused
classifier/selection tests, five transparent opened-root session goldens, and Python
check are green. The bounded recursive tree, maintained recent/navigation reads, Python
registry-document input, and MetaBrowser adapter remain open; do not close this bead yet.
CLI acceptance remains unchanged parser/defaults/output/cache semantics, no existing-path
serving allocation, no Python/async/MetaBrowser dependency, and final
size/startup/memory comparison.

## Recovered and committed 2026-08-27 as `328ca65`

The session carrying the above work ended before handoff, leaving it uncommitted. It is
recovered, stabilized, and pushed; nothing was lost.

One failure was outstanding: the work had first widened `Selection` in place, regenerated
the opened-root goldens at that shape, then refactored to the additive `EntrySelection`
without re-blessing. The four affected goldens were updated one scenario at a time
through `make opened-root-golden-update`, which refuses a blanket corpus update. The
regenerated corpus differs from the intermediate one on `action.read` lines alone — no
`result.` line moved, so no answer changed.

`EntrySelection::retained_heap_bytes` was checked against the 64 KiB continuation record
cap: it accounts for all four added string vectors, so a continuation owning one stays
bounded.

State at `328ca65`: local `make check` exit 0, 558 Rust tests, 125 CLI goldens unchanged,
parity holding with 21 matched deviations, and all 19 CI checks green across the
three-platform matrix and six wheel builds.

## Remaining before this bead closes

1. Bounded native recursive tree and selected-tree complete-or-limit reads.
2. Maintained `Recent` and `Navigation` projections — neither is a `ReadProjection`
   variant yet; the enum currently carries Lookup, RollUp, Tree, Flat, Aggregate, Report,
   Continue, and Diagnostics.
3. Catalog and diagnostic projections without request-time full sorts or Python entry
   loops.
4. Python registry-document input, with provider identity derived from parsed semantic
   content rather than supplied.

Sequenced after this bead, not inside it: the thin adapter under `fdu-2xfp`.
