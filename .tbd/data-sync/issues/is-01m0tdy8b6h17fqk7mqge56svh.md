---
type: is
id: is-01m0tdy8b6h17fqk7mqge56svh
title: Complete the coherent read envelope and version-pinned paging
kind: bug
status: open
priority: 1
version: 6
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
updated_at: 2026-08-24T20:46:49.779Z
---
At PR 47 head e658915, the core ReadBundle captures clock, scope, freshness, and projections under one guard, but PyIndex.read releases that guard and then locks RunState to attach complete, source, and errors. A refresh can therefore pair old data with new status or new data with old status. ReadRequest also has no requested clock or version, so a multi-page catalog can silently mix states after a mutation. Fix: return lifecycle, coverage, freshness, source, progress, and typed issues from the same versioned engine image; add an expected session and clock to a read and return VersionUnavailable on mismatch. A provider may retain only the current version: page two either sees the exact version or fails, never advances silently. Add forced interleaving and mutation-between-pages tests. This is follow-up to closed fdu-2ivi and should precede the wider algebra in fdu-samw. Review finding FDU47-R4.

## Notes

DESIGN SETTLED (2026-08-24 review). Verified: `PyIndex.read` samples RunState under its
own lock BEFORE the engine bundle; the comment argues the report and the bundle agree
with each other, which is true and beside the point -- both can disagree with the engine
image captured after. And `ReadRequest` has no expected version, so pages can mix
generations.

THE FIX, two halves:

1. One lock. RunState (complete/source/errors -- lifecycle, progress, typed issues)
   moves under the same guard as the index, written by the scan/refresh/watch paths at
   the moments they already hold the write lock. `IndexHandle::read` then captures
   projections, clock, freshness, scope, AND run state from one boundary. The binding
   stops sampling `self.state()` separately.

2. Pinning. `ReadRequest` gains `expected: Option<Cursor>` (fdu-325q's type -- land that
   first). Mismatch returns a structured VersionUnavailable error carrying current and
   expected; retaining only the current image is the whole retention policy, per the
   owner's D3: "a stale pin may fail instead of requiring historical snapshots". The
   MetaBrowser coordinator restarts the assembly on that error; pages never silently
   continue on a newer version.

MetaBrowser's contract adds one more requirement worth building to here: an EMPTY
ReadRequest is the constant-work checkpoint ("must not traverse inventory entries").
Assert it with the Work counters: a no-projection read reports zero visits.

TESTS. Forced status interleaving (refresh between old sample point and read -- now
structurally impossible); mutation-between-pages returns VersionUnavailable; empty-read
zero-work; the envelope fields all quote one clock.
