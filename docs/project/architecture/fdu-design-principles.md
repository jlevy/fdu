# fdu Design and Principles

**Date:** 2026-08-12

**Author:** fdu project

**Status:** Active

## Overview

fdu summarizes directory trees: sizes, counts, recency, and file types, rolled up for
every directory at once.
This document covers the engine’s shape, the command-line and query surface built over
it, and the rules both hold themselves to.

Each rule exists because breaking it produces a specific failure: a cache that lies, a
benchmark that measures the wrong job, or a number a consumer cannot calibrate.
The reasoning matters more than the rule, so each one states what goes wrong without it.

For what is built and what comes next, see
[the phase-1 plan](../specs/active/plan-2026-08-08-fdu-phase-1.md).
For where the design comes from and which prior art each piece draws on, see
[the file roll-up engine research](../research/research-2026-08-06-file-rollup-engine.md).

## Architecture

The metadata core has two retained artifacts, one transient answer, and one mutation
contract. Explicit content analysis adds two separately invalidated derived artifacts:

- **Index** (`index.rs`): in-memory parent-pointer tree.
  Every directory carries pre-computed roll-up state.
- **Snapshot** (`snapshot.rs`): the index serialized, invalidated wholesale by an engine
  fingerprint.
- **Delta** (`engine_contract.rs`): a typed, clocked metadata change, and the only way
  the metadata index or snapshot is ever modified.
  Producers submit `Observation`s; the index arbitrates them and mints `AppliedDelta`.
- **Content index** (`content/content_index.rs`): optional sparse file records and
  pre-computed content roll-ups, allocated only for an enabled analysis profile.
- **Content sidecar** (`content/content_cache.rs`): profile-scoped persistence for the
  derived content tier, never loaded by metadata-only requests and never embedded in the
  metadata snapshot.
- **Derived report plan** (`execution.rs`): the minimum transient state sufficient for
  one complete one-shot request when no cache, live session, or later query can consume
  an index. It produces a `Report`, never a hidden cache or second query grammar.

`scan.rs` and `watch.rs` are metadata-delta *producers*. `index.rs` and `snapshot.rs`
are metadata-delta *consumers*. Content workers submit independently fingerprint-checked
analysis observations through the index’s derived-data boundary; they do not advance the
metadata clock or alter snapshot truth.

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

## The Command-Line and Query Surface

The rules the command line, the query layer, and the library hold themselves to, as
actually implemented.
A change that violates one of these needs this document amended first, not a silent
exception.

Distilled from
[the composable CLI and query surface plan](../specs/active/plan-2026-08-10-fdu-composable-cli-surface.md)
after building it. Where implementation forced an amendment, the amendment is recorded
here rather than the original intent.

The governing aspiration: the design should fit the contours of the real problem — no
more complexity, but no less either.
Simple things stay simple (`fdu .` is a good answer) and complex things stay possible,
because any axis composes with any other.
Bare `fdu` is safe discovery: it prints help and never assumes that the current
directory, which may contain millions of entries, was meant to be scanned.
The concrete test for “no more complexity”: before adding a view or a flag, show it
cannot be expressed as a composition of what exists.
`largest` and `recent` were removed from the design by exactly that test; they are
`--view files --sort size --limit N` and `--view files --modified-since 2h`.

### Five Axes, No One-Off Flags

Every option belongs to exactly one axis:

| Axis | Question | Options |
| --- | --- | --- |
| Scope | What is scanned and cached? | `PATH`, `--scan-depth` |
| Selection | Which retained entries does this query consider, and how are results shaped? | `--include`, `--exclude`, `--min-size`, `--modified-since`, `--modified-before`, `--kind`, `--depth`, `--limit`, `--sort`, `--reverse`, `--size` |
| View | Which roll-up is reported? | `--view tree,extensions,types,families,languages,documents,files,summary`, `--words-per-page` |
| Format | How is it serialized? | `--format`, `--color` |
| Mode | One answer or a live feed, how is the cache used, and is content read? | `--watch`, `--interval`, `--cache`, `--analyze`, `--analysis-workers`, `--allow-partial` |

A proposed flag that fits no axis is a design smell: either it generalizes into an axis
value, or it does not ship.

**Scope versus selection is the load-bearing distinction.** Scope decides what is
observed and cached, so one snapshot serves every query; selection filters the retained
index at view time and is never part of the cache key.
That is why narrowing a filter never costs a rescan, and it is the same reasoning as
tagging ignored entries rather than pruning them.

### Intuitive by Default, Everything by Composition

There are no subcommands.
The grammar is always “report on a path”, so a path argument can never be shadowed by a
verb. A report requires that path explicitly; `fdu .` is the short opt-in to the current
directory, while bare `fdu` prints help without touching the filesystem.
`--help` documents each axis, its values, and its defaults plainly enough that the
design is legible from the help text alone.

