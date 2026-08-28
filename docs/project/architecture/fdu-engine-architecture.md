# fdu Engine Architecture

## Overview

This document defines how `fdu-core` turns filesystem observations into coherent,
queryable directory inventory.
It covers the engine’s retained state, mutation boundary, serving lifecycles, trust
model, bounded reads, persistence, observation, and shutdown.

The engine has one fact model and two additive serving lifecycles:

- a detached lifecycle for blocking opens, one-shot reports, snapshots, and callers that
  own an `Index` value;
- an opened lifecycle for progressive discovery, repeated reads, change polling,
  explicit refresh, scheduling hints, optional observation, and joined shutdown.

Opened roots are therefore one part of the engine architecture, not a parallel
subsystem. Both lifecycles use the same entry facts, reducers, query vocabulary, and
commit rules. A streaming feature that needs a second inventory, mutation path, or query
algebra is misplaced.

This is the durable authority for engine structure and behavior.
[The design principles](fdu-design-principles.md) own the reasons and non-negotiable
rules; [the surface architecture](fdu-surface-architecture.md) owns Rust-package,
command-line, Python, and client-adapter boundaries.
Dated plans and reports record sequencing, prototype reuse, and implementation history.
They do not redefine the architecture.

## Goals and Non-Goals

### Goals

- Keep one authoritative representation of filesystem facts and derived roll-ups.
- Make every observable transition exact, atomic, ordered, and explainable.
- Support complete one-shot answers and useful progressive answers without changing
  their meaning.
- Make every limit, uncertainty, stale value, recovery boundary, and shutdown outcome
  explicit.
- Give embedded clients bounded synchronous operations without imposing an async
  runtime, transport, or application vocabulary on the engine.
- Keep observation and dependency-backed ignore handling removable, and keep content
  analysis and persistence dormant when their capabilities are not requested.
- Make a complete causal behavior session recordable from production values, so a small
  transparent-box golden corpus can exercise the orchestration end to end.

### Non-Goals

- Owning command-line presentation, Python event-loop policy, HTTP, SSE, browser state,
  or application caches.
- Defining application-specific query names when they can be composed from fdu-native
  projections.
- Adding a network protocol, portable signed continuation format, generic tag algebra,
  or reducer plug-in framework without a demonstrated second use.
- Treating a resource budget as semantic scope or a partial answer as complete.
- Mixing cached and newly verified facts in one progressive answer until trust can be
  represented and composed without ambiguity.
- Recording implementation phases, branch history, or PR-specific compatibility in this
  document.

## System Context

The filesystem and compatible snapshots supply evidence.
Producers verify and normalize that evidence.
The index alone arbitrates it, updates derived state, and creates exact commits.
Serving lifecycles decide how long to retain that state; query and formatting layers
only read it.

~~~text
filesystem --------------------+
                               |
snapshot ---- load/validate ---+--> verified producers
                                      |
                                      v
                             Index + reducers
                                      |
                              exact atomic Commit
                                      |
                     +----------------+----------------+
                     |                                 |
                     v                                 v
          detached / one-shot lifecycle       opened / live lifecycle
          complete Index or Report            OpenedIndex + shared state
                     |                                 |
                     +----------------+----------------+
                                      |
                                      v
                         pure queries and projections
                                      |
                                      v
                        Rust, CLI, Python, applications
~~~

The command line and Python package are sibling consumers.
Neither is an internal stage of the engine, and an application adapter never becomes a
second source of filesystem truth.

## Design

### Architectural Commitments

#### One fact model serves every lifecycle

`Index` is the authoritative in-memory representation of retained filesystem facts,
control state, classification, directory completeness, and roll-ups.
Cold scans, snapshot reconciliation, explicit refresh, and optional observer-backed
updates all submit evidence to the same mutation boundary.

One-shot execution may retain less state only when the complete request proves that no
cache, later query, live lifecycle, content analysis, or second view can consume the
hierarchy. That derived-report optimization must produce the same `Report` contract; it
is not a second engine or a user-selectable fast mode.

