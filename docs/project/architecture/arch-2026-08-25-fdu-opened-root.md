# Architecture: fdu Opened Root

**Date:** 2026-08-25 (last updated 2026-08-25)

**Author:** fdu project

**Status:** In Review

## Overview

This document defines the intended architecture for fdu’s long-lived, progressively
discovered, incrementally updated directory inventory.
It is the design target for the opened-root rewrite, even where the current code still
implements only the blocking one-shot engine or the earlier watch-session prototype.

The opened-root API is additive.
The existing `Index`, blocking `open()`, report/query engine, command line, and Python
one-shot surface remain useful and keep their current defaults.
The new API gives an interactive client one root it can read while discovery proceeds,
poll for changes, refresh explicitly, reprioritize, and close deterministically.

This architecture document owns durable boundaries and invariants.
[The implementation plan](../specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md)
owns sequencing, file- and function-level work, prototype reuse, test gates, and beads.
If the two disagree on design, amend this document first and review the architectural
change before changing code.

## Goals and Non-Goals

### Goals

- Add one general synchronous engine API for progressive discovery, coherent bounded
  reads, resumable changes, verified refresh, scheduling hints, and deterministic close.
- Preserve the current one-shot engine and CLI defaults while sharing its facts,
  reducers, query vocabulary, runtime registry, and correctness gates.
- Give MetaBrowser a thin provider adapter without moving filesystem or aggregation
  ownership into the application.
- Keep optional observation and ignore support removable and hold dependency and binary
  growth to measured, reviewed additions.

### Non-Goals

- Mixing warm cached and cold verified facts in the first progressive implementation.
- Adding an async Rust runtime, network protocol, signed page-token format, generic tag
  system, or application query vocabulary to core.
- Changing default CLI behavior or making fdu the default MetaBrowser provider before
  cross-provider and installed-wheel acceptance passes.
- Guaranteeing the same retained prefix across concurrent providers after a discovery
  resource stop.

## System Context

The opened root is an additive lifecycle inside the authoritative `fdu-core` engine.
The Rust package and Python extension expose the same engine values.
MetaBrowser adapts the synchronous Python handle to its async provider boundary and
keeps browser, HTTP, root-generation, overlay, and SSE policy above it.

```text
filesystem
    |
    v
verified scan / refresh / optional observer hints
    |
    v
fdu-core Index -- exact Commit --> bounded history and coherent projections
    ^                                      |
    |                                      v
one OpenedIndex Owner <----------- read / changes / close
                                           |
                                           v
                                  synchronous Python binding
                                           |
                                           v
                              MetaBrowser async provider adapter
                                           |
                                           v
                             coordinator / routes / browser / SSE
```

The command line and existing one-shot Python API remain sibling consumers of the same
engine; they do not route through MetaBrowser or the new adapter.

## Design

### Architectural Commitments

#### One opened root has one authority

An opened root has one internal owner for its facts, reducers, version, lifecycle,
journal, continuations, discovery, observation, refreshes, and shutdown.
Public `OpenedIndex` clones share that owner.
They do not clone an independently mutable index while retaining the same session
identity, and no progressive session hands authority to a second watch session.

`close()` closes the shared owner.
The first caller begins cancellation and joined shutdown; concurrent callers wait for
the same terminal result.
Every clone immediately observes closing or closed state.
Dropping the last reference performs the same joined shutdown as a defensive fallback,
but ordinary clients close explicitly.
Owned worker closures hold a weak owner reference or narrower shared state, never a
strong `OpenedIndex` façade reference that could keep the last public handle from
dropping. `Drop` records but cannot return a shutdown error; explicit `close()` is the
path that returns the shared terminal shutdown result.

#### One exact commit is the truth consumers observe

Every verified fact change and every observable state transition lands through one
atomic commit path. A commit contains its version, exact effective changes, fdu-native
impact, terminal state, and work.

Producers submit verified observations or state transitions.
They do not describe consumer invalidations separately, copy requested operations into a
journal, advance a clock beside the mutation, or publish callbacks that reconstruct what
changed. Impact is derived from the exact effects recorded while the index and its
reducers are mutated.

#### Every answer is bounded and honest

An interactive read has an output bound and a deterministic work bound.
It returns one coherent engine version and state with every projection.
A limit that prevents an exact answer returns an explicit capped or query-limit result;
it never silently presents a partial calculation as exact.

Missing entries use three-valued knowledge:

- `present` when the entry is retained;
- `absent` only when the relevant directory coverage is complete;
- `unknown` when discovery, a resource stop, an inaccessible boundary, or portable-path
  loss prevents an absence claim.