### One Scan, Many Views

Views are projections over one consistent scanned state.
The reusable form of that state is the in-memory index: requesting more views never adds
filesystem work, and two reports over the same index cannot disagree about when the tree
was observed. For a one-shot request that proves no cache, live session, second view,
filter, or later query can consume hierarchy, an internal execution planner may retain
an exact aggregate instead.
It derives that decision from the complete request, exposes no fast-mode flag, and falls
closed to the full index when any requirement is unproved.

Within metadata report evaluation, two query-cost tiers follow from this, and both are
milliseconds warm: an unfiltered request reads pre-computed roll-up state directly,
while any selection filter triggers one traversal that re-aggregates what it admits.
One traversal serves every filtered view in a request.
A test pins that the two tiers answer identically when the filter admits everything.
An additional golden and semantic-hash gate pins that a derived summary serializes
identically to the indexed summary.

Content analysis adds a third, explicit I/O tier without changing either metadata tier.
No analysis profile means no regular-file content opens, analyzer workers, sparse
content index, or content-sidecar load.
Any enabled `--analyze` profile retains the full metadata index, then streams every
eligible file absent from the matching sidecar through EOF. Worker count bounds
concurrency, never per-file coverage.
`languages` is a metadata-only byte and count view by default; the code analyzer adds
standard LOC. `documents` requires at least the basic analyzer.
A view never enables an analyzer implicitly or presents an unmeasured value as zero.

Expected content coverage is distinct from operational completeness.
Binary data, invalid UTF-8, and file types without a requested analyzer remain explicit
coverage outcomes while metadata file and byte totals stay complete.
I/O failures, a file that changes while being read, and stale conditional commits make
the content operation partial and must be surfaced as errors.
An implementation may stop reading once a file is proven binary, but it never truncates
or size-skips an eligible text file.

### Views Are Readers

The delta contract stands: `scan` and `watch` produce observations, the index consumes
them, and views only read.
`report()` is a pure function of an index, a query, and provenance — no filesystem
access, no mutation, and the same inputs always produce the same report.
The one-shot execution planner sits before that reader boundary: it chooses what state
must be retained, then constructs the same immutable `Report` shape.
It never changes `report(index, query, provenance)` or lets a view reach into the
filesystem.

*Amended during implementation.* The plan sketched `report(index, query)`. Provenance is
a third argument because `generated_at` cannot be sampled inside a pure function; making
it an input is what keeps the goldens meaningful.

### Fastest Answer the Data Allows, Never Silently Stale

Cache behavior is one explicit policy axis, and every report labels its `source`,
`freshness`, `complete`, and `errors` in every format.
Warm, cold, and cache-only runs are user choices rather than heuristics.

`--cache only` is the one tier that can be stale, and it says so: the loaded index is
marked unverified rather than replaying the freshness it was saved with.
It fails when no usable snapshot exists rather than silently scanning, because a fast
path that is sometimes a full walk — with nothing in the output to say which happened —
is worse than no fast path.

See [the cache design](../guides/cache-design.md) for the two layers and what
verification costs.

### Same Concepts at Every Level; the CLI Invents Nothing

`Query`, `Selection`, `ViewSpec`, `Report`, and `CachePolicy` are typed values in the
library. The CLI parses flags into them, renders what comes back, and does nothing else.
Python exposes the same types through the same value grammars.

The parity rule is mechanical: a capability reachable by flag must be reachable as one
typed call, with the same defaults.
A capability that exists in one surface and not the others is unfinished, and complexity
that exists only at the CLI layer is misplaced.

What legitimately lives only in `cli.rs`: flag parsing, terminal and colour decisions,
exit-code mapping, and the human text layout.
Everything else — value grammars, selection semantics, view construction, cache policy,
session coordination — is library code.

### Subsume the Neighbours

Each of these must be one invocation:

| Instead of | Run |
| --- | --- |
| `dust`, `dut` | `fdu PATH` |
| `du -sh`, `diskus` | `fdu --view summary PATH` |
| `du -a --max-depth 3` | `fdu --depth 3 -n all PATH` |
| `fd -e rs`, `find -name` | `fdu --view files --include '*.rs' PATH` |
| biggest files | `fdu --view files --sort size -n 100 PATH` |
| `find -mmin -60` | `fdu --view files --modified-since 1h PATH` |
| `du` by type | `fdu --view types PATH` |
| two reports, one scan | `fdu --view types,tree PATH` |
| `tail -f` for a tree | `fdu --watch --view files --format jsonl PATH` |

An interactive TUI is a recorded non-goal, not an omission: it would be a consumer of
the same `Query`/`Report` layer.

### Formats Are Serializations, Not Features

Every view renders in every format.
Machine formats are schema-versioned, never colourized, and a schema change without a
version bump fails a golden test.