#### One opened root has one authority

`OpenedIndex` is the public authority for one opened root.
Its clones refer to one private shared state containing the index, logical version,
lifecycle, coverage, journal, continuations, discovery frontier, optional observation,
refreshes, and shutdown.
They never clone independently mutable facts while retaining the same session identity,
and a progressive lifecycle never hands authority to a second watch lifecycle.

`close()` closes the shared live state.
The first caller begins cancellation and joined shutdown; concurrent callers wait for
the same terminal result, and every clone immediately observes closing or closed state.
Dropping the last public reference performs joined shutdown as a defensive fallback, but
explicit close is the error-reporting path.
Worker closures hold weak state references or narrower shared state so they cannot keep
the final public handle alive.

#### One exact commit is the truth consumers observe

Every verified fact change and observable lifecycle transition lands through one atomic
commit path.
A `Commit` contains its version, exact effective changes, fdu-native impact,
resulting state, and bounded work.

Producers submit verified observations or state transitions.
They do not copy requested operations into a journal, independently advance a clock, or
publish callbacks that reconstruct what changed.
Mutation helpers record the effects they actually applied while facts and reducers move;
impact is derived from those effects.
A no-op does not consume a version or journal slot.

#### Every answer is bounded and honest

An interactive operation has explicit input, output, and work bounds.
One read returns projections from one coherent engine version and lifecycle state.
A limit that prevents an exact answer returns an explicit capped or lower-bound result;
it never presents partial calculation as exact.

Presence uses three-valued knowledge:

- `present` when the entry is retained;
- `absent` only when the relevant directory coverage is complete;
- `unknown` when discovery, a resource stop, or an inaccessible boundary prevents an
  absence claim.

Trust and coverage are separate.
A complete cached value can be structurally complete but stale; a freshly discovered
partial subtree can be current but incomplete.
Neither axis may be inferred from the other.

#### A complete behavior session is inspectable

The engine’s values are complete enough for a client or test recorder to explain one
causal session: options, requests, responses, commits, coherent state, work, recovery,
and shutdown. Tests do not need a parallel mutation log, selected debug counters, or a
second event vocabulary that can drift from public behavior.

If an important effect cannot be explained from operation results, the commit journal,
bounded diagnostics, and a final read, improve those production values first.
Test seams may pause named scheduling boundaries, provide deterministic observer hints,
or trigger named failures.
They control when production behavior runs; they never state filesystem facts, mint
commits, or bypass verification.

#### Optional mechanisms remain removable

The retained index, commit path, explicit refresh, bounded reads, opened lifecycle
state, journal, continuation table, and shutdown use the standard library and build
without native observation.
The `watch` feature supplies observation hints and their driver, not a second index or
change contract.

Ignore handling is a fixed semantic capability and remains removable if it requires an
additional dependency.
Content analysis is opt-in and separately persisted.
Core does not acquire an async runtime, web stack, transport serialization framework, or
token-signing dependency for the live API.

### Core Values and Ownership

#### Detached index

`Index` is an independently owned tree image.
It stores platform-native path facts, parent relationships, reducers, classification
state, control state, provenance, directory completeness, and snapshot metadata.

A detached index may retain bounded exact history for a nonblocking `since` API. That
history has no live session identity, waiter, worker, or continuation authority and is
never persisted. A cloned `Index` is a separate value.

`IndexHandle` remains a short-write coordination primitive for reconciliation and
compatibility paths.
It is not the live-root API and does not acquire change-poll or shutdown authority.

#### Commit

`Commit` is the atomic public consequence of one accepted mutation batch or lifecycle
transition. It orders changes with a monotonic process-local version and carries:

- exact inserted, removed, replaced, metadata, classification, and control effects;
- the derived domains and bounded paths a consumer may need to reread;
- lifecycle, coverage, freshness, and recovery state after the transition;
- bounded work and issues associated with the transition.

Facts, reducers, state, version, and journal publication move together.
There is no route that changes one without the others.