#### A complete behavior session is inspectable

The opened-root value model is complete enough that a client or test harness can record
one causal session from real engine values: input options, operation requests and
responses, exact commits, coherent state, work, recovery, and shutdown.
It does not need a private mutation log, selected debug counters, or a second test event
vocabulary that can drift from the public behavior.

This is a design constraint, not only a testing preference.
If an important effect cannot be explained from the operation result, commit journal,
bounded diagnostics, and final read, improve those production values before adding a
test-only observation path.
Test seams may supply deterministic filesystem hints or pause at named scheduling
boundaries; they control when production behavior runs and never state facts, mint
changes, or bypass verification.

#### Optional mechanisms remain removable

The owner, exact commits, reads, explicit refresh, journal, and continuation table use
the standard library and build without native observation.
The `watch` feature supplies observation hints and nothing else.
Removing it leaves cold discovery, reads, changes caused by discovery or refresh, and
joined close working.

Ignore support is a fixed semantic capability behind an explicit removable feature if it
needs a dependency. Core does not acquire an async runtime, web stack, serialization
framework, or token signing dependency for the interactive API.

### Values and Ownership

#### Detached index

`Index` is an independently owned tree image for one-shot Rust callers.
It stores native path facts, reducers, classification state, control state,
completeness, and the metadata needed to produce a snapshot.
A detached index may retain bounded exact commit history for the existing nonblocking
`Index::since` API; that history has no session identity, waiter, worker, or
continuation authority and is never persisted.
A cloned `Index` is a separate value and carries no live session identity, worker,
condition variable, or continuation authority.

`IndexHandle` remains a short-write coordination primitive for existing reconciliation
and watch compatibility paths.
It is not an opened-root lifetime and does not become the owner of change polling or
shutdown.

#### Opened root

`OpenedIndex` is a small cloneable façade over one shared internal `Owner`. The owner
contains:

- the guarded current `Index`;
- one opaque session identity and monotonically increasing sequence;
- validated scope and semantic identities;
- lifecycle, coverage, freshness, progress, and bounded issues;
- the bounded commit journal and its condition variable;
- the bounded continuation table;
- the discovery frontier and scheduling hints;
- cancellation and every join handle that can write or wake a client.

The public Rust surface is synchronous:

```rust
impl OpenedIndex {
    pub fn open(root: &Path, options: OpenOptions) -> Result<Self>;
    pub fn read(&self, request: ReadRequest) -> Result<ReadResponse>;
    pub fn changes(&self, request: ChangeRequest) -> Result<ChangePoll>;
    pub fn refresh(&self, paths: &[RelativePath]) -> Result<RefreshResult>;
    pub fn prioritize(&self, paths: &[RelativePath]) -> Result<PriorityResult>;
    pub fn close(&self) -> Result<()>;
}
```

The names may follow established fdu vocabulary, but the five-operation cardinality,
synchronous boundary, and shared-owner semantics are fixed.
The associated constructor is deliberate: the existing free `fdu_core::open` keeps its
blocking one-shot contract, and Rust does not overload free functions by argument type.

#### Persisted snapshot

A snapshot is a detached representation, not a dormant live owner.
It contains facts, reducers, validated identities, and complete control state.
It never contains session identity, live version sequence, journal history, waiters,
continuations, worker state, or a discovery frontier.

The first opened-root implementation is cold-progressive only.
It does not serve a cached image concurrently with newly discovered facts under one
undifferentiated trust label, and it never persists a resource-stopped partial baseline.
Warm progressive serving requires a separate reviewed trust design.

### Configuration and Identity

`OpenOptions` separates answer semantics from execution policy.

Scope identity includes only facts that change what the live engine can observe:

- hidden-component admission and its exact-name allowlist;
- symlink-following behavior;
- filesystem-boundary behavior;
- admitted object kinds.

Maximum query depth is selection, not scope.
A discovery resource budget is execution policy, not a promise of a deterministic
cross-provider prefix.
Worker count, scan order, batching, scheduling hints, journal capacity, continuation
capacity, and observation mode are also execution policy.

Semantic identity covers the normalized runtime type registry, fixed ignore semantics,
and versioned reducer behavior.
The engine derives scope and semantic identities from validated values.
It never accepts a caller-supplied fingerprint as proof that different content matches.

The root binding is session-local and platform-native.
It is kept separate from portable cross-provider identity.

### Exact Commit Pipeline

All producers use this transition:

```text
verified facts or state transition
  -> scope, path, and control validation
  -> normalized prepared input
  -> atomic fact, reducer, control, and state application
  -> exact effective changes and derived impact
  -> Commit { version, changes, impact, state, work }
  -> bounded journal
```

