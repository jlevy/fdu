---
type: is
id: is-01m0tdy8b6h17fqk7mqge56svh
title: Complete the coherent read envelope and version-pinned paging
kind: bug
status: open
priority: 1
version: 16
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
updated_at: 2026-08-25T05:33:17.493Z
closed_at: null
close_reason: null
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

Reopened: Exact-head review at PR #47 head `7aaaf84` against MetaBrowser #74 at `1e0f9b5`
found that version pinning is implemented, but the bounded paging surface is not yet
complete enough to implement the provider contract.

MetaBrowser requires native, version-pinned continuation for both flat filtered-tree
pages and catalog pages. `FilteredTreeQuery.after` and `CatalogQuery.after` are contract
fields, and the shared provider harness requires a positive row bound, an advancing
cursor, one pinned engine version and `as_of_ns`, no duplicate rows, and exact conserved
`remaining_rows` through the terminal page.

FDU currently has a native continuation only for `ReadRequest.children_page`.
`Report` tree and file sections expose an omission count or aggregate remainder, but no
`after`/`next` cursor; `Query` and `Selection` expose no flat-page offset. A thin adapter
would therefore have to request `limit=all`, cross an unbounded result through PyO3, and
slice or retain it in Python. That violates the mandatory bound and the no-mirror/no
unbounded-FFI adoption rules.

Add native bounded flat-page projections for the filtered-tree and catalog shapes, or a
single native page algebra that serves both. Each page must take a required positive
limit and opaque advancing cursor, return exact remaining rows, honor
`ReadRequest.expected`, and reuse the caller's `as_of` across the assembly. Run the
MetaBrowser provider-harness scenarios against the FDU binding before closing this gate.
