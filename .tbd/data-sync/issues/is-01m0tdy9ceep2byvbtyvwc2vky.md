---
type: is
id: is-01m0tdy9ceep2byvbtyvwc2vky
title: Release the GIL and measure the full Python read boundary
kind: task
status: closed
priority: 1
version: 5
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
updated_at: 2026-08-24T23:30:06.619Z
closed_at: 2026-08-24T23:30:06.619Z
close_reason: |
  Shipped. `make check` green.

  THE GIL. `py.detach` already wrapped refresh, analyze, watch `next_batch`, `prepare_report`
  and both opens -- but not `PyIndex.read`, which `fdu-samw` had just made the heaviest of
  them. A filtered report re-aggregates the entire index, so holding the GIL across it froze
  every other Python thread for a full traversal, and an asyncio consumer moving the call to
  a worker thread would still have frozen its event loop. Now detached, with the GIL
  reacquired only for conversion.

  THE BINDING'S COST. `binding_bytes` counts what the result materializes into Python
  objects; `conversion_ns` times the reacquired half separately from `wall_ns`, since the
  native read now runs without the GIL and the conversion does not. `name_bytes` is what the
  engine *read* and `binding_bytes` is what a caller *pays* to receive -- the difference is
  exactly the conversion, which is why an engine-side counter cannot supply it.

  CPU: ABSENT, NOT INFERRED. `cpu_ns` is `None`. Wall minus lock wait is not CPU time on a
  preemptive system, and putting an inferred number in the one field an embedder uses to
  compare engines is worse than leaving it empty. `Work`'s docstring had previously argued
  CPU was deliberately absent *because* wall minus lock wait covered it; that reasoning is
  replaced rather than kept. MetaBrowser's contract currently makes CPU a mandatory
  nonnegative count -- the comment on their PR #74 (MB74-C2) asks for optional-or-sentinel
  so an honest provider is expressible; the adapter maps `None` to whatever they settle on.

  TESTS, and an honest scoping note. The Rust test blocks *inside* the detach, so a second
  thread must reach `Python::attach` and send while the read runs -- a path needing the
  interpreter hangs rather than finishing late. It exercises the handle read rather than the
  pymethod, so it pins the *precondition* (the whole read path is `Send` and touches no
  Python) rather than the pymethod's detach itself; the name and doc say so instead of
  overclaiming. The first draft counted attaches after the reads finished, which would have
  passed with the GIL held -- worth recording, because that is the shape of a vacuous
  concurrency test. The Python side runs four threads through the real `Index.read` with a
  filtered query and asserts they all finish, plus that `binding_bytes` is non-zero and
  `cpu_ns` is `None`.
resolution: null
duplicate_of: null
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

METABROWSER DECISION LANDED at 68eeaac (2026-08-24). The consumer contract now represents CPU as exact-or-absent: WorkCounters.cpu_time_ns is optional, aggregation is absent if any component is unavailable, and debug JSON emits null rather than zero. Implement the already-settled fdu design directly: exact opt-in native CPU measurement or absence, never wall minus lock wait. No adapter sentinel or compatibility shim is needed.
