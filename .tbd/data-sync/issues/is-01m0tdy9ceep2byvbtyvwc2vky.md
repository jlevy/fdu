---
type: is
id: is-01m0tdy9ceep2byvbtyvwc2vky
title: Release the GIL and measure the full Python read boundary
kind: task
status: closed
priority: 1
version: 7
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
updated_at: 2026-08-25T00:13:37.667Z
closed_at: 2026-08-25T00:13:37.667Z
close_reason: |
  Closed. `make check` green. All three follow-up findings addressed.

  P1-A, ONE END-TO-END WALL. `conversion_ns` was stamped before the projection dicts were
  built, so it excluded part of the conversion it claimed to measure; it is stamped last
  now. More importantly the public `Index.read` builds dataclasses from the extension's
  dict, and none of that was measured at all -- so publishing native `wall_ns` as the call's
  duration omitted a provider-specific phase and understated this path against a
  pure-Python provider that has no such phase.

  `Work` now names every phase: `native_ns` (engine, GIL released), `conversion_ns`
  (building the extension's dict), `model_ns` (dict to dataclasses), and `wall_ns` as the
  end-to-end public call, measured in `_api.read`. The adapter maps `wall_time_ns` from
  `wall_ns`. `wall_ns`'s docstring said "entering the call to returning" while ending before
  both conversion phases; it says what it is now.

  P1-B, BYTES COUNTED RATHER THAN ESTIMATED. `materialized_bytes` is gone. It described
  itself as rough, re-walked the finished bundle, and counted names plus six assumed scalars
  per row while omitting tags, classifications, provenance, extension and group tallies, and
  envelope errors -- so a result could grow substantially without the counter following.

  One rule, stated once on the `payload` module: every string and path *value* contributes
  its own bytes, every fixed-width leaf contributes eight, and dict keys are excluded as
  fixed schema rather than payload. Accumulated at each conversion site -- rollups and their
  tallies, child rows with names, tags and classifications, provenance, tree nodes, report
  sections, scope, cursor, envelope errors -- so there is no second traversal of the hot
  boundary. A thread-local rather than a threaded parameter, because the conversion runs
  through a dozen nested helpers that other paths also call and passing an accumulator would
  change every signature to serve one caller; `read` clears before it starts, and the GIL
  makes it single-threaded by construction.

  The test pins an exact delta rather than `> 0`: adding one roll-up must cost exactly six
  scalars plus each extension and group key with three scalars, computed from the roll-up
  the test itself reads back. A counter that only has to be positive can omit whole field
  families and still pass -- which is precisely how the first version drifted.

  P2, A GIL TEST THAT ACTUALLY DISCRIMINATES. The old one started four readers and checked
  they finished, which the review correctly called vacuous: serialized reads finish too.
  Getting a real one took three attempts, and the first two failed the mutation check --
  worth recording, because both looked convincing:
    1. Release a probe thread, then read; assert its timestamp precedes the return. The
       probe fired *before* the read even began.
    2. Same, with 400k `rollups` to lengthen the call. `_api.read` builds that list in
       Python bytecode, so the probe got its turn during argument construction rather than
       during the native span.
  The version that works is sized from measurement rather than guessed: 20k files make a
  filtered summary take ~10ms of which ~99% is native. The probe sleeps 2ms (releasing the
  GIL, so it always wakes on time) and timestamps (which needs the GIL back), and the
  assertion is that it got its turn in the *first half* of the call -- not merely before the
  return, which is nearly true even when the GIL is held, since the thread runs the instant
  the call gives it back. Removing `py.detach` fails it deterministically: "the probe waited
  0.0118s of a 0.0120s call".

  CPU stays `None`, confirmed by the owner as exact-or-absent with no sentinel and no
  adapter shim.
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
