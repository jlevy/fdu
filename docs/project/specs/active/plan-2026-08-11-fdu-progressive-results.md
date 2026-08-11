# Feature: Progressive Results — Traversal Order, Streaming Sessions, and Instant Warm Opens

**Date:** 2026-08-11

**Author:** fdu project

**Status:** Draft

## Overview

Make fdu usable by a consumer that needs answers *while* the walk runs and *instantly*
on the second open, rather than only after a complete scan.
This is the architectural work an interactive browser over a multi-million-entry tree
requires, and it is independent of the FSEvents journal: everything here lands on every
platform, helps the first scan as much as the second, and is a precondition for the
journal being worth anything rather than a consequence of it.

Three changes, in dependency order:

1. **Traversal order as a policy** with breadth-first by default, so partial results
   mean something. *Landed with this plan.*
2. **A streaming session API** so a caller can start a scan, get control back, and read
   growing roll-ups with per-path completeness while the walk proceeds.
3. **Persisted roll-ups and lazy open** so the second open paints from the cache in
   milliseconds instead of materialising millions of records first.

The motivating measurements, all on this host: a home folder of **4,366,510 files and
1,016,449 directories, 224 GiB** walks cold in **791 seconds**, and a warm snapshot of
that size would take roughly **11 seconds** just to load at today’s ~2 µs/record.
Neither number is compatible with an interactive first paint, and no amount of walking
faster fixes either one on its own.

## Goals

- A consumer reading the index mid-scan sees monotonically improving answers, never a
  confidently wrong ranking
- Traversal order is chosen by the caller’s contract and provably cannot change the
  resulting index or invalidate a cache
- Starting a scan returns control immediately; roll-ups and per-path completeness are
  readable throughout
- A warm open paints the top of a multi-million-entry tree without materialising it
- Every mechanism degrades to the current behaviour rather than replacing it

## Non-Goals

- The FSEvents journal, which is
  [its own plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md).
  This plan is what makes that one worth having; it does not depend on it.
- Content-tier reducers (line counts, hashes).
  The derived-data layer’s shape is fixed by the composable-CLI plan; nothing here ships
  an analyzer.
- A TUI or any UI. This is engine and API surface for consumers that have their own.
- Changing what a one-shot CLI prints.
  Its contract — verify, then print — is unchanged.

## Background

Two facts, both measured, set the shape of the problem.

**Sizes are recursive, so only a cache can answer quickly.** Listing a home folder’s
immediate children takes about a millisecond, but knowing one of them is 53 GiB means
having walked all of it.
A first paint therefore either shows skeletons or reads something that already knows —
there is no third option.

**Partial results are only useful if the order is right.** fdu already streams
internally: the parallel producer emits observation batches as it walks and
`merge_upward` keeps every ancestor’s roll-up current, so `index.rollup(path)` is
already a valid lower bound mid-scan.
But every traversal loop in the crate was a LIFO stack, so those lower bounds arrived
depth-first — one child of the root complete at its final size while its siblings read
zero. A consumer ranking by size mid-scan ranked confidently and wrongly.

metabrowser’s Python walker independently derived the fix years of engine work earlier:
it queues breadth-first to a first-render depth “so a request landing ~500 ms into boot
finds the visible part of the tree already populated”.
It also derived post-order finalize by refcounting children — the same design fdu’s
roll-ups use — and a `max_files` cap of 500,000 that exists only because a Python walker
cannot go further. The full analysis of that comparison and of how the four consumers
differ is in the
[interactive browser research](../../research/research-2026-08-11-interactive-browser-use-case.md).

## Design

### 1. Traversal order as a policy (landed)

```rust
pub enum ScanOrder { BreadthFirst, DepthFirst }   // default: BreadthFirst
```

Both orders visit every entry exactly once and leave an identical index behind, so the
policy sits beside `threads` and `batch_size` as operational, and stays out of
`ScanScope` where it could otherwise invalidate a snapshot.
All four traversal loops — the serial walk, the parallel worker queue, the revalidation
sweep, and subtree reconciliation — take from the front or the back of one `VecDeque`
according to the policy; there is no second walker.

Measured on a 59,654-entry tree over six interleaved trials each: breadth-first costs
**~8% on a complete scan** (51.0 ms against 47.2 ms) and **nothing measurable in
memory** (11 MB either way), because the queue holds directories and this tree has 7,341
of them. Three tests pin the contract: identical engine digests across both orders and
several worker counts, non-decreasing directory depth in emission order under
breadth-first, and scope equality between the two orders.

Eight percent is the right default price: it buys the difference between partial results
that are useful and partial results that mislead.
Consumers that only read the finished index — the one-shot CLI, batch jobs — select
`DepthFirst` and take it back.

### 2. Streaming session API