#### Persisted snapshot

A snapshot is a detached representation, not a dormant opened root.
It contains complete retained facts, reducers, validated scope and semantic identities,
and the control state required to interpret them.
It never contains a live session identity, version sequence, journal history, waiter,
continuation, worker, or discovery frontier.

Saving fails closed when the index is not representable as a complete snapshot.
The current format does not encode an unfinished frontier, unknown children, evicted
nodes, or a resource-stopped baseline, so those states are rejected rather than reloaded
later as complete.

#### Opened root

`OpenedIndex` is the public API and ownership handle for one live root.
A clone contains only a reference to the same private shared state.
There is no parallel `Owner` service, interface-and-implementation pair, or
method-for-method forwarding layer.
Public operations are implemented directly on `OpenedIndex`; private modules provide
data structures and algorithms rather than a second API surface.
The shared-state indirection exists because discovery workers, change waiters,
concurrent callers, cancellation, and joined shutdown must share one lifetime.
It does not translate or mirror `Index`: `Index` is the detached fact value, while
`OpenedIndex` owns the live lifecycle around that value.

The shared state contains:

- the guarded current `Index`;
- one opaque session identity and monotonically increasing version;
- validated root, scope, and semantic identities;
- lifecycle, coverage, freshness, progress, and bounded issues;
- one bounded commit journal and its condition variable;
- one bounded continuation table;
- the discovery frontier and scheduling hints;
- cancellation plus every worker and bridge wakeup it owns.

The synchronous Rust boundary has one constructor and five lifecycle operations:

~~~rust
impl OpenedIndex {
    pub fn open(root: &Path, options: OpenOptions) -> Result<Self>;
    pub fn read(&self, request: ReadRequest) -> Result<ReadResponse>;
    pub fn changes(&self, request: ChangeRequest) -> Result<ChangePoll>;
    pub fn refresh(&self, paths: &[RelativePath]) -> Result<RefreshResult>;
    pub fn prioritize(&self, paths: &[RelativePath]) -> Result<PriorityResult>;
    pub fn close(&self) -> Result<()>;
}
~~~

Names may follow established fdu vocabulary, but the responsibilities and shared-state
semantics are stable.
The associated constructor preserves the existing blocking free `open()` contract.

#### Reports and derived report plans

`Query` values are immutable requests over retained facts.
`Report` values are immutable, provenance-carrying answers.
Formatting serializes a report and never changes query semantics.

A derived report plan is transient execution state for a provably one-shot request.
It produces the same `Report` shape and semantic hash as indexed execution.
It never becomes a hidden cache or alternate query grammar.

#### Content index and sidecar

Content analysis is a derived tier over metadata facts.
The sparse content index and content roll-ups exist only when an analysis profile is
enabled. Workers submit independently fingerprint-checked analysis results through the
index’s derived-data boundary; they do not change metadata truth or advance its clock.

The content sidecar is profile-scoped persistence.
It is not embedded in the metadata snapshot and is never loaded by metadata-only work.

### Serving Lifecycles

#### Detached and one-shot

The blocking `open()` lifecycle may load a compatible snapshot, but it completes the
required filesystem reconciliation before returning a fresh answer.
It never serves an unverified snapshot as fresh or replaces a complete cached tree with
a partial cold result.

Cache-only mode is the explicit exception: it performs no filesystem verification,
labels the source as cached, and fails when no usable snapshot exists.
The caller receives a complete `Index` and an `OpenReport` describing the path taken,
work, and any partial errors.

One-shot `report()` may use the derived-report plan when the request proves that
retained state has no consumer.
One-shot and retained paths must remain semantically identical for the same request.

#### Opened and long-lived

The opened lifecycle returns while cold discovery may still be running.
It owns the root from baseline discovery through optional observation and explicit
refresh, serves coherent bounded reads, exposes a bounded pull journal, accepts
scheduling hints, and closes by cancelling and joining everything it owns.

