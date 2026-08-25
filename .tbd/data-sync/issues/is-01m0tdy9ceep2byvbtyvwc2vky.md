---
type: is
id: is-01m0tdy9ceep2byvbtyvwc2vky
title: Release the GIL and measure the full Python read boundary
kind: task
status: open
priority: 1
version: 8
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
updated_at: 2026-08-25T00:49:15.302Z
closed_at: null
close_reason: null
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

Reopened: Reopened at exact PR 47 head 56dcf56 after review. The GIL detach is correct, but the full-boundary acceptance is not met: conversion_ns is stamped at crates/fdu-py/src/lib.rs:919 before projection dictionaries at lines 925-930, and it cannot include the public Bundle/model conversion in python/fdu/_api.py:384-409. wall_ns therefore remains native-only despite the public Work and Bundle wording that describes total call cost. materialized_bytes at lib.rs:326-346 is an undocumented estimate and a second O(output) traversal; it omits envelope, issue, tag, classification, provenance, and other dynamic fields, so it cannot be mapped honestly to MetaBrowser bytes_copied. The public smoke test at public_smoke.py:529-551 only proves four calls eventually finish and would pass if they serialized while the GIL stayed held. Fix by defining one exact full public-call wall counter plus explicit native/conversion components, count a documented logical binding payload during conversion without a second traversal, and add a forced-interleaving test through actual Index.read that fails when the GIL is retained. CPU None is correct.

Reopened: Reopened again at exact PR 47 head 715f748. The follow-up still does not satisfy the full-boundary gate. First, model_ns in python/fdu/_api.py:422-424 is wall minus native, so it includes the extension conversion that conversion_ns already reports; the three phases do not decompose as documented. The same Work type is also used for ProjectionWork, whose wall_ns remains engine-local while the class now documents wall_ns as the full public call. Second, the stated binding-payload rule is not implemented exhaustively: ext_remainder_dict emits four scalars but charges six; child_list emits three file-attribute scalars and directory empty without charging them; the report envelope, bounds, and metric summaries emit many uncharged values; top-level clock/entries/complete are three scalars but only two are charged. The rollup-only delta test cannot catch these families. Third, the GIL test relies on a hardware-timing threshold and CI already disproved it on macOS/Python 3.14: elapsed was 0.0070s versus the required 0.0080s. Replace the timing oracle with a forced barrier through the actual pymethod; define distinct total-boundary versus projection-native work semantics; time model conversion directly; and test exact payload deltas for envelope, child, remainder, report, optional/null, and metrics families. CPU None and the actual py.detach remain correct.
