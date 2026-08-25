---
type: is
id: is-01m0tdy8b6h17fqk7mqge56svh
title: Complete the coherent read envelope and version-pinned paging
kind: bug
status: closed
priority: 1
version: 26
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
updated_at: 2026-08-25T13:56:53.430Z
closed_at: 2026-08-25T13:56:53.430Z
close_reason: null
resolution: null
duplicate_of: null
---
At PR 47 head e658915, the core ReadBundle captures clock, scope, freshness, and projections under one guard, but PyIndex.read releases that guard and then locks RunState to attach complete, source, and errors. A refresh can therefore pair old data with new status or new data with old status. ReadRequest also has no requested clock or version, so a multi-page catalog can silently mix states after a mutation. Fix: return lifecycle, coverage, freshness, source, progress, and typed issues from the same versioned engine image; add an expected session and clock to a read and return VersionUnavailable on mismatch. A provider may retain only the current version: page two either sees the exact version or fails, never advances silently. Add forced interleaving and mutation-between-pages tests. This is follow-up to closed fdu-2ivi and should precede the wider algebra in fdu-samw. Review finding FDU47-R4.

## Notes

FDU47-D1 addressed at 051e7cc: continuation cost is proportional to the page, not
to the index.

The regression was real and was mine. entry_page began at the root on every call,
filtered the whole subtree, recomputed the selection-wide denominator, and
counted every match at or before the cursor -- so assembling P pages cost P
passes. Bounded and lossless and quadratic is still unusable at catalog sizes.

Two costs, two answers:

- The seek. A bare path cursor identifies where to resume and gives no way to
  get there except forward from the top. seek_after descends the cursor path
  instead: at each level it pushes the siblings *after* the component it came
  through (a range over an ordered map, not a scan that discards what it passes),
  then the cursor's own children on top -- exactly the stack the walk would have
  left. No entry before the cursor is looked at.
- The denominator. An arbitrary predicate over a tree has no ordered index to
  count through, so the first page walks the selection to learn its size. That
  one is unavoidable; recomputing it per page is not. EntryCursor carries the
  total, the aggregates and the delivered count forward, so a continuation stops
  as soon as it has its rows.

Measured on a 660-entry fixture at limit 5: 666 entries visited for the first
page, then a flat 14 for every page after it -- page 2 and page 25 alike. The
test asserts that flatness rather than a ratio, because flatness *is* the
property: a continuation that crossed the prefix would cost more with every page.

The cursor is version-bound, and that is not belt-and-braces beside
ReadRequest::expected: its counts were established against one image, so a
continuation replayed against another would report a denominator for a tree that
is no longer there. Refused whether or not the caller also pinned the read.

No Python mirror, no duplicate adapter cursor, no retained result set, no second
watcher. EntryCursor crosses as a value with the same fields; a bare path is
refused at the facade with a sentence rather than an attribute error out of an
encoder, since entries_after took a path before it took a value.

Tests: the eight lossless-assembly cases are unchanged and still pass, plus
continuing_an_assembly_costs_a_page_rather_than_a_pass,
every_page_reports_the_denominator_the_first_one_established, and
a_continuation_from_another_version_is_refused. Mutations checked: making
continuations recount (664 against 666, exactly the regression), and seeking by
scanning from the top. Both fail named tests. The Python smoke test asserts the
same cost property across the binding.

make check and make cross-lint pass; parity holds at 21 recorded deviations.
