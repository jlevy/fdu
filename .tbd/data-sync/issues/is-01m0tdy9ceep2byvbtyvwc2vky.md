---
type: is
id: is-01m0tdy9ceep2byvbtyvwc2vky
title: Release the GIL and measure the full Python read boundary
kind: task
status: closed
priority: 1
version: 10
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - pr47-review
  - performance
  - metabrowser
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T17:43:54.508Z
updated_at: 2026-08-26T07:01:50.826Z
closed_at: 2026-08-25T01:31:41.390Z
close_reason: |
  Shipped. `make check` green, parity holds (23 recorded deviations matched).

  All three findings from the 715f748 review.

  FINDING 1, THE PHASES DID NOT DECOMPOSE. `model_ns` was `wall - native`, which looked
  equivalent to timing the model phase and was not: it swallowed the extension's own
  `conversion_ns` too, so summing the three double-counted a span already reported. It is
  now timed directly, from the instant the extension hands the dict back. `wall_ns` is the
  public call end to end, and `wall_ns >= native_ns + conversion_ns + model_ns` is asserted
  rather than assumed.

  The shared type was the other half, and it was the worse half. `Work` described a whole
  public call while the same type carried each projection's engine-local span, so `wall_ns`
  meant two things and the smaller silently answered questions asked about the larger. New
  `ProjectionCost` -- entries, dirs, rows, tally rows, name bytes, and `engine_ns`. No
  `lock_wait_ns`: the projections waited together, so attributing one wait to one of them
  would be inventing a number, and the type now says that instead of a zero saying it.

  FINDING 2, THE PAYLOAD RULE WAS NOT APPLIED EXHAUSTIVELY -- and the fix is structural
  rather than a list of patches. Every emission on the read path now goes through
  `put_scalar` / `put_text` / `put_nested`, which charge before they set. The old shape --
  `set_item` calls followed by a separate `payload::scalars(n)` -- is an unenforced
  convention, and it drifted exactly the way those do: a count that had been six was still
  six when the shape had gone to four, and to seven. A `Charge` impl on `Option` fixes the
  null family, which the stated rule had always covered and the code had charged zero for.

  THE TEST IS AN ORACLE, NOT AN EXPECTATION. A second implementation of the documented rule,
  in Python, walks the native dict and must agree with what the binding charged. A
  per-family expected figure only covers the family somebody wrote it for, which is how this
  drifted twice. Eleven request shapes plus an analysed index, chosen so that every branch is
  reached: present and absent remainders, present and absent section bounds, a bounded tree,
  an absent extension, an unclassified row, an absent roll-up, and the metric families --
  which exist only on an analysed index, so a shape list over a plain one never reaches
  `metric_row_dict` at all.

  It found a live defect while being written: three `bound` nulls, one per bounded section,
  emitted and uncharged. Mutation-checked against an over-charge as well -- restoring the
  four-fields-charged-as-six that shipped here fails it. The first attempt at that mutation
  *passed*, because no shape produced an extension remainder; that gap is why the shape list
  is what it is.

  Two carve-outs, stated because a number nobody can reproduce is not evidence. `work` and
  `projections` are telemetry about the call rather than part of the answer. Keys of maps
  keyed by data -- `by_extension`, `by_group`, and the three detection maps -- are payload;
  schema keys are not, since counting them would make the figure grow with the schema.

  FINDING 3, THE GIL TEST IS NOW FORCED RATHER THAN TIMED. CI disproved the threshold
  version on macOS/3.14 exactly as predicted: 0.0070s against a required 0.0080s, failing
  for being right. Two intermediate versions are worth recording as dead ends. Deriving the
  probe delay from a measured warm baseline still left a statistical margin. Reading the
  sampler's cadence to detect "too coarse to measure" was worse than useless when measured
  during the read -- a held GIL slows the sampler, so it reported the instrument as too
  coarse for the one condition the test exists to catch.

  What works has no timing in it. A contending thread busy-loops, yielding each iteration;
  `sys.setswitchinterval(5.0)` stops the interpreter handing the GIL away *voluntarily*, so
  between the two `len()` calls the main thread cannot yield -- while a GIL the native read
  genuinely releases is handed over immediately, interval or not. The count separates the
  cases exactly, at any speed. Mutation-checked: removing `py.detach` yields **0** turns
  during a call that visited 20,021 entries.

  Also fixed: `_native.pyi` had never carried `read`'s report parameters, and had `views`,
  `depth`, and `limit_rows` typed as the wrong scalar kinds.

  CPU stays `None`, and `py.detach` was correct as reviewed.
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