The live baseline is cold-progressive unless a separately reviewed trust design permits
mixing cached and newly verified facts.
Cold discovery has one explainable source and explicit directory completeness.
It does not publish a cached whole beside newly discovered fragments under one
undifferentiated freshness label.

An opened root starts native observation only when its explicit options request a scope
the backend can support honestly.
Unsupported combinations fail explicitly rather than silently omitting paths.

Interactive serving state is opt-in and belongs to the opened lifecycle alone.
An opened root allocates one optional set of maintained structures — portable path and
per-directory child order, classification and declared-name tallies, and a global file
recency order — and updates them inside the same exact commit that moves the facts, so a
read traverses maintained state instead of scanning and sorting per request.
A detached `Index`, a snapshot, a one-shot report, and the command line allocate none of
it, retain none of it, and pay nothing for it.
That isolation is a tested boundary, not an intention: the assertion is that a detached
index’s serving state is absent, and the command line’s existing golden corpus is
required to stay unchanged.

Each maintained structure has to name the measured request work it removes.
None is justified by having existed in an earlier prototype, because a maintained index
is a permanent cost on every commit paid for an intermittent read.

### Configuration, Scope, and Identity

Configuration separates answer semantics from execution policy and query selection.

Scope identity contains values that change which facts the engine may retain:

- hidden-component admission and any exact-name allowlist;
- symlink-following behavior;
- filesystem-boundary behavior;
- admitted object kinds;
- an explicit retained-depth bound, when the serving lifecycle supports one.

A query or display depth is selection, not scope.
The live lifecycle’s observation-compatible default is an unbounded-depth retained
scope; an application’s viewport depth never narrows discovery.

Semantic identity covers the normalized runtime type registry, fixed ignore semantics,
classification rules, and versioned reducer behavior.
The engine derives scope and semantic identities from validated values.
It never accepts a caller-supplied fingerprint as proof that independently supplied
content matches.

Worker count, traversal order, batching, scheduling hints, journal and continuation
capacity, resource budgets, and observation mode are execution policy.
A file-retention budget limits resources; it does not promise a deterministic
cross-provider prefix and is not part of semantic scope.

The root binding is session-local and platform-native.
Portable cross-provider path identity is a projection and remains separate from the
native root identity.

### Trust, Coverage, and Lifecycle State

Every returned value carries enough context to calibrate it:

- source records whether facts were scanned, revalidated, journal-scoped, or cached;
- coverage records whether the relevant retained scope is complete or partial;
- observation time records when the underlying evidence was collected;
- lifecycle records whether the opened root is discovering, reconciling, watching,
  resource-stopped, closing, closed, or terminally failed;
- bounded issues explain operational conditions that affect the answer.

Coverage does not imply direction.
A partial result grows monotonically only while an additive producer is running; a
partial result caused by errors may later move either way.

A change journal narrows where uncertainty may exist but never upgrades unverified
evidence into verified fact.
Provider-reported loss or ambiguity causes explicit reconciliation and trust-state
transition.

### Exact Commit Pipeline

Every producer uses the same transition:

~~~text
verified facts or lifecycle transition
  -> scope, path, precondition, and control validation
  -> normalized prepared input
  -> atomic fact, reducer, control, and state application
  -> exact effective changes and derived impact
  -> Commit { version, changes, impact, state, work }
  -> bounded history and waiter wakeup
~~~

Filesystem I/O and expensive parsing occur outside the write guard where possible.
Prepared input is checked against current facts under the guard, so stale preparation
cannot publish an incorrect effect.
A precondition mismatch is a no-op or retry signal, never a partial commit.

Conditional observations carry generation and revision guards.
They reject present-state ABA, parent replacement, and absent create/remove races at one
batch boundary without making unrelated subtrees conflict.

Mutation helpers report what they actually did.
Verified ancestor creation, kind replacement, a resource refusal after earlier effects,
control-driven reclassification, and a state-only transition are all representable.
The version advances only when the full commit is ready.

The public impact vocabulary remains fdu-native: topology, metadata, classification,
aggregates, content, and trust or lifecycle state, plus bounded dirty paths or
`all_dirty`. Applications may map those domains to their cache vocabulary; core never
stores application query names.

