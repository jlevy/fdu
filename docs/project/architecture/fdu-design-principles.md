# fdu Design and Principles

**Date:** 2026-08-12

**Author:** fdu project

**Status:** Active

## Overview

fdu summarizes directory trees: sizes, counts, recency, and file types, rolled up for
every directory at once.
This document covers the engine’s shape and the rules it holds itself to.

Each rule exists because breaking it produces a specific failure: a cache that lies, a
benchmark that measures the wrong job, or a number a consumer cannot calibrate.
The reasoning matters more than the rule, so each one states what goes wrong without it.

For what is built and what comes next, see
[the phase-1 plan](../specs/active/plan-2026-08-08-fdu-phase-1.md).
For where the design comes from and which prior art each piece draws on, see
[the file roll-up engine research](../research/research-2026-08-06-file-rollup-engine.md).

## Architecture

Three artifacts and one contract:

- **Index** (`index.rs`): in-memory parent-pointer tree.
  Every directory carries pre-computed roll-up state.
- **Snapshot** (`snapshot.rs`): the index serialized, invalidated wholesale by an engine
  fingerprint.
- **Delta** (`types.rs`): a typed, clocked change, and the only way the index or the
  cache is ever modified.
  Producers submit `Observation`s; the index arbitrates them and mints `AppliedDelta`.

`scan.rs` and `watch.rs` are delta *producers*. `index.rs` and `snapshot.rs` are delta
*consumers*. Nothing else mutates state.

A cold scan establishes a historyless baseline.
A reconciliation sweep conditionally applies its diff while it walks.
The watch layer coalesces event hints and verifies them by `stat`. The index alone
arbitrates observations, removes no-ops, advances the clock, and mints `AppliedDelta`.

### Serving Model

`open()` is deliberately blocking: it loads a usable snapshot and completes a filesystem
reconciliation before returning.
It never serves the snapshot as fresh before that pass, and never replaces a complete
snapshot with a partial result.

`IndexHandle` and the reconciliation APIs support readers between applied batches, with
explicit `Fresh`, `Reconciling`, `Stale`, and `Partial` state, but an application must
opt into that serving model.

The watcher is an adapter and driver.
`open()` and the Python API do not start it, and the Python wheel does not compile the
watch dependency.
Its applying driver re-verifies queued samples at a clock-stable commit
boundary and accepts only an unbounded, cross-filesystem scope; bounded-depth and
one-filesystem event filtering fail explicitly rather than indexing excluded paths.

A watch sample is valid at its filesystem `stat` point.
The process does not pretend it can freeze external mutation until the in-memory commit.
Backend events arriving during or after verification stay queued for the next batch,
while reported loss or ambiguity invalidates and reconciles the affected scope.
The logical-clock check prevents an older sample from overwriting a newer commit; it is
not a filesystem transaction.

### Concurrency Guards

Conditional observations carry generation and revision guards.
Present-state ABA, parent replacement, and absent create and remove races are rejected
at one batch boundary, without making changes in unrelated subtrees conflict.

Cold scans and every warm mutation path enforce the same semantic scope.
Depth zero is root-only, and subtree reconciliation refuses paths below depth or
filesystem boundaries, or through symlink ancestors.

## Data Structures

### Partial-Friendly as Well as Delta-Friendly

A partially walked tree is a valid, useful answer as long as the boundary of
incompleteness is knowable: roll-ups are correct lower bounds, unvisited subtrees are
identifiable, and per-value provenance carries `status: Partial`. Queries, sessions, and
reducers accept partial structures as first-class inputs.
Code that requires completeness must demand it explicitly, never assume it.

The two properties compose: a delta stream applied to a partial structure yields another
valid partial structure.
That composition is what progressive results are.

Serialization is the exception, because no format has been designed for it yet, not
because partial snapshots are unwanted.
Saving rejects a non-fresh index: there is no encoding for an unfinished frontier,
unknown children, evicted nodes, or a cancelled walk, and inventing one silently would
produce a snapshot that reloads as if it were complete.
Until a format version carries a completeness boundary, `save` demands a complete index
in its signature and says so in its error.

`Status::Partial` records *coverage*, not direction.
A value is a monotone lower bound only while an additive walk is running; one truncated
by errors can move either way.

### No Mutation Path Bypasses `Delta`

The contract keeps the in-memory structure, the serialized form, and the change feed
from drifting apart.
A new producer emits deltas; it does not reach into the index.

### Never Size an Allocation from Untrusted Input

Snapshot and journal parsers check declared counts against the bytes actually present
before allocating. A corrupt file must fail closed, not abort on an allocation.

