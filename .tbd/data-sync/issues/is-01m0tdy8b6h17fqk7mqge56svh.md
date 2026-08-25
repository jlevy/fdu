---
type: is
id: is-01m0tdy8b6h17fqk7mqge56svh
title: Complete the coherent read envelope and version-pinned paging
kind: bug
status: open
priority: 1
version: 24
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5018121437
    at: 2026-08-25T11:04:22.560Z
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
updated_at: 2026-08-25T11:04:22.561Z
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

EXACT-HEAD FOLLOW-UP at PR #47 4eac1b2 (2026-08-25). `browser_provider.recent_page` does not implement recent-result paging. `after` and `limit` are the child-page arguments on `index.read(children_of=Path())`, while the `FILES` report is independently truncated by `Selection.limit` and has no continuation. The call also omits `expected`, and the smoke test invokes only one 50-row result, so it cannot prove advancing pages, one version, or exact lossless remainder. This confirms rather than closes the native filtered-tree/catalog paging gate. See the formal review on PR #47 at this head.



CONFIRMED (MB74-D4, answered on PR #47 at head `eaae030`): the consumer paging contract
stays bounded and lossless. Filtered-tree and catalog pages need native version-pinned
continuations and exact remainders; unbounded FFI, adapter-side mirrors, and false terminal
pages are all forbidden. This bead is the implementation owner, and it is the item that
most directly gates the adoption spike.



SHIPPED (MB74-D4). `EntryPageRequest` / `EntryPage` on `ReadRequest`/`ReadBundle`, and
`Index.read(entries=True, entries_limit=..., entries_after=...)` on the Python surface.

The invariant a provider contract needs and a truncating limit cannot give: page a pinned
selection until `next` is `None` and the concatenation is the whole answer, in order, with
no repeats and no gaps. `remaining` is exact and is zero **exactly** when `next` is `None`
-- both derived from one count rather than reported independently, because that pairing is
what a consumer checks to detect a lost suffix.

Three design points, each forced rather than chosen:

**Path order, and a path cursor.** A resumable cursor has to be a value the walk can seek
past, and only the tree's own key is stable while the set behind it shifts -- the same
argument `ChildPageRequest::after` already makes against an offset. A page ordered by size
would need the size *and* the path to break ties and would still repeat a row whose size
changed between pages.

**Pre-order, one node per stack item.** Pop it, emit it, then push its children on top so
they are visited before the siblings already waiting below. Pushing a whole directory's
children and emitting them in the same step is the shape that looks right and is not: it
emits every child of a directory before descending into any of them, which is
breadth-per-directory rather than path order, and a cursor that seeks in path order then
skips whole subtrees. The first implementation did exactly that and four tests caught it.

**Per-entry totals, never per-subtree.** A flat page enumerates every match in its own
right, so folding a matched directory's roll-up in would count its matched descendants
twice.

`totals` and `total` describe the selection rather than the page -- the denominator a
bounded page needs to be honest about itself. Charged as its own projection because a
page's cost scales with what it *considered*, not what it returned: a narrow selection over
a wide tree pays that on every page, and folding it into `report` would hide it.

Tests: `crates/fdu-core/tests/entry_paging.rs`, eight cases including assembly at six page
sizes against the unpaged answer, plus
`public_smoke.py:check_bounded_pages_assemble_into_one_complete_answer`. Mutation-checked:
a `remaining` that ignores what the cursor already covered, and a `next` retained when
nothing remains -- four tests fail on each.

A defect this found, unrelated to paging and worse than it: **`TreeNode` has carried a
`kind` since `others` arrived, the binding never emitted it, and the Python model requires
it -- so every tree section the package produced raised `KeyError` on parse.** The tree view
is the one a browser draws, and nothing caught it because the smoke suite asked for every
view except that one. Fixed, with
`public_smoke.py:check_a_tree_report_parses_on_the_python_surface` covering both the
standalone report and the bundled read.

NOT DONE: no CLI form. A flat page with an opaque cursor is not a shape a one-shot command
line wants, and the repository's rule is one-directional -- nothing may be reachable *only*
by flag, and a library-only capability is allowed.

EXACT-HEAD REVIEW at PR #47 9f9bd3d (2026-08-25). The new `EntryPage` surface is functionally bounded and lossless, but every continuation repeats a full subtree and selection pass. `entry_page()` starts at the root, visits and filters every entry, recomputes `total` and `totals`, and counts every match at or before `after` (`crates/fdu-core/src/index.rs:4040-4123`). The implementation explicitly says the scan is the seek because no ordered path index exists (4102-4105). This violates MetaBrowser #74’s Phase 2 gate that a continuation must not repeat a full projection pass merely to advance one page, and makes a P-page assembly O(index × P).

Keep the native, version-pinned, exact-remainder API shape, but make continuation work proportional to the requested page plus bounded native seek. A clean implementation could use an opaque version-bound continuation carrying a bounded traversal checkpoint and exact remaining/totals, or an authoritative native ordered index; it must not create a Python mirror, duplicate adapter cursor, or retained full result set. Add a large-fixture work-counter test proving page 2+ does not revisit the whole selection or preceding prefix. The existing `entries_visited` counter makes that acceptance directly assertable.