Filesystem I/O and expensive parsing occur outside the write guard where possible.
The final prepared input is evaluated against current facts under the guard so a stale
preparation cannot publish an incorrect effect.
A conditional mismatch is a no-op or a retry signal, never a partial commit.

Mutation helpers record what they actually did.
Creating verified ancestors, replacing a kind, refusing a leaf after another mutation,
removing a control source, and changing only lifecycle state are all representable
without substituting the producer’s original request.
The version advances only when the complete commit is ready.

The public impact vocabulary stays fdu-native: topology, metadata, classification,
aggregates, and trust or lifecycle state, plus bounded dirty paths or `all_dirty`. An
application adapter may map those domains to its cache/query vocabulary; core never
stores application query names.

### Discovery, Refresh, and Observation

#### Progressive discovery

Cold discovery emits bounded parent-first commits.
Shallow work is scheduled first so a client can render useful directory structure while
the full tree is still being found.
Each directory records whether its children are completely known.

`prioritize(paths)` may reorder pending discovery work.
It never changes scope, retained facts, query semantics, or the engine version.
Internal queue order is not part of coherent state; progress reports only committed
counters and the committed discovery frontier, so accepting a hint does not create an
otherwise empty state commit.

A file-retention budget caps resources without defining semantic scope.
Reaching the numerical limit remains complete until more admissible work is actually
refused. After refusal, the engine reports explicit partial coverage and unknown absence
outside complete directories.
The readable partial session does not begin observation or accept refresh that expands
the retained set; the caller reopens with a larger budget.

#### Verified refresh

`refresh(paths)` accepts a bounded set of canonical relative paths.
It validates and deduplicates them, collapses descendants covered by an ancestor, widens
when required to verify unknown or invalid ancestry, performs filesystem I/O outside the
write guard, and conditionally lands exact commits.

Refresh cannot bypass admission, control files, the resource budget, or commit truth.
Its receipt reports accepted and rejected paths, the terminal version and state, the
commit range or exact changes, and bounded work.

#### No-gap observation handoff

Where the backend permits it, observation starts before baseline discovery and buffers
hints in a bounded queue.
After the baseline, the owner reconciles buffered hints, overflow, and registration
gaps, performs any required final verification, and enters `watching` only when the gap
is closed.

Events are hints and are verified before mutation.
Provider observation loss causes reconciliation and a trust-state transition.
A consumer falling behind the retained commit journal receives a consumer reset.
Those are separate recovery boundaries and must never share one reset flag.

### Reads, Projections, and Paging

`read()` captures one index guard, engine version, and state boundary.
It returns requested projections in request order with bounded work and a cursor that
can resume changes after that version.
It performs no filesystem I/O and no unbounded scan or sort while holding the guard.

The first native projection vocabulary is deliberately smaller than MetaBrowser’s
application algebra:

1. lookup with three-valued knowledge;
2. parent-first tree page with directory completeness;
3. portable-path-ordered flat entry page;
4. existing roll-up/report query over maintained indexes;
5. fixed-size diagnostics.

Repeated exact aggregates are maintained at commit time.
The initial structures cover hierarchical roll-ups, timestamp order, registry-derived
classification dimensions, and the fixed `all`/`unignored` partition needed by the
driving client. An unmaintained compound total is `exact(n)` or `at_least(n)` under a
request cap.

Page continuations are opaque identifiers into a bounded table owned by the opened root.
A record contains the pinned version, normalized query identity, and the data
structure’s resumable traversal position.
The public token contains no trusted path, total, sort key, request, or signature.

A continuation fails explicitly when its version is stale, its owner is foreign, or its
record was evicted. Close clears the table.
No stale page keeps a historical index image alive.

Tree order is parent-first; within a directory, directories precede nondirectories and
each partition is ordered by canonical component UTF-8 bytes.
Flat portable rows are ordered by complete canonical POSIX-relative UTF-8 bytes.
Portable projections omit unrepresentable native paths but return their exact count,
bounded escaped examples, and incomplete-directory knowledge; native roll-ups still
include them.

### Change Polling

The owner exposes one bounded journal of exact commits.
Its retained range is the same exact history used by the underlying index’s existing
nonblocking `since` compatibility API; `opened/journal.rs` adds cursor identity,
condition-variable waiting, reset, and close wakeup rather than copying commits into a
second store. `changes(after, timeout)` returns newer commits immediately or waits on a
condition variable until a commit, close, cancellation, or timeout.

