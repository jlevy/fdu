---
type: is
id: is-01m0tdy9ceep2byvbtyvwc2vky
title: Release the GIL and measure the full Python read boundary
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels:
  - pr47-review
  - performance
  - metabrowser
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T17:43:54.508Z
updated_at: 2026-08-24T20:46:47.839Z
---
At PR 47 head e658915, PyIndex.read calls IndexHandle::read while holding the GIL. As fdu-samw adds filtered tree, recent, and catalog projections, native O(entries) work on an asyncio worker thread will still freeze the event-loop thread because the extension owns the GIL. The Work record also omits CPU time and binding-copy bytes, and wall_ns stops before Python object construction, despite closed fdu-qgl9 promising those dimensions. Fix: detach the native read and reacquire the GIL only for bounded conversion; count bytes materialized across that conversion. Measure CPU exactly in an opt-in detailed mode or mark it unavailable in both contracts rather than deriving it from wall minus lock wait. Test the actual Index.read path for concurrent Python progress and include conversion in the performance harness. Review finding FDU47-R6.

## Notes

DESIGN SETTLED (2026-08-24 review). Verified: `py.detach` wraps refresh, analyze, watch
next_batch, prepare_report, and both opens -- but NOT `PyIndex.read`. The full report
query executes holding the GIL, and fdu-samw made that read strictly heavier.

THE FIX. Detach around the native `IndexHandle::read`; reacquire only for the bounded
dict/list construction. Count `binding_bytes` during that conversion (names, strings,
and buffers actually materialized into Python objects -- the number MetaBrowser's
contract requires per read). CPU: report exactly (an opt-in detailed mode using
clock_gettime(CLOCK_THREAD_CPUTIME_ID)-equivalent via std) or mark unavailable -- never
infer from wall minus lock wait. NOTE: MetaBrowser's contract currently makes CPU a
mandatory nonnegative count; the consolidated comment on their PR asks them to make it
optional-or-sentinel so an honest provider is expressible. Do not block on their edit:
ship exact-or-absent and let the adapter map absent per whatever the contract decides.

SEQUENCE after fdu-91ru: the coherent-envelope work reshapes PyIndex.read's body; doing
GIL/byte accounting in the same area twice is churn. 91ru first, then this.

TESTS. Concurrent Python threads make progress through the actual Index.read while one
holds a long report (the R6 test); binding_bytes nonzero for a row-bearing read and
zero for the empty checkpoint; conversion cost visible in the provider harness.