The engine already produces what a browser needs; what is missing is a way to hold it
while it is being produced.

```rust
let session = fdu::Session::start(root, config)?;   // returns immediately
loop {
    let view = session.report(&query)?;             // roll-ups + freshness, any time
    render(view);
    if session.is_complete() { break; }
}
```

- **Reads during writes** already exist: `IndexHandle` serves readers while a producer
  applies short writes, and that is exactly the shape a session needs.
- **Completeness is already modelled**: `Freshness` distinguishes `Fresh` from
  `Partial`, and `freshness_marks` is keyed by path, so a consumer can ask whether a
  directory’s total is final or still growing and render “calculating…” accordingly.
- **The contract is monotonicity**: under `BreadthFirst`, every roll-up a consumer
  observes is a lower bound that only increases until the subtree completes.
  That is the property the API documents and tests, and it is what makes a progress UI
  honest.
- **Python mirrors it**, since metabrowser is the driving consumer and a subprocess
  boundary would defeat the point.

Cancellation and bounded memory matter here in a way they do not for a one-shot: a
session must stop promptly when a user navigates away, and a browser should be able to
cap resident entries.
Both are session-level concerns, not engine ones.

### 3. Persisted roll-ups and lazy open

The second open is where a browser should feel instant, and today it cannot: loading a
5.4M-entry snapshot costs ~11 seconds because every record is replayed through apply and
every reducer is recomputed.

This is the existing backlog — H33 (persist per-directory reducer state), H34 (bulk
arena load), H16 (answer from roll-ups without materialising the index), H35 (block
format with a tail index and lazy decompression, bead `fdu-1vd0`) — whose priority this
use case sharply raises.
The browser needs exactly what they provide: read the header and the top-level directory
records, paint, and load deeper blocks only as the user navigates.
A browser displays a few hundred entries at a time and never needs millions resident.

### How the three compose

|  | first open (cold) | second open (warm) |
| --- | --- | --- |
| first paint | skeletons immediately; breadth-first fills top-level totals within the first seconds | persisted roll-ups read from the snapshot header — milliseconds |
| convergence | the walk continues; totals grow monotonically | verify what changed (journal where available, else a sweep or rescan by the adaptive policy) |
| navigating deeper | already walked, or walking | lazy block load per directory |

The journal appears once, in one cell, which is the point of separating these plans.

## Implementation Plan

### Phase 1: Order and session

- [x] `ScanOrder` policy, breadth-first default, all four traversal loops, tests for
  index equality, depth monotonicity, and scope stability
- [ ] `--order` on the CLI and the probe; the one-shot CLI defaults to `DepthFirst` for
  its 8% since it prints nothing until the end
- [ ] `Session`: start/read/complete/cancel over `IndexHandle`, with documented
  monotonicity and per-path freshness; bounded-memory option
- [ ] Python `Session` mirroring the Rust surface
- [ ] Loop experiment: time-to-useful-top-level-ranking, breadth-first against
  depth-first, on a home-folder-scale tree — the metric this plan exists to move

### Phase 2: Instant warm open

- [ ] Persist per-directory reducer state in the snapshot (H33)
- [ ] Bulk arena load, no per-record observation replay (H34)
- [ ] Answer a query from persisted roll-ups without materialising the index (H16)
- [ ] Block format with tail index and lazy decompression (H35, `fdu-1vd0`)

## Testing Strategy

Order equivalence is the load-bearing correctness property and is already tested:
identical engine digests across orders and worker counts.
Monotonicity gets a property test — sample roll-ups repeatedly during a scan of a
fixture tree and assert no observed total ever decreases.
Session tests cover start/cancel determinism and that a partial read is labelled
partial. The real-tree harness gains a time-to-useful-ranking job, since a plan about
*when* answers arrive cannot be validated by a benchmark that only measures when they
finish.

## Open Questions

- Peak queue width for breadth-first on a tree with a very wide level (a home folder has
  ~1M directories); measured at 60k, unmeasured at scale, and the answer decides whether
  a hybrid — breadth-first to a first-render depth, then depth-first below, as
  metabrowser does — is worth the complexity.
- Whether the one-shot CLI should really default to `DepthFirst`, given that a future
  `--progress` would want the opposite.
- How a session should bound memory: entry cap, depth cap, or eviction.

## References

- [Interactive browser research](../../research/research-2026-08-11-interactive-browser-use-case.md)
  — the four-consumer comparison and the measurements behind this plan
- [Performance frontier research](../../research/research-2026-08-10-performance-frontier.md)
  — H16/H33/H34/H35, the verification tiers, and the cost model
- [Composable CLI and query surface plan](plan-2026-08-10-fdu-composable-cli-surface.md)
  — `Query`/`Report`, which a session returns
- [FSEvents-scoped revalidation plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md)
  — the convergence half, deliberately separate

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