### Producers and State Transitions

#### Cold scan and progressive discovery

A cold scan establishes a historyless baseline through the normal index mutation rules.
The detached lifecycle may publish only the complete result to its caller.
The opened lifecycle publishes bounded parent-first commits so a client can render
useful shallow structure while deeper work continues.

Each directory records whether its children are completely known.
Reaching a numerical resource limit remains complete until additional admissible work is
actually refused. After refusal, the opened root reports partial coverage and unknown
absence outside complete directories.
A resource-stopped live root remains readable but does not accept work that would expand
the retained set; the caller reopens with a larger budget.

`prioritize(paths)` may reorder pending discovery.
It never changes scope, retained facts, query semantics, or the engine version.
Progress reports committed counters and coverage, not incidental queue order.

#### Snapshot reconciliation and explicit refresh

Reconciliation conditionally applies verified differences while it walks.
Explicit `refresh(paths)` accepts a bounded set of canonical relative paths, validates
and deduplicates them, collapses descendants covered by an ancestor, and widens when
unknown or invalid ancestry must be verified.

Refresh cannot bypass admission, control files, resource budgets, or commit truth.
Its result reports accepted and rejected paths, terminal version and state, commit range
or exact changes, bounded work, and issues.

#### Native observation

Filesystem events are scheduling hints, not facts.
The observer coalesces hints and verifies affected paths before submitting prepared
input.
A sample is valid at its filesystem observation point; the engine does not pretend
it can freeze external mutation until the in-memory commit.
Logical preconditions prevent an older sample from overwriting newer facts.

Where the backend permits it, observation starts before baseline discovery and buffers
hints in a bounded queue.
After the baseline, the opened root reconciles buffered hints, overflow, and
registration gaps, then enters watching only after the gap is closed.
Hints arriving during verification remain queued for the next batch.

Provider loss and consumer lag are different recovery boundaries:

- observer loss or ambiguity invalidates and reconciles affected filesystem scope;
- a consumer older than the retained commit floor receives a coherent consumer reset.

They never share one reset flag.

### Reads, Queries, and Paging

Queries are pure readers.
They do not perform filesystem I/O, mutate facts, or decide serving policy.
The same retained facts, query, and provenance produce the same report.

An opened `read()` captures one index guard, engine version, and lifecycle state.
It returns requested projections in request order with explicit bounds and a cursor that
can resume changes after that version.
It performs no filesystem I/O and no unbounded scan or sort while holding the guard.

The native projection vocabulary stays smaller than any one application’s algebra:

1. lookup with three-valued knowledge;
2. parent-first tree pages with directory completeness;
3. portable-path-ordered flat entry pages;
4. maintained roll-ups and existing report queries;
5. fixed-size diagnostics.

Repeated exact aggregates are maintained at commit time only when their read frequency
and measured cost justify mutation and memory overhead.
An unmaintained compound total is `exact(n)` or `at_least(n)` under a request cap.

Every ordered page has a **total** order: two providers answering one query at one
version return the same rows in the same sequence, with no tie left to insertion order,
hash iteration, or a stable sort’s input order.
An order a provider merely happens to produce is not a contract, and an order stated
only as prose is not either — “parent-first” once stood here and did not distinguish
level order from pre-order, which are different sequences for any tree deeper than one
level.

Tree order is **breadth-first level order**: all children of the requested path, then
all children of those directories, to the query’s maximum depth.
Within one parent, directories precede nondirectories and each partition is ordered by
canonical component UTF-8 bytes.
Level order matches parent-first discovery, so a page is served from the knowledge that
exists earliest, and it keeps truncation honest — a pre-order page cut at its row bound
can return one directory and a thousand of its descendants while leaving the caller
unable to tell whether the parent held two entries or two thousand.