Timeout is an idle result and does not advance the cursor.
A cursor older than the journal floor receives a coherent consumer reset.
A foreign or future cursor is rejected.
State-only commits are observable, and close wakes every waiter.

The first change result carries bounded invalidation information rather than row
replicas because the driving client rereads coherent projections.
There is no per-subscriber queue in core.

### Surface and Client Boundaries

#### Existing CLI and one-shot Python APIs

The command line remains a presentation of the engine and invents no interactive-only
semantics. This work adds no default CLI progress mode.
Existing one-shot Rust, CLI, and Python answers remain aligned through the shared golden
and parity corpus.

The Python package mirrors the five synchronous operations and releases the GIL around
blocking or substantial native work.
It does not create a long-lived async executor.

#### MetaBrowser adapter

MetaBrowser owns the async bridge because iterator cancellation, event-loop policy, root
replacement, host generations, and SSE replay are application concerns.
Bounded reads, refreshes, and close use MetaBrowser’s ordinary `asyncio.to_thread`
execution policy, as the Python reference provider already does.
Each fdu provider handle uses one dedicated blocking poll worker and a one-slot mailbox
to its single active async iterator.
The worker stores one native result under a lock and wakes an `asyncio.Event` through
`loop.call_soon_threadsafe`; it does not poll again or advance its local cursor until
the iterator consumes that result.
Closing the iterator joins only that bridge; closing the provider handle then joins the
bridge and native owner.

The adapter maps configuration, paths, the eight application queries, state, rows, work,
and invalidations. It does not walk the filesystem, retain a second index, rebuild
aggregates, sort full result sets, reproduce fingerprint recipes, or apply entry
replicas.

The optional fdu package is selected explicitly.
If unavailable or incompatible, startup fails with a typed error; it never silently
falls back to the Python provider.

## Trade-offs and Alternatives

### One owner instead of progressive and watch sessions

**Chosen approach:** One `OpenedIndex` owner covers discovery through observation and
close.

**Alternatives considered:** A progressive session handing an `IndexHandle` to a watch
session makes two lifecycles share one identity and leaves a baseline-to-watch gap.
An application-owned owner would duplicate engine truth and shutdown policy.

**Rationale:** One authority gives version, journal, continuation, and worker lifetime
one place to agree and one close operation to join.

### Exact commits instead of requested-operation deltas

**Chosen approach:** Mutation helpers record exact effects and publish one atomic
commit.

**Alternatives considered:** Copying producer requests into `AppliedDelta` is simpler
but misreports implicit ancestors, kind replacement, partial refusal, and control-driven
reclassification. Independent callback invalidations can drift from the facts.

**Rationale:** Exact effects are already known where reducers move; retaining them there
eliminates reconstruction and makes the reference model decisive.

### Handle-local continuations instead of signed tokens

**Chosen approach:** An opaque identifier addresses bounded state in one opened root.

**Alternatives considered:** Self-contained tokens require encoding, validation,
signing, and compatibility for data that never crosses a trust boundary.
Offset tokens rescan and can repeat work proportional to the whole index.

**Rationale:** The immediate API is in-process.
Bounded server state is smaller, easier to invalidate, and can retain the native
traversal position needed for proportional work.

### Synchronous core instead of an async runtime

**Chosen approach:** Core uses threads, locks, condition variables, cancellation, and
blocking operations; the Python binding releases the GIL and MetaBrowser owns async
adaptation.

**Alternatives considered:** An async runtime in core increases dependency, binary,
executor, and shutdown complexity and still cannot choose an application’s event-loop
policy.

**Rationale:** The filesystem and existing engine are synchronous.
A runtime-free API is usable from Rust, Python, CLI, and other runtimes without imposing
one executor.

### Cold progressive serving before warm mixed serving

**Chosen approach:** The first live baseline is cold and has explicit directory
completeness.

**Alternatives considered:** Serving a complete cached tree while replacing parts from a
cold walk is faster initially but requires composable trust for every aggregate and
deletion.

**Rationale:** Cold progressive facts have one explainable trust source.
Warm mixing is a separate architecture problem and remains available as later additive
work.

### Fixed ignore partition instead of generic tags

**Chosen approach:** Maintain only `all` and `unignored`, driven by bounded
removal-aware control state.

**Alternatives considered:** Generic tags and promoted roll-up planes anticipate future
uses but multiply reducer, snapshot, query, and live reclassification paths before a
second use exists.

**Rationale:** The fixed partition is the demonstrated client requirement and is easy to
remove or generalize after another use case establishes the correct abstraction.

### Controlled transparent-box sessions instead of an internal trace bus

