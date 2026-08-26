# Feature: Opened-Root Inventory Engine Rewrite

**Date:** 2026-08-25

**Author:** fdu project, with Codex review assistance

**Status:** Draft

## Overview

This plan replaces the interactive portion of
[PR #47](https://github.com/jlevy/fdu/pull/47) with a smaller design built from fdu’s
existing retained index and the client boundary proven by
[MetaBrowser PR #74](https://github.com/jlevy/metabrowser/pull/74). It is a rewrite of
the opened-root and streaming work, not a rewrite of fdu’s one-shot engine.

The immediate client is MetaBrowser, but the API must remain a general fdu API.
MetaBrowser owns HTTP, host-side caches, root replacement, and browser projections.
fdu owns filesystem truth: discovery, retained facts, roll-ups, verified changes,
bounded reads, refresh, scheduling, and shutdown.
The adapter between them should translate values, not recreate an inventory engine.

The core design is one opened-root handle with the same five operations MetaBrowser has
already validated:

```text
open(root, options) -> opened root

opened root:
  read(request)
  changes(after, timeout)
  refresh(paths)
  prioritize(paths)
  close()
```

One owner holds discovery, the retained index, the commit journal, optional filesystem
observation, continuation state, and worker lifetime.
Every mutation enters one exact commit path.
Reads and change polls are coherent views of committed state.
The API is blocking and runtime-free in Rust; Python releases the GIL, and MetaBrowser
owns the bounded bridge from blocking change polls to its async provider contract.

This plan deliberately does not reproduce all of PR #47. The branch is valuable as a
prototype and source of tested components, but its history shows that implementing the
entire client contract horizontally made ownership and mutation invariants difficult to
see. The replacement uses one fresh fdu branch and one long-lived draft PR from current
`main`. Work advances through vertical, reviewable commit groups, and every phase must
meet its acceptance gate before the next begins.

MetaBrowser is a separate repository, so its contract and adapter changes necessarily
remain on the MetaBrowser PR #74 branch.
The two PRs form one coordinated integration effort and pin each other’s exact tested
revisions.

## Decision Summary

| Question | Decision |
| --- | --- |
| Merge PR #47 after more patching? | No. Preserve it as implementation evidence and extract selected pieces into one fresh rewrite branch from `main`. |
| How is the rewrite delivered? | One long-lived fdu draft PR with phase-gated commit groups; MetaBrowser changes remain on its own PR #74 branch. |
| Keep MetaBrowser’s provider boundary? | Yes. The coordinator/provider split and five-operation handle are sound. |
| Put MetaBrowser’s eight query names in fdu? | No. Keep a small fdu-native read algebra and map to the client vocabulary in a thin adapter. |
| Use separate progressive and watch sessions? | No. One opened root owns a single lifecycle, session identity, clock, and journal from first discovery through observation. |
| Report requested observations as changes? | No. A prepared mutation records its exact effective changes; the journal stores the resulting commit verbatim. |
| Make `max_files` semantic scope? | No. Treat it as a discovery resource budget with explicit partial coverage, not as a promise of a deterministic retained prefix. |
| Make MetaBrowser’s depth limit semantic scope? | No. It is a read-selection and rendering bound. The live opened root discovers an observation-compatible unbounded-depth scope. |
| Require exact page remainders? | No. Prove lossless bounded paging by conformance; expose product totals separately when the UI needs them. |
| What is provider row order? | Tree pages use parent-first traversal with directories before nondirectories and canonical-name order within each partition. Flat pages use lexicographic canonical POSIX-path UTF-8 byte order. |
| Sign self-contained page tokens? | No. Use opaque, handle-local continuation IDs backed by bounded server-side state. The immediate boundary is in-process. |
| Accept a registry fingerprint as classification input? | No. Pass the actual registry document, validate it in each provider, and derive the reported fingerprint from that content. |
| Ship arbitrary tags and promoted roll-up planes now? | No. Ship the one demonstrated partition, `all` versus `unignored`, behind a feature; defer a generic tag algebra until a second use case exists. |
| Serve warm and cold facts together during discovery? | Not in the first version. Stream a cold baseline honestly; retain current blocking warm-cache behavior until trust can be represented without per-value guesswork. |
| Add an async Rust runtime? | No. Use standard threads and synchronization in core. Async adaptation belongs at the Python boundary. |
| Change the default CLI? | No. Existing one-shot behavior and output remain the default. Interactive progress is a later explicit mode over the same engine. |

The detailed Design and phase acceptance sections are normative for implementation.
The Decision Summary and Bead Reconciliation table are indexes and must be corrected if
they drift from those sections.
The review report owns the diagnosis and evidence; this plan owns later implementation
decisions, including the explicit delivery override recorded below.

## Goals

- Give embedded clients one cloneable handle to one authoritative opened root.
- Serve bounded coherent reads while discovery, refresh, or observation commits proceed.
- Stream discovery and verified filesystem changes through one resumable version and
  journal contract without a handoff gap.
- Keep partial knowledge honest: a caller can distinguish present, absent, and unknown,
  and can see why coverage or freshness is incomplete.
- Supply the roll-ups, classification, recency, directory, and flat-entry primitives
  needed to implement every MetaBrowser PR #74 query without per-request aggregation in
  Python.
- Preserve fdu’s existing CLI and Python one-shot behavior.
- Keep `fdu-core` runtime-free and its default feature set small.
- Prove cross-provider agreement with an independent model, shared classification
  fixtures, and recorded observation replay.
- Land the work as reviewable commit groups on one draft PR, with each phase preserving
  one central invariant and returning the full gate to green.

## Non-Goals

- Merging or continuing to stack work onto PR #47.
- Making MetaBrowser’s HTTP routes, SSE protocol, host overlay, caches, or query names
  part of fdu-core.
- A generic provider command channel.
- A generic user-defined tag system, arbitrary promoted tally planes, or a new reducer
  plugin framework.
- Retaining historical index images for arbitrary old-version reads.
- Guaranteeing an identical retained prefix when concurrent discovery reaches a resource
  budget.
- Signed, portable, or network-safe page tokens.
- Sorted resumable pages for every possible report order.
- Lazy warm-snapshot streaming or mixed-source per-value provenance in the first
  implementation.
- Replacing MetaBrowser’s Python provider before the fdu provider passes the same
  conformance suite and end-to-end measurements.
- Changing fdu’s default CLI output or turning the CLI into a long-lived server.

## Background

### What PR #47 established

PR #47 demonstrated that the retained index can support an interactive client and
produced several components worth extracting:

- shared guarded reads rather than cloning the full index;
- a coherent read envelope containing version, state, projections, and work;
- a clocked bounded journal with explicit reset on lost history;
- runtime file-type registry parsing, logical and canonical extensions, and browsing
  groups;
- bounded refresh, invalidation carriers, GIL release, async Python adaptation, and
  scripted watcher tests;
- scope tests for hidden paths, filesystem boundaries, special objects, and resource
  limits.

The full review is in
[the PR #47 design and readiness report](../../reports/report-2026-08-25-pr-47-design-and-readiness-review.md).
Its central finding is structural: correctness defects recur where a concept has two
owners or where one representation is mutated and another is reconstructed later.
The main examples are requested observations standing in for effective mutations,
mutable index clones sharing live identity, separate baseline and watch sessions,
admission policy copied into producer loops, and self-contained continuation tokens
carrying internal authority.

PR #47 is therefore evidence, not a compatibility contract.
Its unmerged public types can change freely.

### What MetaBrowser PR #74 established

The MetaBrowser prototype was reviewed at exact head
[`3183888`](https://github.com/jlevy/metabrowser/commit/3183888808b366b5ba1c381dec1cbb18b49d969e).
Its current CI matrix passes.
The prototype settles the client-side ownership boundary:

- `InventoryCoordinator` owns root replacement, host versioning, sparse overlay,
  response serialization, host caches, and application events.
- `InventoryBackend.open()` creates one provider-owned root handle.
- The provider owns discovery, retained filesystem facts, roll-ups, watching, refresh,
  prioritization, and bounded query execution.
- The handle exposes only `read`, `changes`, `refresh`, `prioritize`, and `close`.
- Large request and response values are typed and bounded.
- A provider conformance suite exercises the Python reference implementation.

That is a good seam and should remain.
The eight MetaBrowser query variants are also justified at its application boundary: the
provider must compute aggregates, and the coordinator must not rebuild them.
They do not, however, need to become fdu’s internal vocabulary.

Three assumptions should change jointly before fdu adoption:

1. `max_files` is currently called semantic scope, but the Python provider does not
   maintain one exact global prefix across initial walking, subtree rewalks, and live
   upserts. A concurrent native walker cannot promise that prefix either.
2. Exact `remaining_rows` and repeated catalog totals primarily defend page assembly;
   they are not independent product data.
   Runtime conservation fields make every provider count the remainder before it can
   return the first page.
3. `InventoryConfig` carries only `registry_fingerprint`, while the Python provider
   independently loads MetaBrowser’s packaged registry.
   A second provider could report the supplied fingerprint while classifying with
   different rules.

Because both packages and the immediate client are owned together, these should be fixed
in the contract rather than hidden in an adapter.

### Constraints from fdu’s design

The rewrite must preserve the repository’s
[design principles](../../architecture/fdu-design-principles.md) and
[surface architecture](../../architecture/fdu-surface-architecture.md):

- defaults answer the stated question;
- no bound or truncation is silent;
- filesystem events are hints and verified observations are facts;
- `AppliedDelta` describes what changed, exactly;
- queries are pure readers and scope is not selection;
- the watch feature remains removable;
- the core, CLI, and Python surfaces cannot disagree;
- visible CLI behavior is tested broadly rather than by surgically parsing only the
  value a test expects.

### Ownership of progressive work

This plan owns cold progressive discovery, the opened-root/session lifecycle, coherent
mid-discovery reads, the no-gap observation handoff, and the immediate MetaBrowser
adapter contract.

The older [progressive-results plan](plan-2026-08-11-fdu-progressive-results.md) is
narrowed to warm persisted roll-ups, lazy warm open, prefer-cache policy, and per-value
mixed-source provenance.
Its traversal-order work has landed and remains useful background, but it no longer owns
a second streaming-session API. Epic `fdu-wpa0` tracks only that narrowed warm-serving
work; `fdu-snej` owns this plan.

## Design

### Ownership boundary

The system has three layers with one owner at each concern:

```text
MetaBrowser coordinator
  owns root replacement, host overlay, HTTP/SSE, host caches, app events
        |
        | typed InventoryBackend / InventoryHandle contract
        v
thin fdu adapter
  maps config, fdu read primitives, state, impact, and work
        |
        | fdu-native values
        v
OpenedIndex owner
  owns discovery, index, commit journal, observer, continuations, workers
```

The coordinator never applies filesystem deltas to a second inventory.
The adapter never walks or aggregates entries.
The core never names MetaBrowser routes, query kinds, cache keys, or SSE events.

`OpenedIndex` is cloneable as a lightweight reference to one owner.
Cloning it does not copy the index, clock, session, journal, or continuation table.
An owned `Index` value may remain cloneable for current callers if necessary, but such a
clone is a detached value and carries no live session or continuation authority.
Persisted state uses a distinct snapshot representation rather than pretending to be a
second live owner.

`close()` closes that shared owner, not one reference.
The first caller starts cancellation and joined shutdown; concurrent callers wait for
and receive the same terminal result.
Every outstanding clone immediately observes the closing or closed lifecycle and all
operations except repeated `close()` return a typed closed-handle error.
Dropping the last reference performs the same joined shutdown as a defensive fallback,
but ordinary clients close explicitly.

### Surface shape

The initial Rust shape is intentionally synchronous:

```rust
pub fn open(root: &Path, options: OpenOptions) -> Result<OpenedIndex>;

impl OpenedIndex {
    pub fn read(&self, request: ReadRequest) -> Result<ReadResponse>;
    pub fn changes(&self, request: ChangeRequest) -> Result<ChangePoll>;
    pub fn refresh(&self, paths: &[RelativePath]) -> Result<RefreshResult>;
    pub fn prioritize(&self, paths: &[RelativePath]) -> Result<PriorityResult>;
    pub fn close(&self) -> Result<()>;
}
```

The exact names may follow existing fdu vocabulary during implementation, but the
cardinality and ownership are fixed.
There is no callback per entry and no async executor in core.
Blocking waits use standard threads, locks, condition variables, and cancellation.

The Python binding mirrors the five synchronous operations.
Calls that may block or perform substantial Rust work release the GIL. It does not hide
long-lived change polls in Python’s shared default executor.
The MetaBrowser adapter owns the async bridge described below, where iterator
cancellation and handle close have distinct lifetimes.

### Configuration and derived identity

`OpenOptions` separates semantics from execution policy.

The opened root binds one root path separately from its portable scope identity.
The root binding belongs to the session and snapshot header; it is not folded into a
cross-provider configuration digest whose path encoding could vary by platform.

Semantic scope includes values that change which filesystem facts a complete live index
can know and that the observation backend can enforce:

- hidden-component admission and exact-name allowlist;
- whether symlinks are followed;
- filesystem-boundary policy;
- admitted object kinds;

MetaBrowser’s `max_depth` is not part of that scope.
It bounds a tree read and browser rendering, so Phase 3 moves it into query selection.
The opened root discovers at unbounded depth, and a depth-limited read does not change
cache identity or observer admission.

The initial live fdu provider supports the observation-compatible scope MetaBrowser
uses: hidden components are filtered lexically, symlinks are retained but not followed,
files/directories/symlinks are admitted, and filesystem boundaries are crossed.
The joint contract names all four decisions even where v1 accepts only that value.
A future restricted filesystem-boundary mode must either remain nonwatching or amend the
observer design before it can be selected; the adapter returns a typed unsupported-scope
error instead of promising a watch it cannot enforce.

Execution policy includes values that change how or how quickly the answer is found:

- scan order and worker count;
- discovery resource budget;
- observation mode;
- journal and continuation capacities;
- progress batching and scheduling hints.

Classification and ignore rules change answer semantics but not filesystem scope.
They contribute to a semantic identity, not the scope identity.
The opened root reports both identities with every engine version.

Both identities are derived internally from validated values.
The semantic identity combines the normalized registry, fixed ignore semantics, and
versioned reducer behavior; it does not concatenate hashes asserted by the caller.

MetaBrowser must pass the actual File Rollup registry document at open.
Each provider parses and validates that document and derives the registry fingerprint it
returns. A caller-supplied fingerprint is never accepted as proof of content.
The Python and fdu providers must agree on normalized registry identity through the
shared conformance packet.

The first adapter admits files, directories, and non-followed symlinks.
Other filesystem objects remain available to ordinary fdu callers but are outside the
MetaBrowser inventory contract.
The adapter is also the only place that translates fdu’s native relative-path identity
to MetaBrowser’s canonical POSIX-relative strings.
It validates that conversion once at the boundary and never creates aliases.
Representable components become Unicode scalar strings joined by `/`; Windows native
separators are structure, not path content.
Invalid Unix byte sequences and Windows unpaired surrogates are unrepresentable.

Unrepresentable entries remain in native fdu facts and roll-ups.
Portable row projections omit them and return a typed issue containing an exact count
and bounded escaped examples.
A portable directory with such children is incomplete even when native discovery is
complete; lookup below that directory returns `unknown`, not `absent`, when the missing
name could lie in the unrepresentable sibling set.
The conformance packet includes invalid Unix bytes and Windows Unicode/separator cases,
and verifies counts, portable completeness, issues, and knowledge state.

### Version, state, and knowledge

Every committed state has an `EngineVersion` containing:

- an opaque identity unique to one opened-root lifetime;
- a monotonically increasing sequence;
- the validated scope identity;
- the validated semantic identity.

The identity is not a security credential.
It exists to reject cursors and expected versions from another root or process.
It should not require token signing or a new cryptographic dependency.

The state record keeps orthogonal facts orthogonal:

- **phase:** opening, discovering, reconciling, ready, watching, stopped, or failed;
- **coverage:** complete or partial with a typed reason;
- **freshness:** fresh, reconciling, stale, or partial;
- **source:** scanned, revalidated, journal-scoped, or cached;
- **progress:** bounded counters and current work frontier;
- **issues:** a bounded list of typed issues with bounded detail.

Phase is a fact, not a monotonic success latch.
For example, refresh may move freshness to reconciling while phase remains watching.
A read returns the state captured at the same version as its projections.
A change batch returns the terminal state captured with its journal range.
Neither surface performs a second state read after releasing the observation guard.

Phase 3 aligns the adapter-visible values rather than asking the adapter to invent a
lossy mapping:

| fdu value | MetaBrowser value after the contract amendment | Rule |
| --- | --- | --- |
| phase `opening` | `OPENING` | Rename prototype `OPENING_CACHE`; opening is not necessarily a cache read. |
| `discovering`, `reconciling`, `ready`, `watching`, `stopped`, `failed` | same named lifecycle value | Exhaustive one-to-one mapping. |
| coverage complete or partial with `building`, `budget`, `cancelled`, `inaccessible`, or `failed` | same boolean and reason | No reason folding. |
| freshness `fresh`, `reconciling`, `stale`, or `partial` | same named freshness value | Per-path `unknown` remains knowledge state, not freshness. |
| source `scanned`, `revalidated`, `journal_scoped`, or `cached` | same named source value | A provider may support only a subset, but cannot rename one. |

The contract suite fails if either enum gains an unmapped value.

Entry lookup uses three-valued knowledge:

- `present` when the entry is retained;
- `absent` only when relevant coverage is complete;
- `unknown` when a budget, failure, unsupported scope boundary, unrepresentable sibling
  set, or unfinished discovery prevents an absence claim.

Directory completeness is recorded separately from global coverage so a discovered
subtree can be answered honestly before the full baseline is complete.

### One exact commit path

All producers use the same transition pipeline:

```text
verified observations or state transition
  -> validate and normalize against current scope/control state
  -> prepare exact mutation outside the write guard where possible
  -> atomically apply facts, roll-ups, state, and control changes
  -> Commit { version, effective_changes, impact, state, work }
  -> bounded journal
```

Mutation helpers return the exact operations they performed.
They do not return a boolean and ask the caller to copy the requested observation into
the journal. The clock advances only when the complete commit is ready.
No callback, watch sink, or progress stream reconstructs change truth separately.

Producers emit verified parent directories before children.
An unknown parent is not silently synthesized with guessed metadata.
If an observer reports a child below unknown ancestry, the engine schedules verified
reconciliation from the nearest known ancestor before admitting the child.

Control files are first-class verified inputs even when ordinary visibility rules would
exclude their directory entries.
For v1, a removal-aware per-directory control table stores the exact `.gitignore` source
and derived matcher state.
Discovery, refresh, and observation update that table through the same commit path.
Reclassification of affected retained entries is atomic with the control-state change.

The public commit impact vocabulary is stable and fdu-native:

- topology;
- metadata;
- classification;
- aggregates;
- trust or lifecycle state.

It also carries bounded dirty paths or `all_dirty`. Impact is derived from the exact
effective changes and state transition inside the prepared commit; producers do not
supply a second account of what they think changed.
The MetaBrowser adapter maps these domains and requested projections to its own
`QueryKind` invalidations.
fdu does not store MetaBrowser query names in its journal.

### Discovery and resource limits

Cold discovery commits bounded batches so reads can observe useful data before the walk
finishes. Parents are committed before children, and shallow scheduling is the default
because it makes early directory navigation useful.
`prioritize(paths)` may reorder pending work but never changes scope, facts, version, or
answer semantics.

The current `max_files` concept becomes a discovery resource budget rather than a scope
component. The first implementation supports a file-retention limit because that is the
demonstrated client need; the type leaves room for other measured resource limits
without inventing them now.

The contract is:

- at most the configured number of files are retained;
- reaching the limit does not itself prove truncation—the walker must encounter
  additional admissible work before reporting partial coverage;
- once additional admissible work is refused, coverage becomes partial with a typed
  resource-budget issue and absence outside complete directories is unknown;
- concurrent traversal is not required to choose the same retained prefix across
  providers;
- a resource-stopped partial session remains readable but does not begin observation or
  accept refresh that could expand the retained set;
- the caller reopens with a larger budget to continue.

This avoids free-slot accounting, historical admission order, and a live partial-watch
semantics that neither current provider implements correctly.
Complete answers must still agree across providers.
Tests assert the bound and honest state, not an accidental concurrent prefix.

Partial baselines are not persisted as reusable snapshots.

### Discovery-to-observation handoff

One owner and one session cover the entire opened-root lifetime.
There is no progressive session followed by a separate watch session.

Where the backend permits it, the observer is captured before baseline discovery and its
hints are buffered in a bounded queue.
The engine then:

1. commits the cold baseline in bounded batches;
2. records overflow or backend gaps as recovery requirements, not consumer-history
   resets;
3. reconciles buffered hints and any required subtrees;
4. performs a final verification pass for backends whose per-directory registration
   cannot be established atomically;
5. enters `watching` and marks freshness `fresh` only after the gap is closed.

A backend can provide native events, polling hints, or scripted test observations.
Every hint is verified before mutation.
Provider recovery and consumer journal loss remain separate facts:

- a provider gap triggers reconciliation and a trust-state transition;
- a slow consumer older than the retained journal floor receives `reset` and rereads
  current projections.

The `watch` feature must remain deletable.
Without it, the same opened root discovers, serves reads, refreshes explicitly, and
closes; it simply never enters live observation.

### Coherent reads

`read()` accepts a bounded list of fdu-native projections and returns them under one
observation boundary with:

- engine version;
- coherent state;
- projection results in request order;
- bounded work accounting;
- a change cursor for resuming after that version.

Every interactive projection has both an output bound and a deterministic work bound,
such as a maximum number of index rows visited.
Exhausting that budget returns a typed query-limit result for row projections; it never
silently relabels a partial calculation as exact.
Repeated client aggregates must use maintained indexes so their normal exact path stays
within the bound. No read performs an unbounded full-index traversal while holding the
writer guard. Expensive preparation may use immutable indexes or resumable state outside
the short commit critical section, but the returned answer must still be pinned to one
version.

Product totals have a separate bounded result because a missing denominator and a known
lower bound are different user experiences.
The initial exact totals are backed by named commit-maintained structures:

- a timestamp-ordered multiset supports exact recency-window counts and ranked recent
  rows;
- global and per-directory extension, canonical-extension, family, and `all` versus
  `unignored` tallies support the catalog and navigation predicates the client already
  uses;
- hierarchical roll-ups support exact directory totals.

An aggregate outside that maintained set receives a request `count_cap` bounded by a
server maximum. Its result is either `exact(n)` or `at_least(n)`; the latter renders as
`n+` and is never cached or serialized as an exact total.
The initial MetaBrowser default cap is 10,000. Adding another exact total therefore
requires either a maintained index with measured cost or an explicit capped product
contract.

The first native projection vocabulary is:

1. **Lookup:** one retained entry or three-valued absence.
2. **Tree page:** bounded structural rows below a path, including the ancestors needed
   to render them and per-directory completeness.
3. **Flat entry page:** bounded portable-path-ordered entries under a selection, with
   compact or full row shape.
4. **Roll-up/report:** existing fdu `Query` selection and view machinery for bounded
   summaries, breakdowns, navigation tallies, and ranked slices.
5. **Diagnostics:** fixed-size provider and lifecycle diagnostics.

This is enough to map MetaBrowser’s application queries without reproducing them in
core:

| MetaBrowser query | fdu adapter operation |
| --- | --- |
| Entry | Lookup |
| Directory | Tree page at depth one |
| Filtered tree | Tree page plus roll-up/report at the same version |
| Roll-up | Roll-up/report |
| Navigation | Roll-up/report with maintained aggregate indexes |
| Recent | Bounded ranked report |
| Catalog | Flat entry page with compact fields and selection predicates |
| Diagnostics | Diagnostics and the coherent envelope |

Navigation and other repeated aggregates remain provider-owned.
The adapter must not compose them by looping over returned entries.

### Paging and continuations

Page continuations are opaque IDs into a bounded table owned by the opened root.
A continuation record contains the pinned engine version, normalized query identity, and
the data structure’s resumable traversal position, such as the last visited path rather
than merely the last emitted match.
The public token does not carry trusted totals, paths, sort keys, or request structure.

The rules are:

- a page has a positive maximum row count;
- a nonterminal page returns a continuation that advances without rescanning the root;
- a later page fails with a typed stale-version result if the index has changed;
- an evicted or foreign continuation fails with a typed unavailable result;
- close clears the continuation table;
- a continuation belongs to one provider handle and one MetaBrowser host generation;
  root replacement or host-generation change discards it before the new generation is
  published;
- the table and each stored record are bounded;
- no historical index image is retained merely to satisfy a stale page.

Ordering is part of the joint contract:

- a tree page is parent-first.
  Within each directory, directories precede nondirectories, and each partition is
  ordered by canonical component UTF-8 bytes;
- a flat or catalog page is lexicographic by the complete canonical POSIX-relative path
  encoded as UTF-8 bytes.

The tree projection pays through the retained hierarchy and two bounded child
partitions. The flat projection pays through a commit-maintained ordered index of
representable portable paths; it never materializes and sorts the full catalog per
request. Unrepresentable native paths follow the explicit partial-row semantics above.
Other resumable sort orders are deferred until a measured client need justifies their
own maintained index.

The joint provider contract drops exact `remaining_rows` and repeated catalog totals
from page control flow.
Conformance proves conservation by assembling all pages and checking advancing cursors,
row bounds, no duplicates, and exact final contents.
When the UI needs a filtered total, it receives a separate aggregate projection bundled
with the first page at the same engine version.
That projection returns an exact maintained total or the explicit capped-count result
defined above.

### Change polling

The owner keeps one bounded journal of commits.
`changes(after, timeout)` is a pull operation over that journal:

- it returns immediately when newer commits exist;
- otherwise it waits on a condition variable until a commit, close, cancellation, or
  timeout;
- timeout returns an idle result without advancing the cursor;
- state-only commits are observable;
- a cursor older than the journal floor returns a consumer reset at the current coherent
  version and state;
- a foreign or future cursor is rejected rather than treated as an empty range;
- close wakes all waiters.

No per-subscriber engine queue is required.
MetaBrowser may keep its own bounded SSE bus after the coordinator has translated the
provider batch; that is a different owner and recovery boundary.

The first change result carries bounded invalidation information rather than entry
replicas because MetaBrowser rereads coherent projections.
An entry-delta interest can be added later for a client that demonstrates it needs to
maintain a replica.

### Refresh

`refresh(paths)` verifies a bounded set of canonical relative paths.
It validates and deduplicates the input, collapses descendants when an ancestor already
covers them, performs filesystem I/O outside the write guard, and conditionally commits
against the version it observed.

Its result reports:

- the terminal engine version and state;
- accepted and rejected paths with typed reasons;
- exact effective changes or their journal range;
- bounded work.

Refresh cannot bypass admission, control-file handling, the resource budget, or the
exact commit pipeline.
Overlapping refresh, discovery, and observer work may do redundant verification, but
only conditional commits can change retained truth.

### Prioritization and close

`prioritize(paths)` validates a bounded request and changes only discovery scheduling.
It is idempotent, best effort, and legal only while prioritizable work exists.
It does not advance the semantic clock.

`close()` is idempotent and safe under concurrent callers.
It cancels discovery, observation, refresh, and blocking change waits; joins all owned
workers; clears continuations; and leaves no background writer able to reach the index.
A timed or async wrapper may report that shutdown failed, but it must never claim close
completed while a worker remains live.

### Cache and provenance boundary

The first streaming implementation is cold-only.
It can use the current cache path for a blocking complete open, but it does not mix a
warm image with live cold facts while reporting one global provenance value.

That restriction is deliberate.
Per-value provenance is only useful if every roll-up can be composed and invalidated
correctly across sources.
PR #47’s provenance state exposed that this is its own design problem, not a field to
add to the current lifecycle.

A later warm-streaming design must establish trust per scope or per subtree and prove
that aggregate provenance composes under updates.
Until then, a stale or incompatible complete snapshot fails closed to verified
reconciliation or cold discovery, and a partial snapshot is never served as complete.

### Features, dependencies, and binary size

`fdu-core` remains usable with no default features.
The CLI and Python package may opt into `watch` and `gitignore` explicitly.
The new owner, journal, blocking reads, refresh, and continuation table use the standard
library where practical.

Any dependency addition must follow the supply-chain policy and include a before/after
measurement of:

- compressed and installed CLI binary size;
- Python wheel size on supported platforms;
- default and `--no-default-features` dependency trees;
- cold open and steady-state memory on the MetaBrowser corpus.

The first implementation adds no async runtime, web protocol, serialization framework,
or token-signing dependency to core.

### Required MetaBrowser changes

MetaBrowser keeps the architecture in PR #74, including its coordinator, sparse host
overlay, eight application query variants, and five-operation provider handle.
The following changes make that boundary honest and remove work that exists only to
defend the current prototype contract.

#### Provider contract values

- Replace the input `registry_fingerprint` with the immutable File Rollup registry
  document. A provider returns the identity it derived after parsing; it cannot claim an
  identity supplied beside different content.
- Replace semantic `max_files` with an execution-policy `DiscoveryBudget`. The initial
  field can remain a file limit, but the name and state describe resource exhaustion,
  not a deterministic cross-provider scope prefix.
- Remove the discovery budget from `inventory_scope_fingerprint()`.
- Move maximum depth to tree-query selection; remove it from
  `inventory_scope_fingerprint()` and from opened-root cache identity.
- Keep hidden admission as explicit scope and add explicit symlink behavior,
  filesystem-boundary behavior, and admitted object kinds to `InventoryConfig`. V1
  validates the fixed observation-compatible values described above rather than leaving
  provider defaults implicit.
  Do not reproduce a private fingerprint recipe in the fdu adapter; compare the
  identities each provider derives from the same validated values.
- Version the new scope-fingerprint encoding.
  Removing `max_files` and `max_depth` while adding the three explicit scope fields
  intentionally invalidates every prototype fingerprint and persisted provider cache;
  both unmerged providers rebuild rather than carry a compatibility recipe.
- Remove `remaining_rows` from directory, filtered-tree, and catalog page projections.
  Keep `next_page` as opaque provider state.
  Move UI-visible totals to an explicit aggregate projection at the same version.
- Pin tree and flat row order to the two exact definitions above.
  Both providers return that order directly; the coordinator never resorts assembled
  pages.
- Add the typed unrepresentable-path issue and portable-directory completeness rule.
  Native roll-ups still include every retained entry.
- Add a deterministic work budget to potentially scanning queries and a typed
  query-limit result. Output bounds alone do not protect event-loop latency.
- Add an exact-or-capped count result.
  Recency, navigation, and catalog totals use the maintained indexes named above; an
  unmaintained compound total returns `at_least(n)` at the request cap rather than
  dropping the denominator.
- Align lifecycle, coverage, freshness, and source values with the exhaustive mapping
  above, including renaming prototype `OPENING_CACHE` to `OPENING`.
- Preserve caller-pinned `as_of` reads, coherent state/version, typed payloads, bounded
  issue detail, and the closed eight-query application algebra.

These are internal provider changes on an unmerged feature branch.
There is no reason to retain compatibility shims for the prototype spellings.

#### Python reference provider

- Store the parsed registry on the opened provider instead of calling module-global
  classification helpers backed by the packaged default.
  Every classification and roll-up reads the injected registry.
- Enforce one resource-budget policy across initial discovery, targeted rewalk, refresh,
  and observer application.
  Once discovery is partial because work was refused, leave the handle readable but do
  not start its watcher or admit expanding refreshes.
- Derive semantic and scope identities from the same validated values exposed by the
  handle; remove caller-asserted identity fields.
- Change page memoization to return a bounded page and opaque next position without
  counting a suffix for control flow.
  Keep exact product totals only in maintained aggregate projections, and return an
  explicit capped count for an unmaintained compound total.
- Retain the Python provider as the readable reference implementation.
  Favor direct code over imitating fdu’s internal owner, journal, or continuation types.

#### Coordinator, assembly, and routes

- Keep root replacement and host overlay in `InventoryCoordinator`; replacing a root
  closes and joins the old handle before publishing the new host generation.
- Discard every old-handle continuation before publishing a replacement root or host
  generation. A continuation is never retried against the new handle.
- Replace exact-remainder assembly checks with a bounded assembly loop: enforce a stable
  provider version, positive page size, unique advancing continuations, maximum pages,
  maximum assembled rows, and the request work budget.
- Read product totals as a separate projection rather than trusting page-control
  metadata. A route that needs rows and a total requests both in one coherent read.
- Keep provider-history reset distinct from host SSE replay loss and from a provider
  observation gap. Each owner recovers only its own boundary.
- Keep `QueryKind` and host-cache invalidation in MetaBrowser.
  The fdu adapter owns one exhaustive, tested mapping from fdu impact domains to those
  application kinds.
- Remove any coordinator fallback that walks entries, reconstructs roll-ups, or applies
  provider row replicas.
  On invalidation, it rereads the bounded projections it serves.

#### fdu provider adapter and packaging

- Implement `FduInventoryBackend` and its private handle in MetaBrowser, not in
  `fdu-core`. The adapter maps config, paths, eight queries, state, rows, work, and
  invalidations without retaining a second filesystem index.
- Give each opened provider handle a dedicated one-worker change-poll executor and a
  bounded queue to its single active async change iterator.
  The native call releases the GIL and uses a poll timeout no greater than 250
  milliseconds. Queue backpressure does not advance the delivered cursor; journal
  eviction therefore recovers through the ordinary consumer-reset result.
- `aclose()` on the change iterator cancels and joins only that bridge within one poll
  interval. It does not close the provider handle, and a later bounded read on the same
  handle must succeed.
  Handle `close()` cancels the iterator, joins the bridge, and then joins the native
  owner. A second simultaneous change iterator fails with a typed busy result.
- Pass the registry document and explicit MetaBrowser scope at open, then use the
  identities returned by fdu.
  Do not hash a second spelling of either contract.
- Translate fdu continuation IDs only as opaque strings.
  MetaBrowser neither decodes nor rewrites them, and never retains them across root
  replacement or host-generation change.
- Make the fdu package an explicit optional provider dependency until rollout.
  Selecting `fdu` when the extension is unavailable produces a typed startup error; it
  never silently chooses Python.
- In cross-repository CI, build the fdu wheel from the exact checked-out revision and
  install that artifact.
  Do not depend on a moving Git branch or copy Rust artifacts into the MetaBrowser tree.
- Keep the Python provider as the oracle and rollback choice until the dedicated
  integration phase passes.

## Reuse and Disposition of Existing Work

PR #47 should remain available until the replacement slices land.
Code is selected by invariant and retested in its new owner; commits are not merged in
bulk.

| PR #47 area | Disposition |
| --- | --- |
| Runtime registry parser, logical/canonical extension derivation, browsing groups | Extract, then change open to consume actual registry content and derive identity. |
| Typed lifecycle state and coherent read envelope | Extract after simplifying the owner and state clock. |
| Bounded journal and no-history reset | Extract behind the exact commit pipeline. |
| Shared guarded reads and GIL release | Extract and retain. |
| Scripted watcher, hidden/special-object tests, scope fixtures | Reuse as tests against the new producer boundary. |
| Bounded refresh, prioritization, joined close | Reimplement around the one owner; reuse value types and focused tests where sound. |
| Requested-observation `AppliedDelta` and implicit-parent mutation | Replace. |
| Mutable clone/session/continuation authority | Replace. |
| Stateless signed page token | Replace with bounded handle-local continuation state. |
| Exact global `max_files` prefix and live free-slot semantics | Replace with honest discovery-budget behavior. |
| MetaBrowser query names in fdu journal or adapter-facing core values | Replace with stable fdu impact domains and native projections. |
| Generic tags and promoted planes | Defer; retain only the demonstrated unignored partition. |
| Per-value progressive provenance | Defer pending a separate trust design. |
| Sorted resumable pages, lazy snapshot blocks, expanded diagnostics | Defer until measured or required. |
| CLI progress and new goldens | Defer until the opened-root engine and client adapter are proven. |

## Implementation Plan

All fdu phases use `codex/opened-root-inventory-rewrite` and one draft PR rooted at
current `main`, not PR #47. Each checkbox is independently reviewable, each phase ends
in a named green commit checkpoint, and work does not advance across a failed phase
gate. The PR remains a draft until Phase 4 passes.

This merge topology explicitly overrides the review report’s preferred sequence of a
core-integrity PR followed by an opened-root PR. The project owner selected one
cumulative branch after considering that recommendation so the cross-repository effort
has one fdu head, one MetaBrowser counterpart pin, and no stacked-PR base churn while
the contract is still moving.
That choice does not make the final merge unit small: phase checkpoints improve review
timing and bisectability, not total diff size.
Reviewers must approve each named checkpoint and the final accumulated diff.
If a phase cannot remain green or independently understandable, implementation stops and
the merge-topology decision is reopened rather than weakening a gate.

MetaBrowser changes in Phases 3 and 4 land on its own PR #74 branch and are tested
against the exact fdu draft-PR revision recorded in both PR descriptions.

### Phase 1: Exact Engine Kernel

Phase 1 has four separately reviewed green checkpoints.
No checkpoint changes default CLI behavior or exposes an incomplete MetaBrowser
provider. Code is reimplemented from `main` or extracted in minimal pieces from PR #47;
no PR #47 commit is cherry-picked wholesale.

#### Checkpoint 1A: Observable Oracle

- [ ] Fix `fdu-9tdm`: replace surgical golden parsing with broad observable-output
  assertions before importing new surface behavior.
- [ ] Add an independent deterministic reference model for retained facts, parent
  ordering, roll-ups, exact changes, control-file updates, and resource refusal.
- [ ] Gate with the focused model tests, complete golden corpus, CLI/Python parity,
  `make docs-format-check`, and `make check`.

#### Checkpoint 1B: Exact Commit Truth

- [ ] Introduce prepared mutations and one atomic `Commit` containing exact effective
  changes, impact, state, and work.
- [ ] Route scan, reconcile, explicit refresh, control-file updates, and existing watch
  application through that commit path.
- [ ] Remove implicit guessed-parent mutations from the live path; normalize unknown
  ancestry through verified reconciliation.
- [ ] Gate with generated operation-sequence comparison, fault injection, concurrent
  reader/writer tests, `make check`, and `make cross-lint`.

#### Checkpoint 1C: Control State

- [ ] Add the exact removal-aware `.gitignore` control table and the fixed
  `all`/`unignored` partition behind a removable feature.
- [ ] Introduce the runtime registry/classification pieces needed by the fixed partition
  behind explicit features; preserve the no-default-features build.
- [ ] Gate creation, edit, deletion, hidden-control discovery, provider-order
  equivalence, all feature combinations, and dependency audit.

#### Checkpoint 1D: Live Identity and Feature Floor

- [ ] Separate detached index snapshots from live session and continuation authority.
- [ ] Keep core default features empty and record dependency and binary-size baselines.
- [ ] Prove the kernel under operation-sequence tests, fault injection, model
  comparison, and all existing one-shot surface parity tests.
- [ ] Gate clone/detached-image identity, continuation authority, snapshot round-trip,
  `make check`, `make cross-lint`, dependency audit, and size baselines.

Acceptance for Phase 1:

- every retained mutation and state transition is represented exactly once in its
  commit;
- applying the same normalized operation sequence to the reference model and engine
  produces identical facts, roll-ups, state, and effective changes;
- no public clone can mutate a divergent index while sharing live identity;
- current CLI and Python one-shot answers remain unchanged unless a reviewed golden
  records an intentional correction;
- `make check`, `make cross-lint`, and dependency audit pass.

### Phase 2: Opened-Root Vertical Slice

- [ ] Add the one-owner `OpenedIndex` with idempotent joined close.
- [ ] Implement cold progressive discovery with parent-first bounded commits,
  per-directory completeness, explicit budget state, and scheduling priority.
- [ ] Add coherent lookup, depth-one tree, roll-up, state, version, and work projections
  in one bounded `read()`.
- [ ] Add the bounded pull journal and `changes(after, timeout)` with state-only
  commits, cursor validation, timeout, reset, and close wakeup.
- [ ] Add bounded verified multi-path refresh through the shared commit pipeline.
- [ ] Add native observation with capture-before-baseline buffering, scripted overflow,
  final reconciliation, and a no-gap transition to watching.
- [ ] Mirror the five synchronous operations in Python with GIL release; keep the
  long-lived async change bridge in the MetaBrowser adapter.
- [ ] Prove shutdown, concurrent reads and commits, slow consumers, provider gaps,
  resource-stop behavior, and every supported feature combination.

Acceptance for Phase 2:

- a client can open a cold tree, render a shallow directory and roll-up before discovery
  completes, resume changes from that read, refresh paths, reprioritize work, and close;
- no filesystem mutation in the baseline-to-observer handoff is silently lost;
- every bounded output has a bounded-work test or counter oracle;
- watch-disabled core builds and behaves correctly;
- cancelling the MetaBrowser async change iterator without closing the handle joins its
  bridge within one native poll interval, and a read on that same handle still succeeds;
- handle close leaves no Python or native worker alive.

### Phase 3: MetaBrowser Contract and fdu Adapter

Phase 3 starts with measurement against the unchanged MetaBrowser contract before either
provider contract is edited.
This applies the repository’s instrument-before-optimizing rule to the API boundary.

#### Checkpoint 3A: Unchanged-Contract Cost Spike

- [ ] Implement a disposable fdu adapter over the Phase 2 handle against MetaBrowser PR
  #74’s unchanged provider contract.
  Materializing catalog rows, sorting, and scanning for totals are allowed only in this
  spike and are instrumented explicitly.
- [ ] Run the existing route and provider tests on the representative corpus; record
  route predicates, rows visited, sort/materialization work, latency, memory, and which
  totals and orders are product-visible.
- [ ] Publish the evidence before changing either contract.
  Keep the reusable harness and evidence; delete the naive aggregation and replica code.

#### Checkpoint 3B: Joint Contract and Reference Provider

- [ ] Amend MetaBrowser’s provider contract so the registry content is an input,
  discovery budget is execution policy with honest partial state, maximum depth is
  selection, scope fields and state values are explicit, row orders are exact, count
  bounds are honest, and page assembly does not require exact remainders.
- [ ] Update the Python reference provider to use its injected registry and the revised
  budget, identity, work-limit, and page contracts.
- [ ] Update coordinator and route assembly to use bounded continuation safety and
  coherent aggregate totals without reintroducing filesystem or aggregation ownership.
- [ ] Gate the revised Python provider and all existing routes before adding fdu.

#### Checkpoint 3C: Native Indexes and Thin Adapter

- [ ] Add path-ordered tree and flat-entry continuations backed by the bounded
  handle-local table.
- [ ] Complete the existing fdu query indexes needed for filtered tree, navigation,
  recent, catalog, and diagnostics without per-request Python aggregation.
- [ ] Implement a thin `FduInventoryBackend`/handle mapping the eight MetaBrowser
  queries and fdu impact domains to the existing application contract.
- [ ] Package fdu as an explicit optional provider and make missing or incompatible
  native artifacts a typed startup failure.
- [ ] Keep provider selection explicit and the Python provider as the default; do not
  add automatic fallback.

Acceptance for Phase 3:

- the revised Python provider passes MetaBrowser’s provider contract suite;
- the 3A evidence names the work eliminated by every maintained native index or contract
  change; no index is justified only by conjecture;
- the fdu adapter contains no walker, aggregate store, row replica, fingerprint recipe,
  or provider-independent application policy;
- every MetaBrowser query maps to one bounded coherent fdu read rather than a Python
  entry loop;
- iterator-only cancellation preserves a usable handle; root replacement, handle close,
  cancellation, and package-unavailable errors are explicit and leave no worker alive;
- both repositories build independently, and MetaBrowser without the optional fdu
  package retains its current Python-provider behavior.

### Phase 4: End-to-End Integration Proof

This phase changes no default.
It proves the composed product through the same public routes and lifecycle that the
browser uses, then produces the evidence required for a separate rollout decision.

- [ ] Run the same provider conformance registry against Python and fdu providers.
- [ ] Expand the File Rollup packet to include basename-to-logical-extension derivation,
  not only rows whose logical extension is already supplied.
- [ ] Add cross-platform path fixtures for invalid Unix bytes, Windows separator
  normalization and unpaired surrogates, non-ASCII Unicode, and portable-directory
  completeness around unrepresentable children.
- [ ] Assert the exact tree and flat ordering contracts, exact maintained totals,
  explicit capped totals, and exhaustive state-value mapping in both providers.
- [ ] Replay one recorded, verified observation script through both providers and
  compare complete reads after every step.
- [ ] Parameterize MetaBrowser’s route-level inventory tests over both providers for
  initial tree, filtered tree, roll-up, navigation, recent, catalog, and diagnostics.
- [ ] Run the existing browser-lifespan and filesystem-to-SSE integration tests with fdu
  selected, covering progressive discovery, change invalidation, refresh, root
  replacement, and shutdown.
- [ ] Add integration faults for discovery-budget exhaustion, stale and evicted page
  continuations, consumer journal reset, provider observation recovery, unavailable
  native package, iterator-only cancellation, cancellation during close, and attempted
  continuation reuse after root replacement.
- [ ] Verify an installed wheel rather than only a source-tree import, including the
  supported Python and platform matrix.
- [ ] Measure cold usefulness, settled answers, change latency, request latency, memory,
  CPU, dependencies, CLI size, and wheel size on the same corpus.
- [ ] Make fdu the default provider only after correctness and performance acceptance is
  explicit in both repositories.

Acceptance for Phase 4:

- both providers pass the same closed conformance registry;
- complete settled responses and replay checkpoints agree exactly;
- representable path rows, unrepresentable-path issues, roll-up counts, row order,
  aggregate bounds, and state vocabulary agree on every platform fixture;
- partial sessions agree on bounds and knowledge state without requiring the same
  concurrent retained prefix;
- MetaBrowser performs no replacement walk, retained aggregation, or entry-replica
  application in its coordinator;
- route and filesystem-to-SSE tests return the same public application envelopes through
  both providers, apart from explicitly provider-specific diagnostics and work;
- discovery, a live filesystem mutation, refresh, root replacement, and close are each
  exercised in one installed-artifact lifecycle test;
- every injected gap or stale cursor takes the documented recovery path without a lost
  update or silent fallback;
- provider selection, rollback, and error reporting are explicit;
- both repositories’ full gates and supported-platform CI pass.

## Testing Strategy

### Model-based engine tests

Build the independent model before the live owner.
Generate deterministic sequences containing upserts, removes, kind changes, out-of-order
observations, control-file creation and removal, resource refusal, refresh overlap, and
state-only transitions.
After every operation compare facts, roll-ups, coverage, state, clock movement, and
exact effective changes.

The model must not call production mutation helpers.
That preserves its value as an oracle rather than a second spelling of the same code.

### Deterministic lifecycle tests

Use injected discovery order, worker count, observation scripts, barriers, and bounded
queues to force:

- an event before baseline starts;
- an event during a baseline batch;
- queue overflow;
- per-directory observer registration gaps;
- concurrent direct refresh and observer reconciliation;
- state-only commits while a consumer waits;
- journal-floor reset;
- close while discovery, refresh, and change polling are blocked.

Timing-only sleeps are not proof of these interleavings.

### Projection and continuation tests

For every pageable projection assert:

- positive and enforced row bound;
- proportional page work rather than a repeated full selection pass;
- advancing continuation;
- no duplicates or omissions when the version is stable;
- typed stale-version, foreign-token, and eviction failures;
- bounded table memory and cleanup on close.

Product totals are tested as aggregate projections at the same version, not inferred
from page-control metadata.

### Cross-provider conformance

MetaBrowser’s provider registry remains the application-level oracle.
Both implementations must pass the same test cases, including lifecycle, coherent reads,
exact predicates, paging, changes, reset, refresh, close, and budget truth.

The shared File Rollup packet is pinned to a reviewed MetaBrowser revision and exercised
by both packages. Git history already identifies its source; a second hand-maintained
hash beside the vendored data is unnecessary unless the fixture crosses an actual
untrusted boundary.

Recorded observation replay compares both engines after every step.
It is stronger than comparing only final rows because it catches invalidation, state,
and cursor drift.

### Composed MetaBrowser integration matrix

The integration phase runs at progressively wider boundaries so a failure identifies its
owner:

| Boundary | Evidence | Pass condition |
| --- | --- | --- |
| Provider values | MetaBrowser’s provider contract registry, parameterized over Python and fdu | Every query, lifecycle value, page, change, refresh, and close case has the same semantic result. |
| Classification | Shared File Rollup corpus using the actual registry document | Registry identity, logical and canonical extension, family, group, and roll-up rows agree. |
| State transitions | Recorded observation replay with a checkpoint after every step | Version movement, knowledge state, invalidations, and settled reads agree. |
| HTTP routes | Existing tree, filtered-tree, roll-up, navigation, recent, catalog, and diagnostics route tests | Public envelopes agree except for explicitly provider-specific work and diagnostic fields. |
| Live application | Existing browser-lifespan and filesystem-to-SSE tests | A cold open becomes useful, a real mutation reaches the browser-facing feed, refresh and root replacement converge, and shutdown joins. |
| Recovery | Deterministic budget, continuation, journal, observer-gap, package, and cancellation faults | Each boundary takes its documented recovery path with no lost update, false completeness, or silent fallback. |
| Distribution | MetaBrowser test environment with the built fdu wheel installed | Provider selection works from shipped artifacts on every supported Python and platform job. |
| Client regression | Existing provider-neutral browser and wire-shape tests | No provider-specific branch is added to browser JavaScript, and existing UI envelopes remain compatible. |

The installed-artifact lifecycle test is the release-level acceptance test.
It must perform open, progressive read, live mutation, change delivery, refresh, root
replacement, and close in one process with `fdu` explicitly selected.

### Surface and golden tests

The engine remains the source for Rust, CLI, and Python answers.
Existing broad golden and parity coverage protects the one-shot surfaces.
New CLI behavior is added only when the interactive engine is already proven, and its
goldens capture complete visible output with named portability patterns.

No test should parse one scalar out of a broad CLI response while ignoring adjacent
state, warnings, or totals that are part of the contract.

### Performance and size tests

Measurements are gates on adoption, not substitutes for correctness.
Record at least:

- time to first useful shallow directory during cold discovery;
- time to complete cold discovery;
- settled directory, navigation, catalog-page, and recent-query latency;
- work per continuation page;
- verified change-to-readable-version latency;
- retained memory and continuation-table memory;
- Python conversion and GIL-detached work;
- binary, wheel, and dependency deltas.

Use the repository’s performance protocol for claim-grade numbers and record the host,
cache, and filesystem regime.

## Rollout Plan

1. Freeze PR #47 as the prototype and review record.
2. Open one draft PR from `codex/opened-root-inventory-rewrite`, rooted at current
   `main`, containing this review and plan.
3. Complete Phase 1A through 1D on that branch and record every exact green checkpoint
   in the PR.
4. Complete Phase 2 on the same branch and record its exact green checkpoint.
5. Run the Phase 3A unchanged-contract cost spike, publish its evidence, then update
   MetaBrowser PR #74 for Phases 3B and 3C. Pin it to the current fdu revision and keep
   the Python provider as the default.
6. Add the fdu provider behind explicit configuration and run the complete Phase 4
   installed-artifact integration matrix across both exact PR heads.
7. Mark the fdu PR ready only after all four phase gates, the second-agent review, and
   both repositories’ CI pass; merge the complete rewrite as one PR.
8. Change the MetaBrowser default only in a dedicated, reversible change after both
   repositories record acceptance.
9. Consider explicit CLI progress separately after the client integration has proven the
   lifecycle.

There is no automatic fallback from a selected fdu provider to Python.
A failed open must be visible, because a silent fallback would hide packaging and
correctness regressions and make performance evidence ambiguous.

## Bead Reconciliation

The existing graph records valuable findings, but many bead descriptions encode PR #47’s
old shape. Implementation planning should update or supersede them rather than treating
their status as approval of that shape.

| Bead | Disposition in this plan |
| --- | --- |
| `fdu-a7cl` exact cap-refusal delta | Solved by the Phase 1 prepared mutation and exact commit invariant. |
| `fdu-0778` removal-aware gitignore control | Phase 1; keep the defect, replace the generic tag-plane solution. |
| `fdu-o8r8` independent reference model | First Phase 1 implementation prerequisite. |
| `fdu-9tdm` golden anti-pattern | First Phase 1 test-harness prerequisite. |
| `fdu-e86o`, `fdu-a0j0`, `fdu-4o0m` progressive sessions | Supersede with one Rust/Python `OpenedIndex` lifecycle in Phase 2. |
| `fdu-vfx7` watch carrier defects | Solve through journal-derived invalidations and terminal state, not row-carrying callbacks. |
| `fdu-97dd` file cap | Redefine jointly as a discovery resource budget; do not preserve exact-prefix or live-free-slot semantics. |
| `fdu-7sou` watch scope rejection | Preserve the invariant: MetaBrowser depth becomes read selection, its v1 live scope is observation-compatible, and unsupported restricted scopes fail explicitly. |
| `fdu-91ru` coherent paging | Keep the coherent envelope; replace stateless token authority and repeated full selection with bounded continuation state. |
| `fdu-t5h2` sorted resumable pages | Defer; path-order pages satisfy the immediate client. |
| `fdu-8w5k` catalog predicates | MetaBrowser head now pins its side; implement as fdu selection conformance in Phase 3, not as a new core query name. |
| `fdu-sgp7` prioritize and close | Phase 2 operations on the one owner. |
| `fdu-kl7r`, `fdu-vfyw` agreement proof | Phase 4 two-provider registry and observation replay. |
| `fdu-gy3g` File Rollup packet | Phase 4, expanded to exercise basename-derived logical extensions. |
| `fdu-livs` progressive provenance | Defer warm/mixed serving; cold streaming uses honest global source plus directory completeness. |

Implementation epic `fdu-snej` owns this plan.
Detailed implementation beads should be created from the four phases only after this
design is accepted, so the tracker does not pre-commit the code layout before the API
boundary is settled.

## Open Questions

These are measurement or rollout questions, not reasons to weaken the ownership and
commit invariants:

- Does the fdu provider need a polling observer before it can become MetaBrowser’s
  default on network filesystems, or can the Python provider remain the explicit choice
  for poll mode initially?
- Does the CLI opt into the `gitignore` feature by default after the binary-size and
  user value measurements, or only expose it in the Python/MetaBrowser build?
- Which later client, if any, justifies sorted resumable reports rather than bounded
  ranked top-N results?
- What evidence would justify a separate warm progressive design with per-subtree trust
  rather than the first version’s cold-only streaming?

## References

- [PR #47 design and merge-readiness review](../../reports/report-2026-08-25-pr-47-design-and-readiness-review.md)
- [fdu design principles](../../architecture/fdu-design-principles.md)
- [fdu surface architecture](../../architecture/fdu-surface-architecture.md)
- [Existing interactive-client contract at the reviewed PR #47 head](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md)
- [Existing implementation map at the reviewed PR #47 head](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md)
- [Progressive-results plan](plan-2026-08-11-fdu-progressive-results.md)
- [MetaBrowser provider architecture at reviewed head](https://github.com/jlevy/metabrowser/blob/3183888808b366b5ba1c381dec1cbb18b49d969e/docs/project/architecture/arch-inventory-provider.md)
- [MetaBrowser provider plan at reviewed head](https://github.com/jlevy/metabrowser/blob/3183888808b366b5ba1c381dec1cbb18b49d969e/docs/project/specs/active/plan-2026-08-23-inventory-provider-refactor-and-fdu-adoption.md)
- [MetaBrowser PR #74](https://github.com/jlevy/metabrowser/pull/74)
- [fdu PR #47](https://github.com/jlevy/fdu/pull/47)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