## Trust and the Cache

### The Cache May Never Silently Lie

Fingerprints are size, mtime, ctime, and inode, not mtime alone.
mtime is user-settable and some applications roll it back after writing; ctime is
kernel-controlled. All observed stat fields are still compared when updating stored
state, so allocated-byte or device changes cannot leave query results stale.

A corrupt or unrecognized snapshot is treated as absent, never as data.
Failing closed costs a rescan; failing open silently corrupts every answer built on it.
The bootstrap format verifies its payload checksum before parsing records, and Unix
cache files are created owner-only because they contain a filesystem inventory.

Producers that lose precision escalate with `InvalidateSubtree` rather than guessing.

### Trade Speed for Certainty in the Open, Never in Secret

A verified answer over a huge tree costs minutes and a cached one costs milliseconds, so
the trade is legitimate and often necessary.
It is only honest when every value carries its provenance: where it came from, when it
was observed, and whether it is final.

Label per value, not per run.
A consumer rendering a thousand rows needs to know which of them to trust.
Anything that returns a number without that context is the silent lie the rule above
forbids.

### The Journal Bounds Uncertainty; It Does Not Replace Verification

A change journal’s value on a multi-million-entry tree is that it identifies *where the
imprecision could be* in milliseconds, which a full walk can only do in minutes.
That is what makes near-real-time visibility possible at that scale.

It composes with the rule above: the journal narrows what must be checked, the walk
remains the thing that checks it, and provenance records which of the two answered.

A journal can omit history without saying so.
macOS FSEvents reports `HistoryDone` after silently dropping events, so journal-derived
values are labelled `Source::JournalScoped`, never verified, and age bounds and periodic
sweeps are risk controls rather than correctness gates.

## Performance

### Fast Without the OS’s Help; Faster with It

Every platform API is an optional accelerator layered on a portable path that is already
fast on its own, and the portable path is what correctness depends on.

Three tiers, in order: an explicit walk that is quick by itself, a cache that is quick
where a cache can help, and OS-specific enhancements that make the first two cheaper
where the platform offers them.

A feature that is unavailable, disabled, or degraded falls back to the tier below it and
loses speed, never accuracy.
`getattrlistbulk`, `statx`, `io_uring`, fanotify, and the FSEvents journal are all
probe-and-fallback, never load-bearing.

### Claim Only What the Benchmarks Have Shown

The current walker is a portable `read_dir` and `symlink_metadata` implementation, and
is explicitly scaffolding.
Goal 1 is not met until the `getdents64` and `statx` layer replaces it *and* the
benchmark gate against dut and gdu passes.

Benchmarks report cold and warm separately, and raw-walk and with-stats separately, or
they compare different jobs.

The cache is not assumed to be a speed-up.
Its benefit depends on platform and on which reducer tiers a view uses, as measured in
[the performance frontier research](../research/research-2026-08-10-performance-frontier.md).
A warm path that loses to a cold scan of the same view is a defect, not a trade-off.

Speed changes are decided by measurement, never by argument.
A change is kept only when the median improves at least 3% *and* the 95% interval lies
entirely below zero.
The protocol is [the performance loop](../guides/performance-loop.md); every verdict,
including the failures, is in
[the experiment ledger](../reports/report-2026-08-10-fdu-performance-experiments.md).

### Say What Blocked, Not Just How Long It Took

A profile reporting one undifferentiated “blocked” number cannot answer the question
that decides what to optimize.
Measurements attribute time to disk I/O, CPU, lock and coordination wait, consumer
handoff, or idle.

Coordination is measured in chunks, per claimed run of work rather than per file, so the
instrumentation obeys the amortization rule it exists to verify.

## Boundaries

### The Watch Layer Stays Deletable

It sits behind a feature flag and is strictly additive: removing it leaves scan, index,
snapshot, CLI, and Python surfaces working.
The index must never learn what a filesystem event is.

### Two Crates, Not More

`fdu` is the library and CLI. `fdu-py` exists only because a cdylib cannot also be the
crate Rust consumers depend on.

Module boundaries are free; crate boundaries cost a version number, a publish, and a
semver promise each.
Extract a module into a crate when an external consumer exists, not before.

### GPL-Derived Designs Are Clean Reimplementations

`dut`’s atomic-refcount roll-up and `fsearch`’s record layout are described in
[the file roll-up engine research](../research/research-2026-08-06-file-rollup-engine.md)
and are written from those descriptions, not transliterated from their source.

## Code

Changed code carries complete type annotations.
Catch only errors the current layer can handle, and preserve exception causes.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