**Chosen approach:** A test-only typed control drives the real opened-root owner,
workers, commit pipeline, journal, and five operations through named scheduling and
fault boundaries.
A recorder outside the owner drains the real change surface and renders
complete production requests and results.
The control may select worker count, pause or release a named boundary, script
filesystem hints, or trigger a named failure; it never supplies an entry fact, commit,
state transition, or public result.

**Alternatives considered:** Full real-filesystem sessions have the best boundary
fidelity but cannot reliably force races, gaps, overflow, or worker failure.
A pervasive internal event bus can log every queue and thread step, but creates a second
semantic vocabulary, couples goldens to refactors, and adds release-path cost.
A fully simulated owner is deterministic but does not test the orchestration being
added. Abstracting the clock, filesystem, scheduler, executor, and every queue behind
production traits would make tests flexible at the cost of permanent indirection and a
much larger design.

**Rationale:** The difficult new behavior is orchestration around already-tested index,
classification, and projection logic.
Controlled sessions exercise that orchestration end to end while making only
contract-relevant causal events durable: actions, exact commits, operation results,
state, work, recovery, and joined shutdown.
Incidental queue operations, mutex timing, thread identifiers, and wall-clock duration
remain implementation details.
A few native-observer, installed-package, and application tests retain real boundary
coverage.

## Security Considerations

The opened-root API is local and adds no authentication or network listener.
Its session identities and continuation tokens prevent accidental cross-owner use; they
are not credentials and must not authorize filesystem access.

All relative paths, registry documents, control files, snapshot records, request bounds,
and continuation inputs are untrusted at their boundary.
Parsing checks lengths and counts before allocation, path normalization rejects escape,
and portable error examples are escaped and bounded.
Snapshot permissions and fail-closed checksum, fingerprint, and root validation remain
unchanged.

The optional MetaBrowser package is installed from an exact tested fdu revision.
Dependency additions follow the supply-chain policy, and a selected but missing native
provider fails visibly rather than falling back to a different implementation.

## Operational Concerns

### Monitoring

Every operation reports work counters that distinguish filesystem observation, index
rows visited, rows returned, maintained-index work, waits, and recovery.
Performance evidence records platform, host, filesystem, and cache regime; shared CI has
no wall-clock pass/fail threshold.

### Logging

Core returns bounded typed state and issues rather than logging application policy.
The adapter may log provider lifecycle and recovery at the application boundary, but it
does not print unbounded paths, control-file contents, registry contents, or row data.
Worker panic and joined-shutdown failure remain explicit terminal outcomes.

### Deployment

`fdu-core` retains a no-default-features build.
Native observation and ignore dependencies are explicit features, and MetaBrowser keeps
fdu as an explicit optional provider through acceptance.
Cross-repository CI builds and installs a wheel from the exact fdu revision rather than
using a moving branch or sibling source checkout.

### Scaling

Discovery commits, refresh inputs, issues, dirty paths, reads, work, journal history,
continuations, and the adapter mailbox are all bounded.
Repeated exact queries use commit-maintained structures only when measurement justifies
their mutation and memory cost.
No page retains a historical index image, no subscriber owns an engine queue, and an
idle native observer performs no filesystem work.

## Open Questions

- Whether network filesystems require an initial polling observer or remain an explicit
  Python-provider mode.
- Whether the CLI ever opts into ignore support by default after dependency and binary
  measurements.
- Whether a later client justifies resumable sort orders beyond structural path order.
- What trust representation and measurements would justify warm progressive serving.

## Review Checklist

A change to the opened-root work is architecturally sound only if all answers remain
“yes”:

- Does one shared owner retain all live authority for one root?
- Does every observable mutation or state transition produce one exact atomic commit?
- Is impact derived from effective changes rather than requested observations?
- Can every read state its output and work bounds and one coherent version?
- Are unknown, capped, stale, reset, gap, and unavailable states explicit?
- Does a page resume from engine-owned bounded state without retaining an old image?
- Does filesystem I/O stay outside index read/write guards?
- Can `watch` and any ignore dependency be removed without breaking the base owner?
- Are scope, execution policy, and query selection still separate?
- Does the Python binding remain synchronous and does the application own async policy?
- Does the adapter remain a translation layer rather than a second engine?
- Can a complete causal session be recorded from real engine values without a parallel
  test-only behavior log?
- Do the CLI and existing one-shot surfaces retain their behavior and dependency floor?

## References

- [fdu design principles](fdu-design-principles.md)
- [fdu surface architecture](fdu-surface-architecture.md)
- [Opened-root implementation and integration plan](../specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md)
- [PR #47 design and readiness review](../reports/report-2026-08-25-pr-47-design-and-readiness-review.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