Flat and catalog rows are ordered by complete canonical POSIX-relative UTF-8 bytes.
Ranked recency has a selection order and a presentation order, and they differ.
Rows are selected by ignored state, then modification time descending, then canonical
path ascending; the page that survives is returned in modification time descending, then
that same path ascending.
The path is the final key in both, which is what makes each total.

Ignored entries rank last during selection because installing dependencies writes
thousands of files at once, and pure recency would answer “what changed recently” with a
page of vendored output.
The demotion applies in every branch: applying it only when a page overflows gives one
query name two ranking contracts.

Excluding ignored entries prunes the excluded directory’s whole subtree, not merely its
row.

Every entry has a canonical portable name, so ordered pages and native roll-ups answer
over one population.
A native path need not be UTF-8, so exactly two kinds of byte are percent-escaped —
those that do not decode, and `%` itself, whose escaping is what keeps the mapping
injective. Nothing else changes, because the result is a JSON string and not a URL.

A directory whose own name required escaping still lists its children, and a complete
directory that does not hold a name answers `absent` rather than `unknown`, because
nothing can be hiding in a set that cannot be listed.
One completeness value therefore answers for every consumer.

Page continuations are opaque identifiers into a bounded table owned by one opened root.
A record contains the pinned version, normalized query identity, and native resumable
traversal position. The public token contains no trusted path, total, sort key, request,
or signature.

A flat continuation stores the last emitted portable path, which alone fixes the
position. A tree continuation stores a stack of at most the query’s maximum depth
`(parent portable path, last emitted child name)` pairs, which enumerates a level
lazily. It deliberately does not store the frontier: the set of directories one level up
is unbounded in directory width, so it cannot fit a bounded record, and re-deriving it
per page would make paging one wide level quadratic in the number of pages.

A continuation fails explicitly when its version is stale, belongs to another opened
root, or its record was evicted.
Close clears the table.
No page retains a historical index image.

### Change History

The index retains one bounded exact history.
Detached callers may inspect it with the nonblocking `since` API. The opened lifecycle
adds session-aware cursors, condition-variable waiting, reset, and close wakeup around
that same history; it does not copy commits into a second store.

`changes(after, timeout)` returns newer commits immediately or waits until a commit,
close, cancellation, or timeout.
Timeout is an idle result and does not advance the cursor.
A cursor below the journal floor receives a coherent consumer reset; a foreign or future
cursor is rejected. State-only commits are observable, and close wakes every waiter.

The journal carries bounded invalidation and reread guidance, not an application row
replica. There is no per-subscriber queue in core.

### Surface and Client Boundary

The engine API is synchronous and runtime-free.
Blocking or substantial native work is exposed clearly so language bindings can release
their runtime lock. The Python package mirrors engine values; it does not add a
long-lived async executor.

An async application owns its bridge from blocking change polls to its event loop.
That bridge may own cancellation, host generations, root replacement, and application
cache invalidation. It may not walk the filesystem, retain an entry replica, rebuild
roll-ups, reproduce scope or fingerprint rules, or invent paging semantics.

The command line presents engine capability and invents none.
An additive embedded API does not need to become a default CLI mode before a client has
proven it, but any later CLI mode must be a presentation of the same operations and
values.

The complete package and adapter boundary is in
[the surface architecture](fdu-surface-architecture.md).

### Testability and Behavioral Evidence

The difficult live behavior is orchestration around index, classification, query, and
rendering logic that already have focused tests.
The primary integration instrument is therefore a controlled session harness over the
real opened-root state and commit path.

A scenario is a compact action script:

- open with validated options and deterministic worker policy;
- pause or release a named scheduling boundary;
- create, modify, remove, rename, or make a path inaccessible;
- deliver a real or scripted observer hint, gap, or overflow;
- call read, changes, refresh, prioritize, or close;
- record the production response, drained commits, bounded state, and final projection.

The recorder normalizes only fields classified as unstable, such as temporary root
paths, wall-clock timestamps, and platform identities.
Versions, actions, paths inside the fixture, changes, impact, coverage, bounds, resets,
work categories, shutdown outcomes, and serialized projection values remain exact.

