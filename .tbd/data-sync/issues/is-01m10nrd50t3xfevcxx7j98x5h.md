---
type: is
id: is-01m10nrd50t3xfevcxx7j98x5h
title: Complete bounded tree, flat, aggregate, recent, and navigation projections
kind: task
status: blocked
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies: []
parent_id: is-01m0y1shykye8sc7h7e9rkk6kh
created_at: 2026-08-27T03:55:56.959Z
updated_at: 2026-08-28T00:48:45.162Z
---
Update opened/read.rs to traverse approved maintained structures in exact tree and flat orders, resume without root rescans, and return separate exact-or-capped totals and honest portable-path issues. Gate every page size against unpaged independent recomputation, work bounds, and the canonical opened-root sessions.

## Notes

## Notes

Registry parsing, browsing taxonomy, on-demand opened-row name classification, and the
additive `EntrySelection` predicates are complete and committed as `328ca65`. Local
`make check` exits 0, all 19 CI checks are green, and `make cross-lint` is clean on both
cross targets. The prior session ended before handoff; the work was recovered, its stale
opened-root goldens re-blessed one scenario at a time, and pushed.

## Corrected scope, 2026-08-27

An earlier reading of this bead treated `Recent`, `Navigation`, and `Catalog` as missing
`ReadProjection` variants. That is wrong and would have violated the design. Decision
line 70 of the plan asks "Put MetaBrowser's eight query names in fdu?" and answers "No.
Keep a small fdu-native read algebra and map to the client vocabulary in a thin adapter."

The native vocabulary is five projections and is already complete: Lookup, Tree page,
Flat entry page, Roll-up/report, and Diagnostics, plus Aggregate and Continue. The plan's
own mapping table routes the client queries onto them:

| MetaBrowser query | fdu operation | State |
| --- | --- | --- |
| Entry | Lookup | done |
| Directory | Tree page at render depth | **needs render depth** |
| Filtered tree | Complete-or-limit selected-tree report at render depth | **open** |
| Roll-up | Roll-up/report | done |
| Navigation | Report over maintained aggregate indexes | **open: report walks instead** |
| Recent | Bounded ranked report | **open: report walks instead** |
| Catalog | Flat entry page with compact fields and predicates | done at `328ca65` |
| Diagnostics | Diagnostics and the coherent envelope | done |

So no new projection variant is added. Three pieces of work remain.

1. **Serve ranked and aggregate views from the maintained indexes.** `report_work`
   currently charges `entries` — a full pass — to `work.rows` for `ViewSpec::Recent` and
   for the classification views, even on an unfiltered query, while `ServingIndexes`
   already maintains `recent_files` as a global newest-first `BTreeSet` and
   `semantic_by_directory` as per-directory classification tallies. An unfiltered Recent
   view should read the maintained set in work proportional to the limit and charge
   `maintained`, not `rows`.

   Design constraint: `crate::query::report` is the shared one-shot and command-line
   machinery and is currently serving-blind, though it holds the `Index` and can reach
   `serving`. The maintained index must be an accelerator only — the walk stays the
   definition and the detached path must keep answering identically with `serving`
   absent. The oracle is exactly that: an opened root and a detached index must return
   equal reports while charging different work.

2. **Render depth on the tree projection.** `ReadProjection::Tree` carries only `path`
   and `page`; the contract needs a bounded recursive page at a requested render depth,
   carrying the ancestors needed to render it and per-directory completeness.

3. **Complete-or-limit selected-tree report** at a render depth, for the filtered-tree
   query, plus Python registry-document input with provider identity derived from parsed
   content.

Do not close this bead until all three land with their independent-recomputation tests.