This principle inverted a rule that used to exist: `--by-type` conflicted with `--json`,
because the type breakdown was human-only.
Under the axis design that combination is not merely legal but required to work.

One-shot human text has one intentional presentation-only suffix: a compact performance
line after the report.
It is transient execution telemetry, not query data, so it stays outside `Report` and
the versioned JSON, JSONL, and YAML schemas.
The line records regular files and apparent bytes successfully walked, bytes actually
returned by fresh content reads, content-analysis file and byte throughput,
content-sidecar hits and the apparent bytes they represent, the metadata cache tier, and
total report time. A cache-only answer reports zero walked files rather than pretending
cached inventory was filesystem work.
Watch has no final answer and therefore has no footer.
Terminal text renders the footer in gray; uncolored text contains no escape sequences.

### Watch Is the Same Query, Repeated

A watch run evaluates the same selection and views as a one-shot run, re-applied as
changes arrive. There is no separate watch grammar to learn.

Detection is event-driven — the OS notification backend, never polling — so an idle tree
costs no filesystem work, a property asserted by test rather than described.
`--interval` throttles only how often aggregate views repaint; it plays no part in
detection. Overflow and subtree invalidation appear explicitly in the stream and are
never dropped, because they say the consumer’s own view may have gaps.

Two deliberate asymmetries in filtering: a removal is filtered only by path, since
filtering a deletion on a size bound would hide the disappearance of something the
caller was watching; and an escalation is never filtered at all.

### Utilities Are Explicit Flags, Never Side Effects

`--cache-status` and `--cache-clear` run before scan validation, need no readable tree,
and suppress the report.
A missing path is allowed only for these lifecycle operations and discovery surfaces; it
never creates an implicit report scan.
A report run never deletes anything.
Clearing echoes its target before acting, and never removes a file this build cannot
identify.

### Every Output Surface Is a Benchmark Job

Each output surface becomes a named benchmark job, and flags are part of benchmark
identity: renaming one means updating the job manifests in the same change.
The measurement rules themselves are under [Performance](#performance).

### Golden Tests Are the Text Contract

Each golden is a compact end-to-end product example: **minimal fixture and transcript,
maximum critical surface, realistic enough that a person can judge the experience.**
Every committed row or command must protect a distinct contract, but surgical snippets
are not a substitute for showing the complete output a user sees.
Focused unit and property tests own combinatorial edge cases; a small number of
representative golden sessions own the whole invocation, stdout, stderr, exit status,
and relevant side effects.
This is the project form of the tbd golden-testing guidance: concise, broad, stable, and
reviewable.

Their value depends on one habit: **classify every field as stable or unstable.** Paths
inside the fixture, byte counts, entry counts, kinds, and schema strings are stable and
must match exactly. Sandbox paths, timestamps, allocated sizes, and inode-derived values
are unstable and get a *named pattern* — never a bare line elision, which would hide the
field instead of freeing its value.

Re-recording is normal; **reading the diff is the point.** In this workstream the
goldens caught four defects no unit test did, including JSON that was balanced and
invalid because the fixture had no directory with two children.
The default human report has its own realistic nested-project session so alignment, size
ranking, fixed ten-cell bars, default depth, rolled-up hidden descendants, and the
absence of spurious omission markers move together as one visible product contract.

Two hazards worth remembering, both found the hard way:

- Comparing roll-ups by raw interned extension id fails across walks.
  Ids are assigned in first-seen order, so serial and parallel runs assign them
  differently. Compare through `by_ext_named()`.
- Deep trees must be handled iteratively everywhere — expansion, all three renderers,
  and `Drop`. Derived drop glue recurses per level, so a deep tree overflowed the stack
  on release even after rendering was fixed.

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

The portable walker uses `read_dir` and `symlink_metadata`, which the Rust standard
library already implements on Linux as `getdents64` plus dirfd-relative `statx`. A
strace census on a 450,462-entry tree measured fdu, dut, and diskus issuing the same
counts of each, and a single-threaded harness found raw `getdents64` and narrow `statx`
masks within noise of the standard library, so a hand-rolled syscall layer is not what
remains between here and Goal 1 on Linux; see
[the Linux first measurements](../research/research-2026-08-13-linux-first-measurements.md).
Goal 1 is met when the benchmark gate against dut and gdu passes, which on that rig
means closing the index-consumer gap rather than the enumeration one.

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

User-facing throughput follows the same evidence rule.
Metadata walking counts files whose attributes were actually observed and their apparent
lengths.
Content I/O counts bytes returned by `read`, including partial binary probes and
successful bytes before a read failure; it does not substitute the file’s metadata
length. Fresh-analysis rates use the analyzer phase’s wall time, while cached records
report both hit count and the apparent bytes they represent.
These counters describe one run and are not benchmark claims.

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