Each golden is a complete, causal product example rather than a collection of
hand-selected internal assertions.
A small matrix composes lifecycle, producer, recovery, query, and shutdown dimensions so
one session protects many interactions without duplicating setup.
Focused unit and property tests still own algorithms and combinatorial edge cases.
Native-observer smoke tests, installed-wheel tests, and application conformance tests
retain the real platform and packaging boundaries.

The harness controls causality but observes only production values.
A refactor that preserves behavior should not rewrite goldens merely because queue,
mutex, thread, or batching details changed.

## Trade-offs and Alternatives

### Shared fact model instead of a streaming subsystem

**Chosen approach:** Detached and opened lifecycles use one `Index`, commit path, and
query model.

**Alternatives considered:** A separate streaming inventory can be built quickly around
callbacks, but it duplicates mutation, aggregation, freshness, and query semantics.

**Rationale:** Streaming changes when an answer becomes visible, not what the answer
means. One fact model makes parity structural and keeps the additive cost bounded.

### One opened root instead of progressive and watch sessions

**Chosen approach:** One `OpenedIndex` lifetime covers discovery through observation and
close.

**Alternatives considered:** A progressive session handing an `IndexHandle` to a watch
session creates two lifecycles around one identity and a baseline-to-watch gap.
An application-owned live-state coordinator duplicates engine truth and shutdown policy.

**Rationale:** Version, history, continuations, workers, and close need one place to
agree.

### Exact commits instead of requested-operation deltas

**Chosen approach:** Mutation helpers record exact effects and publish one atomic
commit.

**Alternatives considered:** Copying producer requests into a delta misreports implicit
ancestors, kind replacement, partial refusal, and control-driven reclassification.
Independent callbacks can drift from retained facts.

**Rationale:** Effective changes are already known where reducers move.
Retaining them there removes reconstruction and makes model-based tests decisive.

### Synchronous core instead of an async runtime

**Chosen approach:** Core uses threads, locks, condition variables, cancellation, and
blocking operations; bindings and applications own async adaptation.

**Alternatives considered:** An async runtime in core adds dependencies, binary weight,
executor policy, and shutdown complexity while still being unable to choose an
application’s event loop.

**Rationale:** The filesystem and existing engine are synchronous.
A runtime-free API works from Rust, Python, the CLI, and other runtimes.

### Handle-local continuations instead of signed tokens

**Chosen approach:** An opaque identifier addresses bounded state in one opened root.

**Alternatives considered:** Self-contained tokens require encoding, validation,
signing, and compatibility for data that currently stays in-process.
Offset tokens rescan and repeat work proportional to the index.

**Rationale:** Bounded opened-root-local state is smaller, easy to invalidate, and can
retain the native traversal position needed for proportional paging.

### Cold progressive serving before mixed-source serving

**Chosen approach:** A live baseline is cold and carries directory completeness.

**Alternatives considered:** Serving a complete cached tree while replacing pieces from
a cold walk improves first paint but requires composable trust for every aggregate,
move, and deletion.

**Rationale:** Cold progressive facts have one explainable trust source.
Mixed-source serving remains additive once its trust model is proven.

### Fixed ignore partition instead of generic tags

**Chosen approach:** Maintain the demonstrated `all` and `unignored` partition behind an
explicit capability.

**Alternatives considered:** Generic tags and promoted roll-up planes multiply reducer,
snapshot, query, and live reclassification paths before a second use establishes the
right abstraction.

**Rationale:** The fixed partition meets the known use case and can be removed or
generalized when evidence demands it.

### Controlled sessions instead of an internal trace bus

**Chosen approach:** Test-only control drives the real opened-root state and records
public operations, commits, state, work, recovery, and shutdown.

**Alternatives considered:** Real-filesystem-only tests cannot reliably force races,
gaps, overflow, or worker failure.
A pervasive trace bus creates a parallel semantic vocabulary and couples goldens to
internal refactors. A simulated live-root coordinator does not test the orchestration.
Abstracting every clock, filesystem call, scheduler, executor, and queue creates
permanent indirection for test flexibility.

**Rationale:** Controlled sessions expose causal behavior without replacing production
truth or preserving incidental concurrency details.

## Security Considerations

The engine is local and adds no authentication or listener.
Session identities and continuation tokens prevent accidental cross-session use; they
are not credentials and never authorize filesystem access.

Relative paths, registry documents, ignore controls, snapshots, declared counts, request
bounds, and continuation inputs are untrusted at their boundaries.
Path normalization rejects root escape.
Parsers validate lengths and counts before allocation.
Portable diagnostics escape and bound path examples.

Snapshots fail closed on checksum, format, fingerprint, scope, semantic, or root
mismatch and use owner-only permissions where supported.

## Operational Concerns

### Monitoring

Operations report bounded work categories that distinguish filesystem observation, index
rows visited, rows returned, maintained-index work, waits, and recovery.
Performance evidence records platform, host, filesystem, and cache regime; shared CI
does not impose wall-clock pass/fail thresholds.

### Logging

Core returns typed state and bounded issues rather than logging application policy.
Adapters may log lifecycle and recovery at their boundary, but do not print unbounded
paths, registry or control-file contents, or row data.
Worker panic and joined-shutdown failure remain explicit terminal outcomes.

### Deployment

`fdu-core` retains a no-default-features build.
Native observation, ignore handling, and content analysis are explicit capabilities.

### Scaling

Discovery commits, refresh inputs, issues, dirty paths, reads, work, history, and
continuations are bounded.
Repeated exact queries gain commit-maintained structures only after measurement
justifies their mutation and memory cost.
No page retains a historical index image, no subscriber owns an engine queue, and an
idle native observer performs no filesystem work.

## Future Considerations

### Open Questions

- What trust representation and measurements would justify progressively serving a warm
  snapshot while it is revalidated?
- Which network filesystems can support native observation honestly, and which should
  remain explicit refresh-only modes?
- Does a later client justify resumable sort orders beyond structural path order?
- Should ignore support ever become a default CLI capability after dependency, binary,
  and behavior measurements?

### Potential Improvements

- Add mixed-source progressive serving after per-subtree trust and deletion semantics
  have a reviewed composition proof.
- Add maintained projections only when recorded read workloads show that bounded
  on-demand work is insufficient.
- Add portable or persistent continuations only if an external trust boundary requires
  them; do not pre-pay the protocol cost for an in-process handle.
- Generalize fixed control partitions only after a second independent use case exposes
  the shared abstraction.

## Architecture Conformance Checklist

An engine change is sound only if these answers remain yes:

- Do detached and opened lifecycles use one fact and query model?
- Is `OpenedIndex` the only live-root behavior surface, with private shared state rather
  than a parallel service API?
- Does one shared live state retain all authority for one root?
- Does every observable mutation or lifecycle transition produce one exact atomic
  commit?
- Is impact derived from effective changes rather than requested observations?
- Can every operation state its input, output, and work bounds?
- Does every read identify one coherent version, lifecycle state, source, and coverage?
- Are unknown, capped, stale, reset, gap, unavailable, and terminal states explicit?
- Does a page resume from bounded opened-root state without retaining an old index
  image?
- Does filesystem I/O stay outside index read and write guards where possible?
- Can observation and other optional capabilities be removed without breaking the base
  engine?
- Are scope, execution policy, and query selection still separate?
- Does async adaptation remain outside core?
- Does every adapter translate engine values rather than recreate inventory state?
- Can a complete causal session be recorded without a parallel test-only behavior log?
- Do the command line and existing one-shot surfaces retain their behavior unless a
  separately reviewed product change says otherwise?

## References

- [Architecture index](README.md)
- [fdu design principles](fdu-design-principles.md)
- [fdu surface architecture](fdu-surface-architecture.md)
- [Opened-root implementation and integration plan](../specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md)
- [PR #47 design and readiness review](../reports/report-2026-08-25-pr-47-design-and-readiness-review.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
