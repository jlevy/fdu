# Feature: Opened-Root Inventory Engine Rewrite

**Date:** 2026-08-25

**Author:** fdu project, with Codex review assistance

**Status:** Active — Phase 3B contract and Python oracle complete; Phase 3C bounded
native projections in progress; Phase 4 control-state scale open after macOS field
reports

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

One `OpenedIndex` holds discovery, the retained index, the commit journal, optional
filesystem observation, continuation state, and worker lifetime.
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

[The engine architecture](../../architecture/fdu-engine-architecture.md) is the durable
design authority for this work.
This plan maps that architecture to phases, files, functions, reusable prototype units,
tests, and beads.

## Decision Summary

| Question | Decision |
| --- | --- |
| Merge PR #47 after more patching? | No. Preserve it as implementation evidence and extract selected pieces into one fresh rewrite branch from `main`. |
| How is the rewrite delivered? | One long-lived fdu draft PR with phase-gated commit groups; MetaBrowser changes remain on its own PR #74 branch. |
| Keep MetaBrowser’s provider boundary? | Yes. The coordinator/provider split and five-operation handle are sound. |
| Shape the Python API around MetaBrowser? | No. MetaBrowser is the reference client, not the public vocabulary. Rust and Python expose the same fdu-native lifecycle and values; the client adapter translates them. |
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
| Preserve the standalone CLI? | Yes. Existing one-shot behavior and output remain the default and retain full utility. The binary acquires no Python, async-runtime, or MetaBrowser dependency. Any later interactive mode is additive and calls the same public engine operations available to Rust and Python callers. |

The engine architecture and the detailed Design and phase acceptance sections are
normative for implementation.
The Decision Summary and Bead Reconciliation table are indexes and must be corrected if
they drift from those sections.
The review report owns the diagnosis and evidence; this plan owns later implementation
decisions, including the explicit delivery override recorded below.

## Current Implementation Status

This status describes the current Phase 3C checkpoint on fdu PR #48. The PR remains
draft while the native projections, MetaBrowser adoption, and composed proof proceed.
The registry parser and additive `EntrySelection` landed at `328ca65`, and the three
ordered-page contracts are now stated in
[Row order is stated here, not inferred from an implementation](#row-order-is-stated-here-not-inferred-from-an-implementation)
after implementation work found that none of them was written down: ordered pages were
specified as prose that did not separate level order from pre-order, ranked recency left
its tie to a stable sort’s input order, and MetaBrowser’s `contract.py` documented no
row order at all. Implementing against an unstated order is what the cross-provider
replay would have caught late, so the contract moved first.
The no-gap observation and five-session golden beads are complete.
The direct synchronous Python binding, typed public namespace, and installed-wheel
lifecycle are implemented and pass the full local handoff, distribution, parity, and
cross-target gates. MetaBrowser commit `45266a8` now carries the revised bounded
contract, provider-wide conformance registry, Python reference implementation,
coordinator assembly, route integration, and full application gate.

| Phase | Status | Evidence and next boundary |
| --- | --- | --- |
| Architecture and implementation map | Complete | The durable architecture, PR #44 and #47 reconciliation, direct-API correction, file/function map, test design, and reuse ledger are committed and reviewed. |
| Phase 1: exact engine kernel | Complete | Checkpoints 1A through 1D passed their local gates and the cumulative cross-platform PR gate. |
| Phase 2: opened-root vertical slice | Complete | The native lifecycle and five transparent session goldens are green. The direct `PyO3` handle, exhaustive value conversion, immutable `fdu.opened` API, typed errors, GIL-detached operations, strict downstream typing fixture, installed-wheel lifecycle, source distribution, CLI parity, and cross-target lint all pass. |
| Phase 3: MetaBrowser adoption | Checkpoint 3C in progress | MetaBrowser commit `2743064` measures the unchanged contract against the exact fdu wheel from `0583a1a`; `45266a8` completes the shared bounded contract and Python oracle. fdu `a286145` completes the approved optional serving-index set, and `27aeed0` completes the bounded continuation authority with green CI. The current native checkpoint parses the actual File Rollup v3 registry without adding a dependency, projects classification on demand, and adds the selection predicates needed by catalog and filtered reads. The bounded projection readers, Python registry input, and thin production adapter remain open. |
| Phase 4: control-state scale | Not started | Branch builds cannot roll up `~`, `~/wrk`, or `~/Library` on macOS while `main` can. The control table aborts the scan at a 4 MiB cumulative budget that `~/wrk` exceeds 2.4-fold. Epic `fdu-2lkf`. |
| Phase 5: composed proof | Not started | Cross-provider conformance, route and lifecycle integration, installed-wheel proof, and final performance and size acceptance remain required. |

The implementation epic has completed the native and Python Phase 2 dependency chain.
MetaBrowser adoption, composed integration, and final acceptance remain open.
The tbd graph is authoritative for live status and dependencies; the checklists and bead
table below are maintained as its readable plan view.

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
[surface architecture](../../architecture/fdu-surface-architecture.md), with the live
ownership and transition boundaries defined by
[the engine architecture](../../architecture/fdu-engine-architecture.md):

- defaults answer the stated question;
- no bound or truncation is silent;
- filesystem events are hints and verified observations are facts;
- one atomic `Commit` describes every exact effective change and observable state
  transition;
- queries are pure readers and scope is not selection;
- the watch feature remains removable;
- the core, CLI, and Python surfaces cannot disagree;
- visible CLI behavior is tested broadly rather than by surgically parsing only the
  value a test expects.

### Ownership of progressive work

This plan owns cold progressive discovery, the opened-root/session lifecycle, coherent
mid-discovery reads, the no-gap observation handoff, the immediate Python client
contract, and MetaBrowser adoption.

The older [progressive-results plan](plan-2026-08-11-fdu-progressive-results.md) is
narrowed to warm persisted roll-ups, lazy warm open, prefer-cache policy, and per-value
mixed-source provenance.
Its traversal-order work has landed and remains useful background, but it no longer owns
a second streaming-session API. Epic `fdu-wpa0` tracks only that narrowed warm-serving
work; `fdu-snej` owns this plan.

## Design

### Ownership boundary

The system has three boundaries with one authority at each concern:

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
OpenedIndex API
  owns discovery, index, commit journal, observer, continuations, workers
```

The coordinator never applies filesystem deltas to a second inventory.
The adapter never walks or aggregates entries.
The core never names MetaBrowser routes, query kinds, cache keys, or SSE events.

`OpenedIndex` is the live-root API, not a façade over another API-shaped object.
Its operations are implemented directly on the public type.
Private shared state holds data and synchronization only; it has no parallel service
interface or method-for-method forwarding surface.

`OpenedIndex` is cloneable as a lightweight reference to one live state.
Cloning it does not copy the index, clock, session, journal, or continuation table.
An owned `Index` value may remain cloneable for current callers if necessary, but such a
clone is a detached value and carries no live session or continuation authority.
Persisted state uses a distinct snapshot representation rather than pretending to be a
second live owner.

`close()` closes that shared live state, not one reference.
The first caller starts cancellation and joined shutdown; concurrent callers wait for
and receive the same terminal result.
Every outstanding clone immediately observes the closing or closed lifecycle and all
operations except repeated `close()` return a typed closed-handle error.
Dropping the last reference performs the same joined shutdown as a defensive fallback,
but ordinary clients close explicitly.

### Surface shape

The initial Rust shape is intentionally synchronous:

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

The exact names may follow existing fdu vocabulary during implementation, but the
cardinality and ownership are fixed.
The associated constructor deliberately preserves the existing free `fdu_core::open` and
its blocking one-shot return type.
There is no callback per entry and no async executor in core.
Blocking waits use standard threads, locks, condition variables, and cancellation.

The Python binding mirrors the five synchronous operations.
Calls that may block or perform substantial Rust work release the GIL. It does not hide
long-lived change polls in Python’s shared default executor.
The MetaBrowser adapter owns the async bridge described below, where iterator
cancellation and handle close have distinct lifetimes.

The public Python package is a peer surface over `fdu-core`, not a MetaBrowser SDK and
not a façade over the command line.
It uses fdu-native immutable request and result values, preserves the Rust lifecycle and
error distinctions, and performs no query-specific aggregation.
That makes the same package suitable for another Python inventory client without
depending on MetaBrowser names, HTTP payloads, or asyncio policy.

The command line remains an equally complete standalone consumer of the engine.
The opened-root work does not replace its existing one-shot path, change its defaults,
or route Python through a subprocess.
If an interactive CLI is later justified, it composes the same `OpenedIndex` operations
and serializers rather than adding CLI-only engine behavior.
The existing one-shot CLI/Python parity corpus remains in the handoff gate throughout
this effort.

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
Components become Unicode scalar strings joined by `/`; Windows native separators are
structure, not path content.

**Every entry has a canonical name.** A native path is not obliged to be UTF-8 — Unix
filenames are arbitrary non-NUL bytes and Windows filenames may hold unpaired surrogates
— so exactly two kinds of byte are percent-escaped: those that do not decode, and `%`
itself. Nothing else is touched, because the result is a JSON string rather than a URL:
`café/naïve.txt` is unchanged, while `x` followed by `0xFF` becomes `x%FF`.

Escaping `%` in every name is what makes the mapping injective, and it is not optional.
A file named `caf%FF.txt` is valid UTF-8 and one named `caf` + `0xFF` + `.txt` is not;
escaping only the undecodable byte gives both the same wire name, which is the aliasing
failure of lossy conversion in better clothes.
The price is that `100%.txt` transmits as `100%25.txt` — invisible wherever the adapter
decodes, visible to anyone reading raw JSON, and the same property git’s quoted paths
have.

Because the encoding is total, ordered pages and native roll-ups answer over **one**
population. A directory whose own name holds a stray byte still lists its children under
that directory’s escaped name.
A complete directory that does not hold a name answers `absent` rather than `unknown`,
because nothing can be hiding in an unlistable set.
One completeness flag suffices, since portable and native consumers now agree about
which entries exist.

This replaced a partial derivation that returned nothing for such a component.
That version needed an omission count, bounded escaped examples, and a second
completeness flag to describe what it could not name, and it left a directory reporting
ten thousand files while paging it returned nine thousand nine hundred and ninety-eight
— both answers correct, over different populations.
Every mature system that meets this problem makes the derived name total instead: git’s
quoted paths, Python’s surrogate escapes, and the `file://` URIs that LSP and the
desktop file managers exchange.
None of them tells a caller that a file has no name.

The conformance packet includes invalid Unix bytes and Windows Unicode/separator cases,
and must pair a literal `%` with an undecodable byte in one corpus, since that pair is
what a non-injective encoding collapses.

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
- `unknown` when a budget, failure, unsupported scope boundary, or unfinished discovery
  prevents an absence claim.

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

`OpenedIndex` covers the entire opened-root lifetime.
There is no separate progressive or watch session object.

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
| Directory | Tree page at the requested render depth |
| Filtered tree | One complete-or-limit selected-tree report at the requested render depth |
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

#### Row order is stated here, not inferred from an implementation

Ordering is part of the joint contract, and every order below is **total**: two
providers answering the same query at the same version return the same rows in the same
sequence, with no tie left to insertion order, hash iteration, or a stable sort’s input
order. An order a provider merely happens to produce is not a contract.
This section states the orders because the earlier prose did not: “parent-first” does
not distinguish level order from pre-order, and the two emit different sequences for any
tree deeper than one level.

**Tree and directory pages are breadth-first level order.** All children of the
requested path are emitted, then all children of those directories, until the query’s
maximum depth is reached or the page bound is met.
Within one parent, directories precede nondirectories, and each partition is ordered by
canonical component UTF-8 bytes.

Level order rather than pre-order, for three reasons:

- it matches how the index fills.
  Discovery is parent-first in bounded batches, so a level-ordered page is served from
  the knowledge that exists earliest, while a pre-order page descends into the subtree
  least likely to be discovered yet;
- it keeps truncation honest.
  A pre-order page cut at its row bound can return one directory and its first thousand
  descendants, leaving the caller unable to tell whether the parent has two entries or
  two thousand. Level order returns the complete shallow picture first, so what a bound
  withheld is visible as depth rather than hidden as breadth;
- the reference provider already implements it, so this documents behavior rather than
  changing an oracle.

The cost is a more intricate cursor, and that cost is paid in the engine rather than by
the caller. A level’s frontier is every directory one level above it, which is unbounded
and therefore cannot be stored in a record capped at 64 KiB; re-deriving it per page
would make paging one wide level quadratic in the number of pages.
So a tree continuation stores **one frame — the parent’s native path, its depth, the
partition, and the last emitted child name** — and derives the rest.
That last name is optional, because a page can stop having just arrived at a parent none
of whose children are emitted yet; the frame then resumes at the start of the partition,
which is not the same statement as naming its first child.

One frame is enough because the ancestor chain of that path already *is* the stack this
paragraph once described, and splitting the path recovers it.
Advancing to the next directory at a depth walks that chain with one `portable_children`
lookup per level, so resuming costs work proportional to depth rather than to everything
already emitted, and the record stays bounded by a single path.

Two questions have to stay distinct in the implementation, and conflating them is what
turns level order back into pre-order: *the next parent at this depth*, and *the first
parent at the next depth*. The first walks siblings; the second descends only once the
level above is exhausted.

Stopping a page between those two questions is the one place the work bound stated under
Coherent reads is not enforced, and the exception is deliberate.
Searching for the next parent to expand is charged against the budget but never cut
short, because a search abandoned mid-level has no emitted row for the one frame to
name. A cursor that cannot record where the search had reached leaves the next page to
restart it and overrun the same budget, so a level wider than the budget would never be
crossed at all: the tree would become unpageable rather than merely slow, and returning
a typed limit instead has the same effect, since re-asking cannot make progress.
Letting the search finish costs one scan of one level, paid once at each level boundary
rather than once per page, because the parent it finds is what the frame then records.
Teaching the frame to express the searching state as well as the emitting one is tracked
as `fdu-pokc`.

What is never acceptable, budget or no budget, is stopping with rows left and no
continuation: that is the silent relabelling that rule forbids, and it is what
`opened::tests::a_tree_page_stopped_by_the_work_budget_is_resumable` pins.

**Flat and catalog pages are lexicographic by the complete canonical POSIX-relative
path, encoded as UTF-8 bytes.** A flat continuation stores the last emitted portable
path; that path alone determines the position.

**Ranked recency answers two ordering questions, and both are part of the contract.**
Rows are *selected* by ignored state, then modification time descending, then canonical
POSIX-relative path ascending.
The page that survives is *returned* in modification time descending, then that same
path ascending, so a caller sees the newest first among rows chosen for relevance.
The path is the final key in both, because it is unique within one index and that is
what makes each order total.

Ignored entries rank last during selection deliberately.
Installing dependencies writes thousands of files at once, and pure recency answers
“what have I been working on” with a page of `node_modules` and none of the caller’s own
work. An earlier draft of this section forbade exactly this, reasoning that ranking
should depend on nothing but time; the consuming application already had a test
asserting the demotion by name, and it is right.
Structural tidiness is not a reason to delete behavior a product depends on.

What was genuinely wrong is narrower and still holds: the demotion applied only when the
match count exceeded the row bound, so one query name carried two ranking contracts and
which one a caller received depended on the size of the corpus.
It applies in every branch now.
Beyond these keys nothing reorders ranked rows — not size, not type, not depth.

**`include_ignored: false` prunes the subtree, not merely the row.** An excluded
directory contributes neither itself nor any descendant.
This is what a browser means by hiding ignored content, it is cheaper than filtering
rows a caller will never see, and stating it prevents one provider pruning while another
filters.

**Ordered pages are drawn from representable entries.** Tree, flat, catalog, and recent
rows come from the maintained portable structures, so an entry whose native path has no
canonical representation is absent from all four.
It is not silently dropped: it remains in native facts and roll-ups, and its count and
examples reach the caller through the portable-path issue and per-directory completeness
already defined above.
Ordered projections and native roll-ups therefore answer over deliberately different
populations, and a conformance case pins that difference rather than letting it read as
a defect.

The tree projection pays through the retained hierarchy and two bounded child
partitions. The flat projection pays through a commit-maintained ordered index of
representable portable paths; it never materializes and sorts the full catalog per
request. The recent projection pays through the maintained timestamp-ordered set, which
is what makes a ranked slice proportional to the row bound instead of to the tree.
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

The opened root keeps one bounded journal of commits.
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
provider batch; that is a different authority and recovery boundary.

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
The opened-root state, journal, blocking reads, refresh, and continuation table use the
standard library where practical.

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
- Write the three row orders defined above into `contract.py` itself, beside the query
  and projection dataclasses they govern: breadth-first level order for tree and
  directory pages, canonical POSIX path order for flat and catalog pages, and
  `(mtime descending, canonical POSIX path ascending)` for ranked recent rows.
  The contract currently states none of them, so each provider’s order is discoverable
  only by reading its implementation, and a tie is settled by whichever sort the
  provider happened to use.
  Both providers return the stated order directly; the coordinator never resorts
  assembled pages.
- Delete the ranked-recency reordering in `_recent_projection` that moves ignored
  entries behind unignored ones when the match count exceeds the row bound.
  It applies in one branch and not the other, so `include_ignored` silently selects
  between two different ranking contracts.
  `include_ignored` filters; it does not rank.
- State that `include_ignored: false` prunes the excluded directory’s whole subtree.
  The reference already prunes, by skipping before extending its frontier; the contract
  has to say so, because filtering rows instead is an equally reasonable reading of an
  unstated rule.
- Remove the unrepresentable-path issue and the second completeness flag.
  The canonical name is total, so pages and roll-ups cover the same entries and one
  completeness value answers for both.
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
  Favor direct code over imitating fdu’s private state, journal, or continuation types.

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
  observation gap. Each authority recovers only its own boundary.
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
- Run bounded reads, refreshes, and close through MetaBrowser’s existing
  `asyncio.to_thread` policy, matching the Python provider.
  Give each opened provider handle a dedicated one-worker change poll and a one-slot
  locked mailbox to its single active async change iterator.
  The native call releases the GIL and uses a poll timeout no greater than 250
  milliseconds. The worker wakes an `asyncio.Event` with `loop.call_soon_threadsafe`, and
  it neither polls again nor advances its local cursor until the iterator consumes the
  pending result. Backpressure can therefore recover from journal eviction through the
  ordinary consumer-reset result.
- `aclose()` on the change iterator cancels and joins only that bridge within one poll
  interval. It does not close the provider handle, and a later bounded read on the same
  handle must succeed.
  Handle `close()` cancels the iterator, joins the bridge, and then joins the native
  native handle. A second simultaneous change iterator fails with a typed busy result.
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

PR #44 and PR #47 should remain accessible as evidence while the replacement slices
land. PR #44 is the measured design and research base; PR #47 is its implementation
descendant. Neither branch is a compatibility contract.
Code and evidence are selected by invariant and retested or re-evaluated in their new
context. One commit is eligible for an audited whole-commit cherry-pick; every other
reusable piece is smaller than its original commit.

### PR #44 Design and Research Base

PR #44 is reconciled but not merged.
Its formal review and later MetaBrowser handoffs were never folded into the branch, so
merging it would add a known-stale active plan beside this one.
The complete ancestry, research summary, measurement table, and superseded decisions are
recorded in the
[PR #47 review](../../reports/report-2026-08-25-pr-47-design-and-readiness-review.md#pr-44-reconciliation).

| PR #44 material | Disposition |
| --- | --- |
| Measured 120,001-entry client comparison | Evidence. Preserve the regime and single-trial caveat; rerun through Phase 3A before making adoption claims. |
| Source verification and raw-versus-canonical extension correction | Retain. The Phase 5 File Rollup packet gains direct basename derivation cases. |
| Requirement-by-requirement MetaBrowser inventory | Retain as historical input; this plan and the reviewed MetaBrowser contract own current requirements. |
| Retained-engine seam, no callbacks, coherent reads, read-on-dirty, registry-at-open, and capture-before-baseline | Retain in the current architecture. |
| Separate session surfaces, `IndexHandle` as the live API, package-owned async adaptation, generic tag planes, and `since(clock)` without session identity | Reject or replace as specified by the current architecture. |
| PR #44 `TODO.md`, tbd config, and active-plan files | Do not import. The current branch already has the later tracker configuration and one active opened-root plan. |
| PR #44 commits | Do not cherry-pick. Link the immutable artifacts for attribution and extract only the evidence above. |

PR #44 can close as superseded after this reconciliation is visible on PR #48. It should
not merge into `main` or into the rewrite branch.

| PR #47 area | Disposition |
| --- | --- |
| Runtime registry parser, logical/canonical extension derivation, browsing groups | Extract, then change open to consume actual registry content and derive identity. |
| Typed lifecycle state and coherent read envelope | Extract after simplifying the live state and state clock. |
| Bounded journal and no-history reset | Extract behind the exact commit pipeline. |
| Shared guarded reads and GIL release | Extract and retain. |
| Scripted watcher, hidden/special-object tests, scope fixtures | Reuse as tests against the new producer boundary. |
| Bounded refresh, prioritization, joined close | Reimplement around the one `OpenedIndex`; reuse value types and focused tests where sound. |
| Requested-observation `AppliedDelta` and implicit-parent mutation | Replace. |
| Mutable clone/session/continuation authority | Replace. |
| Stateless signed page token | Replace with bounded handle-local continuation state. |
| Exact global `max_files` prefix and live free-slot semantics | Replace with honest discovery-budget behavior. |
| MetaBrowser query names in fdu journal or adapter-facing core values | Replace with stable fdu impact domains and native projections. |
| Generic tags and promoted planes | Defer; retain only the demonstrated unignored partition. |
| Per-value progressive provenance | Defer pending a separate trust design. |
| Sorted resumable pages, lazy snapshot blocks, expanded diagnostics | Defer until measured or required. |
| CLI progress and new goldens | Defer until the opened-root engine and client adapter are proven. |

### Reuse protocol

Use three distinct operations, and name which one a rewrite commit used in its commit
message:

1. **Audited cherry-pick.** Run `git cherry-pick --no-commit <sha>`, read the complete
   staged diff against the new base, remove any out-of-scope hunks, and run the gate for
   that slice before committing.
   This is allowed only when the old change already owns one coherent invariant and does
   not import the prototype’s live-state object, journal, token, cap, or generic-plane
   design.
2. **Selective extraction.** Reapply named functions, value types, fixtures, or tests in
   their new context. Preserve the source commit in the new commit message, but do not
   use `git cherry-pick` when the old commit also carries rejected semantics.
   Tests are rewritten to assert the new contract before production code is ported.
3. **Evidence only.** Keep the old commit as a regression record.
   Do not move its code.
   Mine its tests, review notes, and failure mode when they help prove the replacement.

Before the first `index.rs` mutation-pipeline edit, land the registry commit identified
below. It applies cleanly to the current branch today, changes no default answer, and
overlaps the exact files that Checkpoint 1B will substantially rewrite.
Landing it after 1B would manufacture avoidable conflicts.
It is a preparatory commit credited to Checkpoint 1C, not permission to expose an
incomplete interactive surface.

### Whole-commit decision

| Commit | Decision | Exact procedure |
| --- | --- | --- |
| `9b31220` `classify: the rule set becomes a value a caller can supply` | Audited cherry-pick. This is the one coherent, semantically retained implementation unit. | After Checkpoint 1A, apply with `git cherry-pick --no-commit 9b31220`; inspect all 11 files; retain `TypeRegistry`, the shared manifest parser, registry-owned classification, scope/content fingerprinting, and its migration tests; run default/no-default, snapshot, content, and classification tests before committing. Do not add the later PR #47 surface flags in the same commit. |
| `29cd8bf` `test: assert what a run costs` | Do not cherry-pick despite a clean mechanical apply. | Retain `CounterSnapshot::to_json` only if the 3A harness needs it. Replace the inline parsed `cli-cost` golden with focused Rust relations under `fdu-9tdm`; never import the selective-output anti-pattern. |
| `a4b0626` `serve reads during a write, and expose scope knobs` | Do not cherry-pick despite a clean mechanical apply. | Extract the `IndexHandle::with_index` shared-read pattern, GIL-detached read tests, and owned child snapshots only. The CLI knobs and mutable shared handle do not belong to the new opened-root contract. |
| `50e078c` `name the roll-ups each batch invalidated` | Do not cherry-pick despite a clean mechanical apply. | Reuse the independent ancestor-oracle test. Its implementation derives dirtiness from requested operations instead of the exact commit and would preserve the central defect. |

These four are the only implementation commits whose raw patches currently pass
`git apply --check` against this branch.
Mechanical applicability is recorded so an implementer does not have to rediscover it;
it is not a correctness signal.
The `9b31220` decision was also exercised in an isolated worktree at this plan’s base:
`git cherry-pick --no-commit 9b31220` completed without conflicts, then
`cargo test --locked -p fdu-core --no-default-features` and
`cargo test --locked -p fdu-core --all-features` both passed.
That establishes a reproducible starting point, not permission to skip the post-pick
diff review or the checkpoint gate.

### PR #47 implementation ledger

The ledger covers every non-documentation commit unique to the reviewed PR #47 head.
“Extract” means selective extraction under the protocol above; “evidence” means no
production code moves.

| PR #47 commit | Decision | Reusable unit or reason to leave it |
| --- | --- | --- |
| `a4b0626` | Extract | `IndexHandle::with_index`, owned child snapshots, concurrency and GIL tests. Leave CLI scope flags and live mutable-handle authority. |
| `3df2e1a` | Extract | Python refresh conversion and subtree reconciliation tests. Replace the one-path API with bounded `refresh(paths)`. |
| `50e078c` | Tests only | Independent ancestor-impact oracle. Replace requested-op reconstruction with commit-derived impact. |
| `d050dcc` | Defer | Typed performance conversion may serve 3A. Do not widen the first public surface for optional telemetry. |
| `b3fb4b1` | Defer | Tree remainder presentation belongs after the joint contract removes exact remainder control flow. |
| `cbb4e88` | Extract | Capped aggregate value shape and conservation tests. Adapt them to `exact(n)` or `at_least(n)`. |
| `9b31220` | Cherry-pick | Runtime `TypeRegistry`, one manifest parser, registry-owned classification, and derived fingerprints. |
| `ccb7881` | Extract later | Python registry model and conversion. Do not add CLI flags until the adapter proves a CLI use. |
| `77e5b7b` | Extract | Browsing-group values and reducer tests, after the registry lands. Maintain the fixed group index only if 3A demonstrates it. |
| `b6a0391` | Extract | Listing-row kind conversion and tests. |
| `cc91bef` | Defer | Polling backend remains an explicit later-platform question. |
| `0120e23` | Evidence | Cancellation and shutdown cases. Reject the package-owned async executor and implicit bridge policy. |
| `9460231` | Extract | `watch/scripted_events.rs`, observer injection seam, overflow script, and deterministic tests. |
| `29cd8bf` | Extract narrowly | Counter JSON serialization if needed; move cost relations to Rust tests and do not import `cli-cost` as written. |
| `2ab02ee` | Extract | One-guard coherent-read pattern and version-consistency tests. Rewrite the projection vocabulary. |
| `2a70a12` | Extract | `logical_ext`, `TypeRegistry::canonical_ext`, platform-specific name tests, and roll-up invariants. |
| `a5a7ae3` | Extract | Allocation-free child traversal and page-conservation tests. Replace offset/token paging with handle-local traversal state. |
| `b8b0aef` | Extract | `Work` accounting and proportional-read tests. Rename counters only where the approved contract requires it. |
| `11b6dc6` | Extract | Non-file leaf count and empty-directory regression tests. |
| `5ace86c` | Extract fixes | Port each Windows-only correction with its test when the affected code moves; do not cherry-pick the accumulated diff. |
| `3ddcda1` | Tests only | Contract-alignment cases, updated to the new vocabulary. |
| `a07fa17` | Extract | Coverage-reason vocabulary and exhaustive state tests, aligned with this plan’s state table. |
| `e658915` | Evidence | Reject generic tag algebra. Reuse bit-move and reclassification mechanics only if they simplify the fixed `all`/`unignored` partition. |
| `e47a535` | Extract | Query fixtures and report mappings. Keep application query names out of core. |
| `5012069` | Extract | Symlink-only directory emptiness correction and complete-output regressions. |
| `f4c60ed` | Extract | `.gitignore` parsing, control-source, negation, edit, and deletion tests. Reject generic planes, the old open-time tree walk, and its dependency/MSRV decision. |
| `6a6291f` | Extract | The rule and tests that a bound cannot hide a reported dimension. |
| `9adc8c7` | Extract | Scope/semantic fingerprint snapshot tests. Rewrite snapshot construction around detached `IndexImage`. |
| `be04134` | Extract | Control sources come from retained control state, not a second tree walk. Rewrite reclassification as an exact prepared commit. |
| `37e791f` | Evidence | Confirms one opened-root authority. Its lifecycle remains entangled with the old session and is replaced. |
| `4ddf0e9` | Tests only | Foreign, stale, and resumed-page failure cases. Reject stateless signed tokens. |
| `a3960fb` | Extract | Coherent envelope assembly and read-lock boundary tests. |
| `558461a` | Tests only | Consumer-information fixtures. Replace its requested-versus-effective batch construction. |
| `b18393d` | Extract | Unresumable-page and maximum-work validation tests. |
| `5f31ba5` | Extract | Report work charging and proportionality tests. |
| `56dcf56`, `715f748`, `278457a` | Extract final form | Final GIL-detached read wrapper plus decisive overlap and whole-boundary measurements. Port only after `PyOpenedIndex` exists. |
| `b4123e4` | Tests only | Preserve both regressions it repaired as pre-port tests. |
| `ac38584`, `4fbb7d1` | Tests only | State-transition and commit-clock vectors. Replace the old split state mutation with one prepared commit. |
| `fad3d2f` | Extract | Journal range, resume, and cursor tests. Rehouse them under `opened/journal.rs`. |
| `c31ad3c` | Extract | Pinned assembly must keep one version and clock; adapt to one `EngineVersion`. |
| `112981c` | Extract | Only the approved exhaustive state enums and vocabulary checks. |
| `1e1c207` | Extract | Condition-variable wake, timeout, and bounded payload tests. Replace generic retag transport. |
| `44e79c3` | Extract | Strong consumer-history reset semantics and tests. |
| `1e6b648` | Extract | fdu-native impact domains and dirtiness tests, derived inside the exact commit. |
| `6b9f080` | Extract | Change-batch work counters and tests. |
| `a6a89ab`, `7aaaf84` | Tests and mechanics | Reject generic promoted planes. Reuse merge/unmerge conservation cases for the fixed two-partition reducer. |
| `c4f3343` | Extract test fix | Deterministically select the batch carrying the target commit. |
| `6a8ac6f` | Extract | Hidden-admission fixtures, `admission.rs` seam, and pruned-control exception. Keep hidden admission fixed and explicit. |
| `4eac1b2` | Spike only | Provider fixture and adapter skeleton for 3A. Delete its naive aggregation and private identity recipe after measurement. |
| `b8ead94` | Tests only | Classification-flag corpus. Reject folding flags into a general tag system. |
| `ff210d0` | Extract | macOS admission parity cases, bulk-path fix, and admission-site check. Derive identity in the engine. |
| `eaae030` | Extract | Terminal-state-on-every-batch and journal-range tests, aligned to the approved enum. |
| `349de4b`, `1b76062`, `ee5728b` | Evidence | Reuse the resource-stop fixture only. Reject deterministic-prefix, strict semantic-cap, live-free-slot, and cap-as-scope semantics. |
| `d19b0ce` | Extract | Path normalization, validation, deduplication, ancestor collapse, widening, and receipt tests. Route the result through exact commits. |
| `91b6895` | Extract | Predicate evaluation and page-conservation tests. Rewrite ordering and continuation authority. |
| `515d52c` | Extract | Invalidations-only consumer interest and bounded-work tests. Keep the journal exact and interest-free. |
| `048b0cc` | Extract | Admitted-kind detection, special-object fixtures, and platform tests. V1 exposes one fixed MetaBrowser-compatible configuration. |
| `d0a6a6a` | Tests only | Special-object provider-boundary examples and agreement cases; the new conformance packet owns them. |
| `e380113`, `ce8d78b` | Tests only | Provider receives engine-derived identity. Do not preserve the prototype’s duplicate digest recipe. |
| `581c369` | Evidence | Reject depth as live scope and watching restricted scope. Maximum depth is selection in the new contract. |
| `051e7cc` | Extract | Continuation work proportionality and no-rescan traversal position. Replace token payload/signing with the handle table. |
| `1e9b85d` | Spike only | Real catalog-page example and the recorded nonpaged work become 3A measurement inputs, not shipping adapter code. |
| `825fd92` | Extract test fix | Deterministic wait target and standard fixture writer. |
| `353d48f` | Extract | One typed refusal event and live-scope validation tests, remapped to the fixed V1 config. |
| `b5035e4`, `5eb2574`, `e9af881` | Tests only | Typed stale, absent, foreign, and refusal cases. Reject token encoding, signing, and decoder code. |
| `57e04fd` | Extract | `CatalogPredicate` parsing/evaluation and the generated JSON corpus. Keep predicates fdu-native and adapter mappings exhaustive. |
| `d58d9c5` | Extract tests | Control-file signals and refusal-mutation regressions. Replace the underlying generic rule rebinding and inaccurate delta path. |

All PR #47 documentation and merge commits are evidence only.
The review report and this plan supersede their implementation choices; links remain in
References so rationale is still recoverable.

## Implementation Plan

All fdu phases use `codex/opened-root-inventory-rewrite` and one draft PR rooted at
current `main`, not PR #47. Each checkbox is independently reviewable, each phase ends
in a named green commit checkpoint, and work does not advance across a failed phase
gate. The PR remains a draft until Phase 5 passes.

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

Checkpoint 3A runs on the dedicated MetaBrowser branch
`codex/fdu-opened-root-e2e-spike`, created from unchanged PR #74 head
`3183888808b366b5ba1c381dec1cbb18b49d969e`. It measures the current contract with an
exact-revision fdu wheel and contains no shipping contract decision.
After its evidence is published and its naive adapter is deleted, retained harness work
and Phases 3B through 5 land on the MetaBrowser PR #74 branch.
Both PR descriptions pin the exact counterpart revision used by every cross-repository
checkpoint.

### Phase 1: Exact Engine Kernel

Phase 1 has four separately reviewed green checkpoints.
No checkpoint changes default CLI behavior or exposes an incomplete MetaBrowser
provider. Code is reimplemented from `main` or extracted in minimal pieces from PR #47;
the sole audited whole-commit exception is `9b31220`, landed immediately after 1A as
described in the reuse protocol so later `index.rs` work does not create conflicts.

#### Checkpoint 1A: Observable Oracle

- [x] Fix `fdu-9tdm`: replace surgical golden parsing with broad observable-output
  assertions before importing new surface behavior.
- [x] Add an independent deterministic reference model for retained facts, parent
  ordering, roll-ups, exact changes, control-file updates, and resource refusal.
- [x] Fix the closed session schema, automatic contract-coverage manifest, and staged
  per-owner test-control boundaries described in Testing Strategy.
  Implement each seam with the production Phase 2 capability it controls; compose the
  five session goldens only after all five operations exist.
- [x] Gate with the focused model tests, complete golden corpus, CLI/Python parity,
  `make docs-format-check`, and `make check`.

Implementation status: complete.
The transparent-output policy and independent reference model landed under `fdu-utf1`;
the session schema and seam ownership are fixed here, while `fdu-0kv7` still owns the
later composition of the five real opened-root session artifacts.

#### Checkpoint 1B: Exact Commit Truth

- [x] Introduce prepared mutations and one atomic `Commit` containing exact effective
  changes, impact, state, and work.
- [x] Route scan, reconcile, explicit refresh, control-file updates, and existing watch
  application through that commit path.
- [x] Remove implicit guessed-parent mutations from the live path; normalize unknown
  ancestry through verified reconciliation.
- [x] Gate with generated operation-sequence comparison, fault injection, concurrent
  reader/writer tests, `make check`, and `make cross-lint`.

#### Checkpoint 1C: Control State

- [x] Add the exact removal-aware `.gitignore` control table and the fixed
  `all`/`unignored` partition behind a removable feature.
- [x] Introduce the runtime registry/classification pieces needed by the fixed partition
  behind explicit features; preserve the no-default-features build.
- [x] Gate creation, edit, deletion, hidden-control discovery, provider-order
  equivalence, all feature combinations, and dependency audit.

#### Checkpoint 1D: Live Identity and Feature Floor

- [x] Separate detached index snapshots from live session and continuation authority.
- [x] Keep core default features empty and record dependency and binary-size baselines.
- [x] Prove the kernel under operation-sequence tests, fault injection, model
  comparison, and all existing one-shot surface parity tests.
- [x] Gate clone/detached-image identity, continuation authority, snapshot round-trip,
  `make check`, `make cross-lint`, dependency audit, and size baselines.

The Phase 1D size baseline was recorded on Apple Silicon macOS with Rust 1.97.1 using
the release profile, LTO, and stripping declared by this repository.
It is a regression reference, not a cross-platform size claim.

| Artifact or dependency surface | Baseline |
| --- | ---: |
| `fdu-core --no-default-features`, unique normal package nodes | 15 |
| `fdu-core --all-features`, unique normal package nodes | 21 |
| `fdu --all-features`, unique normal package nodes | 39 |
| Stripped release CLI | 2,688,608 bytes |
| Gzip-9 release CLI | 1,198,715 bytes |
| CPython 3.12 stable-ABI macOS arm64 wheel | 1,179,071 bytes |
| Extracted wheel payload | 2,392 KiB |

The implementation changed no lockfile entry and added no dependency.
The dependency counts come from unique normal-package lines in `cargo tree`; the wheel
was built with the locked `maturin build --release` path used by the repository gate.

Acceptance for Phase 1:

- every retained mutation and state transition is represented exactly once in its
  commit;
- applying the same normalized operation sequence to the reference model and engine
  produces identical facts, roll-ups, state, and effective changes;
- no public clone can mutate a divergent index while sharing live identity;
- current CLI and Python one-shot answers remain unchanged unless a reviewed golden
  records an intentional correction;
- no capability needed by the CLI is implemented only in CLI parsing or presentation,
  and Python never shells out to the CLI;
- one complete parity scenario proves that equivalent core, CLI, and Python one-shot
  requests produce the same normalized answer;
- `make check`, `make cross-lint`, and dependency audit pass.

### Phase 2: Opened-Root Vertical Slice

- [x] Add the single-authority `OpenedIndex` with idempotent joined close.
- [x] Implement cold progressive discovery with parent-first bounded commits,
  per-directory completeness, explicit budget state, and scheduling priority.
- [x] Add coherent lookup, depth-one tree, roll-up, state, version, and work projections
  in one bounded `read()`.
- [x] Add the bounded pull journal and `changes(after, timeout)` with state-only
  commits, cursor validation, timeout, reset, and close wakeup.
- [x] Add bounded verified multi-path refresh through the shared commit pipeline.
- [x] Add native observation with capture-before-baseline buffering, scripted overflow,
  final reconciliation, and a no-gap transition to watching.
- [x] Compose the staged test seams into the five opened-root session goldens and close
  the automatic public-contract coverage manifest against real production values.
- [x] Mirror the five synchronous operations in Python with GIL release; keep the
  long-lived async change bridge in the MetaBrowser adapter.
  Expose fdu-native immutable values and typed errors; do not leak MetaBrowser query or
  transport vocabulary into the package.
- [x] Prove native shutdown, concurrent reads and commits, slow consumers, provider
  gaps, resource-stop behavior, and every supported feature combination.
- [x] Prove Python GIL release, typed conversion, clone-wide close, installed-wheel
  isolation, and no surviving worker after shutdown.

Phase 2 validation evidence:

- `make check` passes the complete Rust feature matrix, MSRV, docs, audits, the 125-case
  CLI golden corpus, the installed wheel and source distribution, strict downstream
  Python typing, and CLI/Python parity with only the 21 classified existing deviations.
- `make cross-lint` passes both installed macOS and Windows targets.
- The installed-wheel lifecycle covers coherent mixed projections, single-use paging,
  exact change polling, verified refresh, foreign version and cursor errors, concurrent
  close through shared Python aliases, and typed post-close behavior.
- A Rust embedding test blocks in `changes()` while another Python thread acquires the
  GIL and publishes the waking refresh; no timing sleep stands in for synchronization.
- The wheel smoke invokes the packaged `fdu` entry point for version, help, guide,
  successful JSON scanning, and usage errors.
  The new surface adds no dependency or lockfile change and does not alter the command
  line’s defaults or output.

The Python cost was isolated by building the clean pre-binding checkpoint `00cce1a` and
the clean binding checkpoint `fa85812` with the same Apple Silicon release toolchain.
The CLI’s raw size is byte-for-byte unchanged; the six-byte compressed variation is
revision and layout compression noise rather than an added command-line code path.

| Artifact | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Stripped release CLI | 2,771,296 bytes | 2,771,296 bytes | 0 |
| Gzip-9 release CLI | 1,230,254 bytes | 1,230,260 bytes | +6 bytes |
| CPython 3.12 stable-ABI macOS arm64 wheel | 1,209,989 bytes | 1,349,352 bytes | +139,363 bytes (11.5%) |
| Extracted wheel payload | 2,456 KiB | 2,740 KiB | +284 KiB (11.6%) |

The wheel delta is the complete typed opened-root surface and native conversion
boundary. No package node, feature default, or standalone binary capability was added to
pay for it.

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

The spike runs on `codex/fdu-opened-root-e2e-spike` at the exact unchanged MetaBrowser
base named above. It installs an fdu wheel built from the exact PR #48 revision under
test; a sibling checkout or source-tree import is a failed experiment.
The disposable adapter may do inefficient work only when the harness counts that work.
It may not fake missing semantics, bypass the five-operation handle, or change the
provider contract to make the spike pass.

- [x] Implement a disposable fdu adapter over the Phase 2 handle against MetaBrowser PR
  #74’s unchanged provider contract.
  Materializing catalog rows, sorting, and scanning for totals are allowed only in this
  spike and are instrumented explicitly.
- [x] Run the existing route and provider tests on the representative corpus; record
  route predicates, rows visited, sort/materialization work, latency, memory, and which
  totals and orders are product-visible.
- [x] Run one installed-wheel application lifecycle through MetaBrowser: cold open,
  useful progressive read, completion, live mutation, change delivery, coherent reread,
  refresh, root replacement, iterator cancellation, and joined close.
- [x] Publish the evidence before changing either contract.
  Keep the reusable harness and evidence.
  Quarantine the naive adapter on the spike branch until the thin provider replaces it;
  it is not packaged, registered, or selectable, and the final adapter gate requires its
  removal.

The recorded spike passes nine of twelve unchanged provider-contract cases and twelve of
thirteen selected route and SSE cases.
The complete lifecycle passes cold and progressive reads, settled state, live mutation
and coherent reread, explicit refresh, root replacement, iterator-only cancellation,
concurrent close, and zero surviving poll workers.
The four differences are now contract work rather than adapter guesses:

- resource refusal enters fdu’s terminal `stopped` phase instead of pretending the
  refused root is watching;
- journal capacity must be defined in provider-batch terms because one application
  refresh may create several native commits;
- the pending-discovery route test needs a provider-owned scripted barrier rather than a
  private Python-walker pause;
- recursive invalidations need one explicit host coalescing boundary.

On the fully provisioned MetaBrowser checkout, the materializing eight-query read
visited 8,830 native rows and 412,836 path bytes, rebuilt 470 child buckets, ran four
full-result sorts and four aggregate passes, returned 9,071 rows, took about 852 ms, and
reached about 13 MB of traced Python allocation.
A source-only checkout measured 791 rows and about 98 ms.
These are recorded observations, not adoption claims.

#### Approved native index set

Checkpoint 3A justifies two structures that Phase 2 already implemented and two
additions. They live together in one optional, commit-maintained `ServingIndexes`
allocation enabled by `OpenedIndex`; detached indexes, one-shot Python calls, and the
standalone CLI neither populate nor retain them.
This is the complete Phase 3C index set.
A later structure requires new measurement and a plan amendment rather than being added
because a projection happens to expose the same noun.

| Structure | Measured work removed | Update and memory cost | Rejected alternative |
| --- | --- | --- | --- |
| Existing `portable_entries` canonical-path order | Removes the 8,830-row and 412,836-path-byte materialization and every catalog full-result sort. Flat continuations resume at the first unvisited key. | In an opened root only, one portable path string and one tree-map entry per retained non-root entry; insert and remove are part of the exact commit. | Do not rebuild and sort a Python list, retain a second catalog image, or add a separate catalog-order index containing the same keys. |
| Existing `portable_children` directory/nondirectory partitions | Removes reconstruction of 470 child buckets and their repeated sorts. Tree continuations resume within the exact parent and partition. | In an opened root only, one child-name map entry per retained child; insert and remove are part of the exact commit. | Do not reconstruct parent buckets in the adapter or retain a second hierarchy. |
| One maintained semantic classification tally per directory partition | Removes the navigation-wide classification pass and the file-type aggregate passes in roll-up. Each file contributes one registry type identity; canonical extension, family, group, and preset rows derive from bounded type and raw-extension tally maps rather than separate per-dimension indexes. Exact basenames used by declared presets retain one bounded classifier-key tally because an extension tally cannot distinguish `Makefile` from another extensionless file. | One interned type identity per file and one type-tally row per distinct type under each ancestor, in both existing `all` and `unignored` partitions; an exact-basename key exists only for names declared by the active registry. The independent reference model must prove add, update, kind change, ignore reclassification, removal, and subtree removal. | Do not add separate canonical-extension, family, group, preset, or navigation caches. They multiply ancestor-update and memory cost while carrying values derivable from the type tally and immutable registry. |
| One global portable-file recency order keyed by modification time, canonical path, and entry identity | Removes the recent-query full-result sort and supplies bounded cumulative recency counts for navigation windows. | One ordered key per regular file; metadata update, kind change, and removal replace it in the same exact commit. Ignored state stays on the entry and is tested while traversing, avoiding two time indexes. | Do not retain a sorted Python image, one index per recency window, or per-directory time indexes before a measured subtree-recency client exists. |

Three proposed structures are explicitly rejected:

- **No second catalog index.** `portable_entries` already is catalog order.
  Catalog predicates scan that order under `max_work`, stop counting at `count_cap`, and
  never sort or materialize the unreturned suffix.
- **No arbitrary-filter index.** The conjunction space for filename, type, time, size,
  ignored state, and subtree scope is combinatorial.
  Filtered tree performs one bounded native selection and aggregation pass against one
  committed version. It either returns a complete result within its row and work limits
  or a typed limit; it does not retain a historic result image merely to make an
  application route pageable.
- **No navigation cache.** Navigation derives bounded rows from maintained root
  partitions, semantic tallies, and recency order.
  The adapter formats those aggregate rows but never loops over entry rows.

The retained manual commit-cost probe applies the same 10,101-entry, two-level batch to
fresh detached and opened indexes, alternates their order over seven release-build
samples, and reports medians without enforcing a machine-dependent timing threshold.
On the local uncontrolled checkpoint host, the detached median was 51.2 ms and the
opened median was 62.8 ms, a 1.225 ratio.
The opened shape held 10,100 portable-entry rows, 10,100 child rows, 10,000 recency
rows, 404 semantic partition rows, and 202 declared exact-name partition rows from a
fixed 11-name vocabulary.
The incremental cost is accepted for the opt-in interactive path because it removes
repeated full-result materialization, sorting, and aggregate passes; detached indexes
exit before serving classification or basename allocation.
Final Phase 5 evidence still measures whole CLI and opened-root startup, peak memory,
and client query latency on the shared corpus.

The existing Phase 2 continuation implementation is retained rather than replaced.
It already keeps opaque authority in the opened handle, pins each record to one engine
version and parsed fdu-native query, resumes tree and flat reads from structural keys,
uses a 128-record oldest-first table, treats IDs as single-use, restores an underfunded
record, rejects stale and foreign or evicted IDs, and clears the table during joined
close.
Checkpoint 3C adds one missing memory bound: each record may retain at most 64 KiB
of structural payload, for at most eight MiB across the table before fixed map nodes and
allocator bookkeeping.
The insert checks this bound before advancing the identity or evicting a valid record,
and any projection error restores the consumed record.
The continuation request carries no second query, so query mismatch is deliberately
escaped into its canonical name rather than raising another public failure mode.
Tests prove retained filtered-query identity, constant structural resume work, single
use, version pinning, foreign and evicted outcomes, underfunded retry, bound atomicity,
and clear-on-close behavior.

The filtered-tree paging field in MetaBrowser commit `45266a8` is therefore provisional
and must be removed before the native provider lands.
MetaBrowser’s route accumulates every provider page before returning one bounded
response, so paging did not bound the route’s final memory or output.
A complete-or-limit provider projection is simpler and honest: it avoids a retained
selected-row image, a root rescan on every page, and an unbounded breadth-first
continuation frontier.

The standalone binary is an independent acceptance denominator for this index work.
The serving structures live in `fdu-core` and are available through `OpenedIndex`, but
the existing command parser, detached `Index`, and one-shot renderer do not allocate or
maintain them. The Phase 3C gate repeats the complete CLI golden corpus and records
stripped bytes, gzip bytes, cold startup, one-shot scan time and peak memory,
`cargo tree`, default-feature state, and `--no-default-features`; a regression is a
design failure, not an adapter tradeoff.

#### Checkpoint 3B: Joint Contract and Reference Provider

- [x] Pass immutable registry content, derive provider identity from parsed content,
  make discovery budget execution policy, move maximum depth to query selection, and
  name the supported v1 filesystem scope explicitly.
- [x] Update the Python reference provider so the injected registry drives filters,
  navigation tallies, and rollups, and resource refusal remains terminal, readable, and
  non-expanding.
- [x] Align lifecycle and issue values, make row orders exact, add honest count and work
  bounds, and remove exact page remainders from the provider contract.
- [x] Update the Python reference provider to implement the revised lifecycle,
  work-limit, count, and page contracts.
- [x] Update coordinator and route assembly to use bounded continuation safety and
  coherent aggregate totals without reintroducing filesystem or aggregation ownership.
- [x] Gate the revised Python provider and all existing routes before adding fdu.

#### Checkpoint 3C: Native Indexes and Thin Adapter

- [x] Add path-ordered tree and flat-entry continuations backed by the bounded
  handle-local table; cap retained record payload, preserve tokens after underfunded or
  failed resumes, and prove query ownership, version and handle isolation, eviction,
  proportional work, and close cleanup.
- [x] Isolate all interactive indexes behind an opened-root-only `ServingIndexes`
  allocation; maintain semantic type tallies, declared exact-name tallies, and global
  file recency through exact insertion, metadata update, kind change, ignore
  reclassification, and subtree removal; prove detached indexes and snapshots retain
  none of that state, and retain a release-build commit-cost and structural-row probe.
- [x] Parse the actual File Rollup v3 registry once at opened-root setup, derive its
  identity from validated content, and expose registry-owned classification and browsing
  taxonomy without a TOML, Python, or MetaBrowser dependency in the standalone binary.
  Compose the established one-shot `Selection` inside a new additive `EntrySelection`
  carrying the name, ignored-state, maximum-size, suffix, and ancestor predicates
  required by bounded native reads.
  Do not add fields to the existing public struct or create adapter-only filtering
  semantics.
- [x] State the three row orders in the joint contract before implementing against them.
  Ordered pages were specified only as prose that did not distinguish level order from
  pre-order, ranked recency left its tie to a stable sort’s input order, and
  `contract.py` documented no row order at all.
  A guess is invisible until cross-provider replay, which is the failure mode this
  rewrite exists to remove.
- [ ] Complete the existing fdu query indexes needed for filtered tree, navigation,
  recent, catalog, and diagnostics without per-request Python aggregation, traversing
  the stated orders with the depth-bounded cursor stack rather than materializing a
  frontier.
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
- the stripped standalone fdu CLI remains byte-for-byte at its Phase 2 baseline, its
  existing one-shot goldens remain unchanged, and neither MetaBrowser nor the Python
  package shells out to it.
- the command parser, defaults, output shapes, exit behavior, and one-shot cache
  behavior remain unchanged.
  No opened-root serving allocation is reachable from an existing CLI invocation, and
  the CLI dependency tree contains no Python, async-runtime, serialization-framework, or
  MetaBrowser package.

### Phase 4: Control-State Scale and Bound Discipline

Phase 1 built exact control state and proved it on repositories.
It does not survive a real home directory, and that is a merge blocker rather than a
tuning question.

Field agents running ordinary roll-ups on macOS could not complete a scan of `~`,
`~/wrk`, or `~/Library` with a branch build.
Two aborted with `control table requires N bytes; limit is 4194304 bytes`; the third was
SIGKILLed. Current `main`, which has no control table, scans the same `~/wrk` in 15
seconds and exits 0. The regression is this branch’s, and it arrives on `main` when this
PR merges.

The overshoot is not marginal, and the reported numbers say otherwise only by accident.
The budget trips at the first crossing of a running cumulative total, so the number in
the message is where the walk crossed, not what the tree needed.
Applying this branch’s own `retained_source_cost` to all 3,256 `.gitignore` files under
`~/wrk` charges 9.93 MiB against a 4 MiB cap — 2.4 times over, not the 0.4 % the message
implies. A cap raised to 5 or 8 MiB would look sufficient and would not be.

Two directions are already closed, and are recorded here so they are not reopened:

- The cap cannot be checked before the walk.
  The total is a property of the tree and is unknowable until the tree has been walked,
  so “fail fast instead of after three minutes” is not implementable as stated.
  Degrading at the crossing is what turns the elapsed work into an answer.
- The `~/Library` slowness first attributed to fdu was a cold-cache artefact, and the
  regime was not controlled when it was recorded.
  Warm, fdu finishes `~/Library` in 35 s at 359 MiB and `dust` in 30 s at 218 MiB. Cold,
  `du` and fdu both exceed a minute on `~/Library/Containers` alone, whose 1,012 sandbox
  containers each cost a TCC permission check.
  Cache state, not the tool, explains the original observation, and this plan states it
  because the project’s own rule is to record the regime rather than the number.
  What survives the correction is memory, not time: fdu’s peak is consistently 1.5 to
  1.6 times dust’s on the same trees, which is what makes fdu the first process to die
  on a host already under pressure.

#### What dust already solves, and what it does not

Two recorded comparisons through `make perf-compare-tools`, paired and interleaved, on
`rustup-toolchains`: 119,368 entries, no symlinks, no mutation during the run, zero
invalid samples, zero semantic or oracle mismatches.

The subject had to be chosen, not assumed.
`~/wrk` cannot carry this comparison at all: dust’s claim-grade contract excludes
symlink-bearing trees, and that live working directory grew by 47,881 entries mid-run.
The harness refused every sample of the first attempt, correctly.

| fdu contract | fdu wall | dust wall | fdu peak RSS | dust peak RSS |
| --- | ---: | ---: | ---: | ---: |
| `fdu-transient-summary` | 0.146 s | 0.218 s | 15.0 MiB | 59.1 MiB |
| `fdu-default-tree` | 0.157 s | 0.197 s | 77.9 MiB | 59.2 MiB |

Read the second row.
`fdu-transient-summary` passes `--cache off` with an explicit view, which isolates
engine work; `fdu-default-tree` is what a user gets by typing `fdu PATH`, and typing
`fdu PATH` is the only thing the field reports did.

On the default path the wall-time lead is not real: dust is +20.6 % with a 95 % interval
of −12.8 % to +27.2 %, which crosses zero.
What is real is memory.
fdu peaks at 77.9 MiB against dust’s 59.2 MiB, and the harness classifies fdu
**inferior** on this subject because `peak_rss_bytes` exceeds its +5 % resource limit
and `minor_faults` exceeds its +10 % limit.

The gap is fdu’s own rather than dust’s advantage, and it is the retained index that the
default *view* forces — not the cache, and not the snapshot write.
Holding `--cache off` fixed and changing only the view, peak RSS on the same subject
moves from 14 MiB under `--view summary` to 66 MiB under the default tree.
Cache policy adds roughly 9 MiB on top of that, about an eighth of the gap.

`plan_report` in `crates/fdu-core/src/execution.rs` states the rule: `--cache off` does
not avoid the index, it only permits the summary tier, and `RetainedState::FullIndex` is
selected by any view other than a bare unfiltered summary.
The depth-2 default tree therefore retains one node per entry, about 550 bytes per
retained entry on this subject.

Reducing it is a question about the default view’s retention rather than about cache
policy: either the tree is served from a bounded structure instead of a full index, or
per-entry retained size comes down.
Control state built for every scan sits inside that per-entry cost and is the cheapest
part to remove first (`fdu-etfj`); `fdu-syyl` carries the attribution.

Three differences remain qualitative, and only one is a technique fdu can copy.

- dust has no per-directory control state to exhaust.
  `-I`/`--ignore-all-in-file` takes one file of regexes; there is no hierarchical
  `.gitignore`, no retained table, and no snapshot to serialize one into.
  It escapes this phase’s headline defect by not offering the feature rather than by
  bounding it better, which fdu cannot adopt while keeping exact control state.
  It does establish the weaker claim this phase rests on: a size roll-up needs none of
  it.
- dust suppresses filesystem errors by default and offers `--print-errors` to opt in,
  exiting 0. fdu does the inverse and exits 2. dust’s quiet is not free — its `~` total
  is 206 G against fdu’s 214 GiB, and part of that gap is what it skipped without saying
  so — so `fdu-5ffm` should take the opt-in shape without taking the silence.
- dust accepts a regular file as a root and reports its size; fdu rejects one
  (`fdu-tsdy`).

Neither recorded run is confirmable under the release rule yet: the transient run lacks
a paired interval for `voluntary_context_switches`, and the default run is decided
against fdu on resources rather than being inconclusive.
Both are clean measurements; neither is a published claim.

The phase’s ordering rule: the roll-up must stop paying for state it does not consume
before anything tunes what that state costs.

- [ ] Gate control observation on a runtime capability rather than the `gitignore`
  compile feature alone, so a default roll-up performs no control-file I/O and retains
  no control state (`fdu-etfj`).
- [ ] Replace the abort with degradation: on crossing the budget, stop retaining further
  control sources, mark coverage partial with a typed control-budget issue that names
  the affected directories, and keep the roll-up answer (`fdu-1onj`).
- [ ] Deduplicate retained sources by the `ControlIdentity` fingerprint already
  computed, so identical control files are charged and compiled once (`fdu-szkg`).
- [ ] Split the constant into a strict snapshot-parser guard and a separate, larger
  runtime retention budget; make the runtime budget liftable from the command line and
  name it in the diagnostic (`fdu-okne`).
- [ ] Establish whether peak memory grows unbounded on `~/Library`-shaped trees — deep,
  wide, many small files — and bound whatever accumulates, keeping that question
  separate from TCC-induced slowness (`fdu-6o5o`).
- [ ] Re-measure `~`, `~/wrk`, and `~/Library` on macOS against `main` as the control,
  and record the regime alongside the numbers.
- [ ] Attribute the peak-memory gap against dust — 1.5 to 1.6 times on every tree
  measured — between the retained index contract and retention a roll-up never uses
  (`fdu-syyl`), through the recorded peer comparison rather than by hand (`fdu-zibs`).

This phase does not change what a complete answer means.
Control state remains exact for the consumers that ask for it; what changes is that a
roll-up no longer builds it, and that exhausting its budget yields a stated partial
result instead of an error.

Acceptance for Phase 4:

- a default CLI roll-up of `~` and of `~/wrk` completes on macOS, performing no
  control-file I/O and retaining no control state;
- opened-root and inventory consumers still receive exact, removal-aware control state,
  and the `--no-default-features` build is unaffected;
- crossing the runtime retention budget produces a usable roll-up plus an explicit
  partial marker whose boundary of incompleteness is knowable, and never an aborted
  scan;
- retention is deduplicated by fingerprint, with removal semantics unchanged and tested,
  and measured retention on `~/wrk` falls by the predicted order;
- the runtime budget is settable from the command line and named in its own diagnostic,
  while the snapshot parser guard stays strict and independent of it;
- peak memory on an `~/Library`-shaped tree is bounded and measured, or the SIGKILL is
  attributed to a cause outside fdu with evidence;
- macOS measurements name platform, host, and cache state, and use `main` as the
  control.

### Phase 5: End-to-End Integration Proof

This phase changes no default.
It proves the composed product through the same public routes and lifecycle that the
browser uses, then produces the evidence required for a separate rollout decision.

- [ ] Run the same provider conformance registry against Python and fdu providers.
- [ ] Expand the File Rollup packet to include basename-to-logical-extension derivation,
  not only rows whose logical extension is already supplied.
- [ ] Add cross-platform path fixtures for invalid Unix bytes, Windows separator
  normalization and unpaired surrogates, non-ASCII Unicode, and portable-directory
  completeness around children whose names need escaping.
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

Acceptance for Phase 5:

- both providers pass the same closed conformance registry;
- complete settled responses and replay checkpoints agree exactly;
- escaped path rows, roll-up counts, row order, aggregate bounds, and state vocabulary
  agree on every platform fixture;
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

## File and Function Execution Map

This map is the implementation authority below the architectural sections above.
If implementation discovers that a named function cannot preserve its assigned
invariant, update this map and its bead before widening the function’s responsibility.
Do not solve the mismatch by adding a second live authority or a translation cache.

### New source-file boundaries

The rewrite adds a small number of named modules instead of continuing to grow
`index.rs`, `watch_session.rs`, and the Python binding monolith.

| File | Owns | Must not own |
| --- | --- | --- |
| `crates/fdu-core/src/engine_contract.rs` | Public value types: `EngineVersion`, state values, `EffectiveChange`, `Impact`, `Work`, `Commit`, read/change/refresh request and result envelopes, typed limit and continuation errors. | Locks, threads, filesystem I/O, MetaBrowser query names. |
| `crates/fdu-core/src/index.rs` | Detached tree facts, reducers, prepared-input validation against current facts, exact atomic application, maintained indexes, and the bounded exact history behind existing nonblocking `Index::since`. | Opened-root identity, blocking waits, observer threads, page-token authority. |
| `crates/fdu-core/src/control.rs` and `control/gitignore.rs` | Bounded removal-aware control sources, matcher construction, affected-subtree calculation, and fixed ignore semantics. | Generic tags, generic promoted planes, filesystem walking at bind time. |
| `crates/fdu-core/src/admission.rs` | One lexical/native admission decision used by every scan path, refresh, watch verification, and portable projection. | Query depth or discovery budgets. |
| `crates/fdu-core/src/opened.rs` | Direct public `OpenedIndex` API, private shared state, lifecycle, cancellation, worker registry, five synchronous operations, and joined close. | A parallel `Owner` service, method-for-method forwarding, projection algorithms, journal storage, token encoding, or async runtimes. |
| `crates/fdu-core/src/opened/read.rs` | Coherent read assembly and dispatch to bounded native projections. | Filesystem reads or application query policy. |
| `crates/fdu-core/src/opened/journal.rs` | Opened-root cursor validation, condition-variable polling over the index’s one exact commit history, reset, and close wakeup. | A copied commit store, provider recovery, or per-subscriber queues. |
| `crates/fdu-core/src/opened/continuation.rs` | Bounded handle-local continuation records and eviction. | Signed or self-describing public tokens, historical index images. |
| `crates/fdu-core/src/scan.rs` | One-shot scan/reconcile plus reusable verified discovery production, scheduling frontier, and refresh verification. | A second live authority or consumer-visible change truth. |
| `crates/fdu-core/src/watch.rs` | Native or scripted hint capture, bounded coalescing, verification, and gap reporting. | Direct index mutation, journals, projection invalidations. |
| `crates/fdu-py/src/opened_binding.rs` | PyO3 conversions and GIL-detached calls for the synchronous opened-root API. | Async bridging, query aggregation, package policy. |
| `crates/fdu-py/python/fdu/opened.py` | Direct public Python namespace, immutable values, and ergonomic validation over `_native.OpenedIndex`. Keeping it as the public submodule preserves every existing top-level one-shot name without adding a forwarding façade. | Background executors, an event loop, MetaBrowser vocabulary, or duplicated engine decisions. |
| `src/metabrowser/inventory_engine/providers/fdu_inventory.py` in MetaBrowser | Contract mapping, one async change bridge, optional-package error handling. | Filesystem walking, retained entry replicas, aggregate stores, identity recipes. |

`Index` remains the detached, independently owned one-shot value for compatibility.
Its existing `since` surface becomes a nonblocking compatibility view over exact
commits; opened-root polling adds identity and waiting without duplicating those
commits.
`IndexHandle` may remain as the existing short-write coordination primitive, but
it is not an opened-root session and carries no session, continuation, or shutdown
authority.
`OpenedIndex` clones share exactly one live state; `close()` closes that state
for every clone. `watch_session.rs` continues to serve the existing CLI watch surface
during the rewrite and becomes a thin compatibility consumer only after the opened-root
implementation is complete.

### Checkpoint 1A: observable oracle

Bead `fdu-utf1` integrates the existing `fdu-9tdm` and `fdu-o8r8` prerequisites.

| File or function | Change | Proof |
| --- | --- | --- |
| `tests/golden/cli-content.tryscript.md` | Replace each Node command that parses a complete JSON report and prints selected fields with either the direct complete deterministic JSON response or a focused Rust/Python test. Preserve Node commands that only create or inspect fixture state. | `rg` finds no product-output parser that discards adjacent contract fields; reviewed golden diffs show the whole answer. |
| Any future `tests/golden/cli-cost.tryscript.md` | Do not import PR #47’s relation-only golden. Put counter relationships in `counters.rs`, scan integration tests, or the performance harness; a golden may show one complete stable diagnostic record only if that record is itself the product surface under test. | `fdu-9tdm` closes with an audited inventory of every parsing site. |
| `crates/fdu-core/tests/reference_model.rs` | Add a dependency-free canonical tree model that recomputes parents, ordering, roll-ups, completeness, control effects, exact changes, and state from scratch. Use a fixed-seed operation generator and print the seed plus full trace on failure. | Compare every observable field after every operation; retain every minimized discovery as a named regression. |
| `crates/fdu-core/src/index.rs` test support only | Expose no production helper to the model. Add test-only constructors only for facts that cannot be expressed through public observations. | A source check and code review confirm the model does not call production reducers or mutation helpers. |
| `scripts/run-golden.mjs`, `scripts/run-parity.mjs`, `scripts/parity-classes.mjs` | Change only if broad observations require a portability class; classes match unstable fields, never whole semantic subtrees. | Golden portability and CLI/Python parity gates pass. |

The production `OpenedIndex` and exact commits do not exist at 1A. Creating the runner
or artifacts here would require a parallel test API whose values could agree with
themselves while production later diverged.
Checkpoint 1A therefore fixes the session design and seam ownership only; the owner,
discovery, read, journal, refresh, and observation beads add their own typed test
controls as those real boundaries land, and `fdu-0kv7` composes the runner and artifacts
afterward.

Gate: focused model tests, complete golden corpus, parity, docs format, `make check`,
and `make cross-lint`. Record the green commit before the registry reuse commit.

### Preparatory registry reuse

Bead `fdu-tewk` lands PR #47 `9b31220` before the `index.rs` rewrite.

The audited patch owns these exact symbols:

- `TypeRegistry::compiled`, `TypeRegistry::from_manifest`, `TypeRegistry::fingerprint`,
  registry lookup, and `classify_with` in `classify.rs`;
- the internal `parse_manifest`, `validate_manifest`, and semantic
  `manifest_fingerprint` implementation in the new shared
  `classify/type_rule_manifest.rs`;
- `ScanConfig::with_types`, `Index::types`, and registry-based classification at every
  content and scan site;
- snapshot/content invalidation from the registry fingerprint;
- the runtime-parsed-default migration test and the supplied-registry cold-open test.

The implementation commit does not add `--type-rules`, Python public models, browsing
groups, gitignore, or MetaBrowser code.
If any hunk no longer applies after 1A, resolve only the test-harness overlap; do not
manually retype the 738-line implementation unless the semantic audit rejects it.

Implementation status: complete.
The audited reuse keeps manifest parsing behind `TypeRegistry`, derives identity from
the parsed ordered values rather than source-file formatting, borrows compiled rule
text, and rejects any internal registry/scope mismatch.
Both default and no-default core suites cover classification, snapshots, and content
provenance.

### Checkpoint 1B: exact commit truth

Beads `fdu-qzqf` and `fdu-gpls` split the kernel from producer migration so each can be
reviewed independently.

#### Commit kernel

| File or function | Change | Direct regressions |
| --- | --- | --- |
| `engine_contract.rs` | Add `EffectiveChange`, `ImpactDomain`, bounded dirty paths, `Work`, `StateTransition`, and `Commit`. Keep `Observation` as verified producer input; retain `AppliedDelta` only as a compatibility projection derived from exact commit changes. | Exact enum vocabulary, bounded payload, compatibility conversion, and commit/version construction tests. |
| `Index::apply`, `apply_with`, `apply_validated`, `apply_validated_with` | Replace the boolean/request-copy pipeline with `prepare_observation` and `commit_prepared`. Preparation validates and normalizes producer input; `commit_prepared` evaluates against current facts, applies facts/reducers/state, derives impact, and advances the clock once. | Malformed input, overflow, injected reducer failure, no-op, and stale conditional input leave facts, reducers, state, journal, and clock unchanged. |
| `ensure_dir_chain`, `apply_upsert`, `upsert_beneath`, `apply_remove`, `remove_entry` | Return or record exact inserted, updated, removed, and completeness/control effects through an internal `MutationEffects`; never report the requested leaf when only ancestors or a replacement removal changed. | Port the refusal and kind-change regressions from `d58d9c5` and `fdu-a7cl`; compare complete effective changes, not non-emptiness. |
| `merge_upward`, `unmerge_upward`, `recompute_newest_upward` | Update reducers through one mutation recorder and derive aggregate impact from the ancestors actually touched. | Independent ancestor oracle from `50e078c`, tally conservation, negative/pre-epoch mtime, extension interner churn. |
| `IndexHandle::apply`, `apply_if_clock`, `begin_reconcile`, `finish_reconcile`, `invalidate_root` | Delegate to exact commits. A freshness or lifecycle change is a state-only commit rather than an unclocked side mutation. | Simultaneous writers retain contiguous versions; readers observe only whole commits; state-only change polling tests are enabled later by the same values. |
| `crates/fdu-core/tests/reference_model.rs` | Extend generated traces to cap refusal, unknown ancestry, control updates, state-only transitions, journal floor, and ABA conditional observations. | Engine and model match after every step and on the final journal range. |

Implementation status: the `fdu-qzqf` kernel is complete.
Producer input is normalized before the shared write guard; `Index` retains one bounded
exact commit journal; mutation helpers record inserted, updated, removed, invalidated,
and state-only effects; impact is derived and fail-closed when its path set exceeds the
bound; and `AppliedDelta` remains a derived compatibility view.
Detached commits carry the existing process-local clock sequence.
The opened-root layer binds that sequence to its lifetime, scope, and semantic
identities without putting live identity into clonable detached `Index` state.
The independent model compares exact commits, impact, work, journal floor, ABA
arbitration, and reconciliation state transitions.
Resource refusal, control changes, and rejection of unknown live ancestry remain with
their owning producer, budget, and control beads rather than speculative kernel hooks.

#### Producer migration

| Current seam | Required edit | Reused proof |
| --- | --- | --- |
| `scan_internal`, `scan_into_index*` | Produce verified parent-first inputs and use the exact baseline commit path. Preserve the blocking one-shot return and existing scan diagnostics. | Current `scan_populates_an_index_end_to_end`, baseline, depth, scope, serial/parallel equivalence tests. |
| `reconcile_target_inner`, `reconcile_direct_parallel`, `apply_deferred_reconcile`, `flush_direct_reconcile_batch`, `flush_reconcile_batch` | Carry normalized verified inputs and conditional expectations into `commit_prepared`; stop publishing requested batches. | Current stale-arbitration, overflow retry, concurrent invalidation, widening, symlink, and scope mismatch tests. |
| `watch::apply_intent`, `apply_observation`, `apply_reverified_with` | Verify outside the index guard, then conditionally call the exact commit path. Watch code receives the returned commit; it never reconstructs it. | Current blocked-verifier, contention, error-no-mutation, root-disappearance, and scope rejection tests. |
| `watch_session::Session::next_batch` | Transitional adapter translates exact commits for the existing CLI watch surface. It does not own the new journal. | Existing `watch_session_integration.rs` and CLI watch goldens remain unchanged. |
| `lib.rs` open and cache paths | Keep one-shot public behavior. Route any revalidation and state movement through the same internal commit path. | Existing warm/cold/cache-only/snapshot and surface parity corpus. |

Unknown live ancestry has one path: the producer schedules reconciliation from the
nearest retained ancestor and admits the child only after verified parents exist.
Ancestry is never synthesized.
Cold producers publish a directory before making its children claimable, and the
snapshot loader inserts each record beneath the explicit parent recorded in the snapshot
format.

Implementation status: complete.
`make check` and `make cross-lint` pass with the exact producer contract in place.
The parallel cold walker now establishes parent-first causal publication without a
global level barrier; every reconciliation and watch callback receives the exact commit
returned by the index, including state-only reconciliation boundaries; watch
verification turns unknown ancestry into one bounded reconciliation request from the
nearest known directory; and the transitional watch session derives its legacy change
view from exact effective changes.
The independent reference model no longer manufactures ancestry and requires generated
producers to supply exact parent facts.

Gate `fdu-qzqf` with model/fault/concurrency tests, then gate `fdu-gpls` with scan,
reconcile, watch, one-shot parity, `make check`, and `make cross-lint`.

### Checkpoints 1C and 1D: control, admission, and identity

Beads `fdu-wzu9` and `fdu-ff6r` finish the kernel before a worker is added.

| File or function | Change | Reuse and proof |
| --- | --- | --- |
| New `control.rs` | Add `ControlTable::{upsert, remove, matcher_for, affected_subtree}` with one shared bound enforced at mutation, snapshot save, and load. Store exact source identity and parsed matcher by directory. | Extract control-source and deletion cases from `f4c60ed`, `be04134`, and `d58d9c5`; add last-control deletion, create/delete churn, at-bound self-roundtrip, and no second tree walk. |
| New `control/gitignore.rs` | Parse and evaluate the fixed `.gitignore` semantics required by MetaBrowser, including nested negation and removal. Dependency choice is made under supply-chain policy; if the reviewed crate raises MSRV or size without enough benefit, keep the narrow parser in core. | Port the prototype corpus, then compare provider order against MetaBrowser on the shared fixture. |
| `index.rs` `PartitionRollUp` | Maintain only `all` and `unignored` roll-ups plus the registry-derived classification dimensions used by existing reports. Control changes prepare exact reclassification moves and commit them atomically. | Extract generic-plane merge/unmerge tests from `a6a89ab` and `7aaaf84` without their abstraction. |
| New `admission.rs`; `scan.rs`; `scan/macos_bulk.rs`; `watch.rs` | Centralize hidden, symlink, filesystem-boundary, and object-kind admission. Every scan acceleration and live path calls the same decision. Control-file signals bypass ordinary row admission without creating a visible row. | Extract `6a8ac6f`, `ff210d0`, and `048b0cc`; add `scripts/check-admission-sites.mjs`, Unix invalid-byte, Windows surrogate/separator, macOS bulk, FIFO/socket, and control-file cases. |
| `classify.rs` and new `classify/file_rollup_manifest.rs` | Add portable `logical_ext`, registry-owned canonical extension and name classification, ordered browsing groups and families, and the dependency-free validated File Rollup v3 profile. Keep the compiled analyzer registry and existing `derive_ext` answer stable for detached and CLI consumers. | Parse the exact shared document; prove formatting-insensitive semantic identity, exact-basename precedence, longest-suffix matching, unknown fallback, compact-manifest compatibility, and cross-platform components. Reject malformed and unsupported documents before opening a root. |
| `Index`, `IndexHandle`, and new `OpenedIndex` boundary types | Keep cloned `Index` detached. Do not put session identity, worker ownership, journal waiters, or continuations into it. Reserve those for the Phase 2 opened-root state. | Clone independence, no shared live identity in snapshots, and existing `IndexHandle` read/write behavior. |
| `snapshot.rs` `save`, `save_handle`, `load`, `put_scope`, `read_scope`, `engine_fingerprint` | Serialize detached facts, control table, reducers, validated scope, and semantic identity only. Bump format/fingerprint once for the cumulative representation change; reject partial-resource baselines. | Existing corruption/size/atomicity tests plus registry, control-removal, portable-path, and partial-baseline cases. |
| `Cargo.toml`, `crates/fdu-core/Cargo.toml`, `crates/fdu-py/Cargo.toml`, `Makefile`, CI | Make core default features empty; keep `watch` and any `gitignore` dependency removable and explicit; update library-only feature matrix, audit pins, and recorded size commands. | `cargo tree` deltas, `make check`, `make cross-lint`, MSRV, audit, no-default tests, CLI/wheel size baselines. |

The 1D green checkpoint is the base for every opened-root commit.
No Phase 2 bead starts if a one-shot surface differs without a reviewed correction.

### Phase 2: opened-root vertical slice

#### Shared live state and progressive discovery

| File or function | Change | Gate |
| --- | --- | --- |
| New `opened.rs` `OpenedIndex::{open, read, changes, refresh, prioritize, close}` | Implement the operations directly on `OpenedIndex`, backed by a private `Arc<OpenedState>` that holds the guarded `Index`, lifecycle data, cancellation, join handles, journal wait state, and continuations, and binds root, options, scope identity, semantic identity, and a fresh opaque session identity. `OpenedState` is storage and synchronization, not an internal service API. The associated `open` constructor avoids colliding with the existing free one-shot `open`. Workers hold `Weak<OpenedState>` or narrower state so they cannot create a last-reference cycle. | `fdu-mkga`: old and new open contracts coexist, clone-wide close, concurrent close with one shared terminal result, close during every blocked operation, final-reference fallback, poison/panic propagation, and no worker after success. API-shape review confirms that `OpenedState` has no mirror of the six public methods. |
| `lib.rs` | Export the new synchronous values without changing existing `open`, `OpenConfig`, `Index`, CLI, or Python one-shot defaults. Update crate-level docs to distinguish detached indexes from opened roots. | Rust docs and backward-compatibility tests. |
| `scan.rs` reusable discovery producer | Split verified entry production and traversal scheduling from `scan_into_index*`. Feed bounded parent-first `PreparedCommit` inputs through the opened-root commit path; retain the current blocking consumer for one-shot scans. | Same tree and diagnostics under one-shot and opened-root settled reads. |
| `opened.rs` discovery frontier | Track shallow pending directories, per-directory completeness, bounded commit batches, cancellation, and `prioritize` scheduling hints. Apply a file-retention execution budget without promising a deterministic prefix. | `fdu-194x`: first useful shallow read, parent-before-child, priority changes order only, exact file bound, limit-without-refusal remains complete, first refusal becomes typed partial, stopped session does not watch or expand. |

#### Reads, journal, refresh, and observation

| File or function | Change | Gate |
| --- | --- | --- |
| New `opened/read.rs` `read`, `lookup`, `tree_page`, `flat_page`, `rollup_report`, `diagnostics` | Capture one guard/version/state boundary and return projections in request order. Charge rows visited, returned, and maintained-index work. Never do filesystem I/O or a full sort under the guard. | `fdu-r7s7`: coherent mixed projections, three-valued absence, portable incomplete directories, exact/capped totals, row/work bounds, current report equivalence. |
| `index.rs` maintained indexes | Move Phase 2 portable path order and direct-child partitions into an optional `ServingIndexes` allocation enabled only by `OpenedIndex`. Add one per-partition semantic type tally plus declared exact-basename classifier keys, and one global portable-file recency order there. Reuse the base index’s raw-extension and `all`/`unignored` roll-ups. Derive canonical-extension, family, group, preset, and navigation rows at the read boundary; add no duplicate dimension or catalog index. Each serving-index update is part of the exact commit and reversible on removal. | `fdu-ixhy`: independent reference-model comparison, tally/index conservation, exact add/update/kind-change/reclassification/removal behavior, resource and memory counters, measured opened-root commit cost, and proof that a detached index and one-shot CLI allocate no serving state. |
| New `opened/journal.rs` `JournalWait::{notify_commit, poll, reset_at, close}` plus `Index::since` | Retain exact commits once in the index, validate opened-root session and sequence, wait on a condition variable, return idle without moving the cursor, distinguish consumer reset, and wake on close. No copied commit store or subscriber queue. | `fdu-ngnm`: detached `since` and opened polling return the same commit range; immediate, blocking, timeout, state-only, slow consumer, floor, future/foreign, close, cancellation, and bounded-memory cases; extract `fad3d2f`, `1e1c207`, `44e79c3`, `eaae030`. |
| `scan.rs` `normalize_subtree`, new `normalize_refresh_paths`, reconciliation functions | Generalize the sound PR #47 `d19b0ce` algorithms to a bounded path set and one conditional exact commit stream. Return accepted/rejected paths and the committed journal range. | `fdu-3za7`: canonical validation, duplicate and descendant collapse, missing/non-directory widening, control files, budget refusal, overlap with discovery/watch, and exact receipt. |
| `watch.rs` plus `watch/scripted_events.rs` | Separate capture from verification. Capture before baseline where supported, buffer bounded hints, report overflow/gaps, reconcile, and use the opened-root commit path for the final state transition to `watching`. | `fdu-9jzp`: scripted before/during/after baseline events, overflow, registration gap, final reconciliation, live mutation, disabled-watch behavior, and deterministic shutdown. |
| Transitional `watch_session.rs` | Consume the same verifier and exact commits for the CLI; do not share `OpenedIndex` journal or lifecycle state unless it can become a truly thin adapter. | Existing watch behavior and goldens. |

Each capability adds only its own typed, per-owner test seam while its production
boundary is in view: lifecycle and joined shutdown with `fdu-mkga`, discovery order and
budget with `fdu-194x`, complete read envelopes with `fdu-r7s7`, wait boundaries with
`fdu-ngnm`, conditional-commit barriers with `fdu-3za7`, and scripted hints with
`fdu-9jzp`. Bead `fdu-0kv7` then composes those seams into the five canonical sessions,
artifact tooling, invariant checks, and automatic coverage closure.
It adds no new production seam and blocks the Python surface.

#### Python surface

| File or function | Change | Gate |
| --- | --- | --- |
| New `crates/fdu-py/src/opened_binding.rs` `PyOpenedIndex` | Bind the five synchronous operations and value conversions. Use `py.detach` for native open/read/change poll/refresh/close and any substantial projection. Store only the shared native handle. | `fdu-bnsk`: real thread overlap, a read during commit, change timeout without GIL starvation, iterator-independent handle use, concurrent close, and post-close typed failures. |
| `crates/fdu-py/src/lib.rs` module registration | Register `PyOpenedIndex` and conversion helpers; move no opened-root lifecycle or state logic into the existing monolith. | Extension and embeddable modes compile. |
| New public `python/fdu/opened.py` and `_native.pyi` | Add immutable typed models and the direct wrapper in one cohesive opt-in namespace. Keep async code out of the package and preserve every existing top-level one-shot name. | public smoke, BasedPyright, sdist/wheel smoke, parity, installed import. |

Phase 2 closes only when all five operations are useful through Rust and Python, the
watch-disabled build is complete, and close leaves no native or Python worker alive.

### Phase 3: measured MetaBrowser adoption

MetaBrowser paths below are relative to the reviewed PR #74 checkout.
Every MetaBrowser implementation commit records the exact fdu revision whose wheel it
used; every fdu counterpart commit records the exact MetaBrowser revision whose contract
and fixtures it used.

#### Checkpoint 3A: unchanged-contract cost spike

Bead `fdu-sewa` owns a disposable experiment before either provider contract changes.
Its five ordered children are `fdu-q21b`, `fdu-x53q`, `fdu-cdc2`, `fdu-jvpr`, and
`fdu-pe58`.

| File or function | Temporary change and measurement | Retained result |
| --- | --- | --- |
| New `explorations/fdu-inventory-adapter/README.md`, `adapter.py`, `probe.py`, and `run.py` in MetaBrowser | Record exact revisions and the wheel build/install command; implement the smallest adapter from `PyOpenedIndex` to the existing `InventoryHandle` protocol; allow full row materialization, Python sorting, repeated scans, and exact-remainder calculation only when each cost is counted. | A reproducible harness, normalized evidence, and report. The naive adapter remains quarantined and unpackageable until the thin provider replaces it, then is deleted. |
| Spike-local composition over `InventoryCoordinator` and `tests/inventory_harness.py` | Supply the experimental backend directly to the coordinator or a test-owned runtime. Do not register `fdu` in the shipping factory or change `METABROWSER_INVENTORY_PROVIDER` during 3A. | Provider-neutral route and lifespan tests exercise the real coordinator, overlay, event bus, and routes without a production selection path. |
| `contract.py` query registry and `tests/test_inventory_provider_contract.py` | Record which query and projection fields each route requests, which orders and totals are visible, and which bounds are relied on. Reuse the existing registered cases; do not edit the contract or copy their expectations. | A closed evidence table mapping visible requirements to native index or contract work. |
| `python_inventory.py` `_capture_image`, `_read_sync`, `_project_query`, projection helpers | Run the same counters around the reference provider so the comparison distinguishes inherent product work from adapter duplication. | Rows visited/returned, full sorts, materialized bytes, aggregate passes, latency, and peak memory per query. |
| `server.py` `_read_tree_from_provider`, `api_tree`, `api_rollup`, `api_recent`, diagnostics route; `test_browser_inventory_api.py` | Run existing route tests and representative requests without route changes. | Exact public ordering, totals, page behavior, and latency needed for 3B acceptance. |
| `test_browser_lifespan_e2e.py` and `test_e2e_filesystem_to_sse.py` through the spike harness | Run one exact-wheel lifecycle from progressive open through live mutation, reread, refresh, root replacement, iterator cancellation, and joined close. | A composed trace proving that the Python wrapper has the shape MetaBrowser actually needs; failures become 3B/3C requirements rather than adapter workarounds. |

No native maintained index is approved merely because PR #47 contained one.
Bead `fdu-hgnj` must cite the 3A observation that each index eliminates.
Checkpoint 3A closes when the report identifies every incomplete or expensive mapping,
the reusable harness and normalized evidence remain, and no experiment code is packaged
or selectable. Checkpoint 3C deletes the naive adapter when its thin replacement lands.

#### Checkpoint 3B: joint contract and Python oracle

| MetaBrowser file or function | Required edit | Gate |
| --- | --- | --- |
| `inventory_engine/contract.py` `InventoryConfig`, `__post_init__`, `inventory_scope_fingerprint` | Replace asserted registry fingerprint with immutable registry content; introduce `DiscoveryBudget`; move maximum depth to queries; name hidden, symlink, filesystem, and object-kind scope; version the new scope encoding; use provider-derived identities. | `fdu-m68r`: validation and byte-level identity cases, including non-ASCII and every field moving independently. |
| `LifecyclePhase`, `CoverageReason`, `Coverage`, `Freshness`, `SourceKind`, `IndexState` | Align exactly with the architecture vocabulary and require exhaustive mapping tests. Rename `OPENING_CACHE` to `OPENING`; do not fold reasons. | Transition graph, state explanation, and cross-provider enum-closure tests. |
| `WorkCounters`, `ReadRequest`, query and projection dataclasses | Add deterministic work limits and typed limit results; remove `remaining_rows`; add opaque `next_page`, one directory completeness value, exact-or-capped count, and exact row-order documentation. | Bound validation, page conservation, version pin, escaped path, capped total, and no-hidden-dimension tests. |
| `InventoryHandle` and `InventoryBackend` protocols | Keep exactly `open` plus read, changes, refresh, prioritize, close. Specify one active change iterator and iterator-close versus handle-close behavior. | Structural protocol and lifecycle tests. |
| `tests/test_inventory_provider_contract.py` | Replace exact-remainder assertions with the bounded assembly contract; expand the closed conformance registry for identities, states, paths, counts, order, paging, changes, refresh, cancellation, and close. | The revised Python provider must pass before the fdu backend is registered. |
| `providers/python_inventory.py` constructor and `start` | Parse and retain the supplied registry, derive both identities, and bind one budget policy. Stop reading module-global classification state for provider answers. | `fdu-yv1o`: injected registries produce different answers and identities in one process. |
| `_capture_image`, `_read_sync`, `_read_cached_tree_page_sync`, `_read_snapshot_sync`, `_project_query`, `_directory_projection`, `_filtered_tree_projection`, `_recent_projection` | Return the revised coherent envelopes, exact orders, bounded pages, separate totals, and work-limit results. Remove suffix counting used only for `remaining_rows`. | All eight projections pass the conformance registry and targeted-work tests. |
| `_filter_matches`, `_catalog_entry_matches`, `_compute_navigation_tallies`, `file_type_tallies`, `rollup` | Use the injected registry and one explicit selection rule; keep straightforward reference aggregation where bounded by the revised contract. | File Rollup packet, catalog predicate corpus, exact/capped totals, and group/partition conservation. |
| `_run_walker`, `rewalk_subtree`, `refresh`, `apply_live_entry`, `_record_provider_change` | Enforce one resource-budget state across discovery and later expansion; stop watcher startup and expanding refresh after refusal; emit honest provider gap versus consumer reset. | Budget, refusal, watcher-gap, refresh, and lifecycle tests. |

#### Coordinator, assembly, runtime, and routes

Bead `fdu-kh2d` updates application-owned policy after the revised Python provider is
green.

| File or function | Required edit | Invariant |
| --- | --- | --- |
| `tree_page_assembly.py` `assemble_tree_pages` and `TreePageAssembly` | Enforce stable provider version, positive row bound, unique advancing opaque continuation, maximum pages, maximum rows, and request work budget. Stop checking or summing exact suffix remainders. | Assembly terminates on malicious repetition, rejects version drift, and conserves rows over a good provider. |
| `coordinator.py` `read`, `read_session`, `_compose_read_locked` | Preserve one provider read boundary and sparse overlay join. Never resort provider rows, walk returned entries, or reconstruct totals. | Existing coherent-read and decoration tests plus exact-order assertions. |
| `_run_change_relay`, `_pump_provider_changes`, `_dispatch_provider_changes`, `_publish_provider_batches`, `_merge_provider_batches` | Keep provider journal reset, provider observation gaps, and host replay loss distinct; map provider invalidations without applying row replicas. | Old-root changes cannot publish, drift resets correctly, and close joins the relay. |
| `_replace_root_locked`, `_stop_handle_locked`, `close` | Close and join the old handle and discard its continuations before publishing a new host generation. | Cancellation-drain, root replacement, and no-worker tests. |
| `runtime.py` `default_inventory_config`, `inventory_provider_from_environment`, `InventoryRuntime::{open, replace_root, close}` | Pass the actual registry document and explicit scope; keep provider choice explicit. | Runtime cache invalidation and lifecycle tests. |
| `server.py` `_read_tree_from_provider`, `api_tree`, `api_rollup`, `api_recent`, diagnostics and event routes | Request rows and product totals as separate projections in one coherent read; render `n+` for capped totals; preserve public envelopes. | Existing browser inventory API tests and provider-neutral route snapshots. |

#### Checkpoint 3C: native indexes and thin adapter

| Repository file or function | Required edit | Reuse and gate |
| --- | --- | --- |
| fdu `index.rs` maintained-index mutation hooks | Add only indexes named by 3A, updated inside every exact commit and removed symmetrically. Likely candidates are portable path order, timestamp order, registry dimensions, fixed partitions, and navigation presets. | `fdu-hgnj`: model equality, conservation, memory, commit cost, and one named 3A work reduction per structure. |
| fdu `opened/continuation.rs` `create`, `resume`, `evict`, `clear` | Store version, normalized fdu-native query identity, and last visited structural position. Use a bounded opaque ID; do not encode or sign the record. | Extract traversal/work cases from `a5a7ae3`, `91b6895`, and `051e7cc`; port only failure tests from token commits. |
| fdu `opened/read.rs` tree, flat, and ranked projections | Traverse the maintained structures in the three stated contract orders and resume without a full selection pass. Add the query’s maximum depth and ignored-subtree pruning to the tree projection, and serve ranked recency from the maintained timestamp-ordered set rather than a request-time sort over every entry. Return separate exact/capped aggregates. | Stable-version page conservation, proportional work, stale/foreign/evicted, escaped path, and close cleanup; plus the packet’s order cases, which must distinguish level order from pre-order and exercise a recency tie rather than assume one. |
| fdu `opened/continuation.rs` tree cursor | Store **one frame** — the parent’s native path, its depth, the partition, and the last emitted child name, the last being optional because a page can stop having just arrived at a parent. The anticipated depth-bounded stack proved unnecessary: the ancestor chain of that path already is the stack, recoverable by splitting the path. Do not store a frontier: it is unbounded in directory width, and re-deriving it per page makes paging one wide level quadratic in pages. | Resume cost proportional to depth rather than to rows already emitted, measured by paging one wide level to exhaustion; record size bounded by a single path. A page stopped by the work budget must carry a continuation, never `next: None`. |
| MetaBrowser new `providers/fdu_inventory.py` `FduInventoryBackend`, private handle | Map config, paths, eight queries, state, rows, work, and impact. Retain the native handle and bridge only. | `fdu-2xfp`: no walker/index/aggregate/fingerprint recipe; every query is one bounded native read. |
| `fdu_inventory.py` private change bridge | Run bounded provider operations with `asyncio.to_thread`. Give one handle one dedicated poll worker, a one-slot locked mailbox, and an `asyncio.Event` woken with `loop.call_soon_threadsafe`. Keep one result pending and do not poll or advance the local cursor until consumption. Iterator `aclose` joins the bridge only; handle `close` then joins the bridge and native opened root through `to_thread`. | Cancellation within one poll interval, later read after iterator close, concurrent bounded reads, second-iterator busy error, reset after backpressure, event-loop closure, and close during poll. |
| `factory.py` `InventoryProvider`, `create_inventory_backend`; `pyproject.toml`; `uv.lock` | Register explicit `fdu` selection and optional dependency. Missing or incompatible package is a typed startup failure; Python remains default and no automatic fallback exists. | Clean install without fdu, exact-revision wheel install with fdu, lock/supply-chain gate. |

### Phase 4: control-state scale and bound discipline

| Bead and files | Work | Acceptance |
| --- | --- | --- |
| `fdu-etfj`: `crates/fdu/Cargo.toml`, `crates/fdu-core/src/scan.rs` `read_control_op` | Gate control observation on a runtime capability rather than the `gitignore` compile feature alone. A roll-up that never consumes ignore classification must not open, parse, or retain control files. | A default roll-up performs no control-file I/O; inventory consumers still receive exact control state; the `--no-default-features` build is unaffected. |
| `fdu-1onj`: `crates/fdu-core/src/control.rs` `upsert`, `crates/fdu-core/src/index.rs` `install_controls` | Replace `Err(ControlSourceLimit)` with degradation to partial coverage carrying a typed control-budget issue, matching the resource-budget contract this plan already states for `max_files`. | Crossing the budget yields a usable roll-up and a stated partial boundary; no scan aborts on control state alone. |
| `fdu-szkg`: `crates/fdu-core/src/control.rs` `retained_source_cost`, `ControlSource` | Deduplicate retained sources by the `ControlIdentity` fingerprint already computed, so identical control files are compiled and charged once. | Removal semantics unchanged and tested; measured retention on `~/wrk` falls from 9.93 MiB toward the deduplicated 3.81 MiB. |
| `fdu-okne`: `crates/fdu-core/src/control.rs`, `crates/fdu-core/src/snapshot.rs`, `crates/fdu/src/cli.rs` | Split the constant into a strict snapshot-parser guard and a separate, larger runtime retention budget. Expose the runtime budget where it is stated and name it in the diagnostic. | The bound is liftable by a flag; the parser guard stays strict against untrusted `u32` lengths on load. |
| `fdu-6o5o`: macOS `~/Library` memory investigation | Establish whether peak memory grows unbounded on deep, wide, many-small-file trees and bound whatever accumulates. Keep this separate from TCC-induced slowness, which is not fdu’s. | Peak memory is bounded and measured, or the SIGKILL is attributed outside fdu with evidence. |
| `fdu-syyl`: peak-memory deficit against dust | Separate the portion of fdu’s peak that the retained index contract requires from the portion a roll-up never uses. Measure through the harness, paired and interleaved, not by hand. | Peak RSS on nominated macOS trees is measured and attributed; any reducible part has an owner or a recorded decision to keep it. |
| `fdu-zibs`: recorded macOS peer comparison | Run `make perf-compare-tools` over the nominated macOS subjects and record the artifact, so no fdu-versus-dust figure rests on an unpaired or cache-uncontrolled run. | A recorded artifact exists with platform, host, and cache state stated; the `~/Library` case where dust leads is explained or filed. |

### Phase 5: composed proof and rollout evidence

| Bead and files | Work | Acceptance |
| --- | --- | --- |
| `fdu-xu27`: MetaBrowser File Rollup packet, fdu vendored fixture, provider contract tests | Add basename-to-logical-extension cases, injected-registry identity, group/partition rows, invalid Unix bytes, Windows separators and unpaired surrogates, non-ASCII names, exact tree/flat order, portable completeness, and exact/capped totals. Replay one verified scripted observation sequence into both providers. | Complete settled reads and every replay checkpoint agree; partial concurrent prefixes need only agree on bounds and knowledge. |
| `fdu-bldb`: `test_browser_inventory_api.py`, `test_browser_lifespan_e2e.py`, provider/coordinator/runtime tests | Parameterize public routes over both providers; test cold useful read, completion, mutation-to-change-to-reread, refresh, root replacement, iterator cancellation, and joined close. Inject budget, stale/evicted continuation, journal floor, observer gap, missing package, and cancellation faults. | Same public envelopes except provider-specific diagnostics/work; every recovery is typed; no fallback or worker leak. |
| `fdu-bldb`: fdu wheel build and MetaBrowser CI workflow | Build the exact fdu revision as a wheel, install it into a clean MetaBrowser environment, and run the lifecycle test on every supported Python/platform job. | Source-tree imports and sibling checkout leakage cannot make the integration pass. |
| `fdu-ekad`: both performance harnesses and evidence docs | Measure time to first useful read and completion, settled query latency, page work, mutation latency, CPU, memory, continuation memory, GIL conversion, dependency trees, CLI binary, and wheel. Record host, cache, filesystem, and exact revisions. | Published evidence meets explicit thresholds or the provider remains opt-in; this bead does not silently change defaults. |

### Implementation bead graph

The tbd planning shortcut materializes the execution map as children of `fdu-snej`. An
arrow means the bead on the left depends on the bead or beads on the right.

| Phase | Bead | Status | Depends on |
| --- | --- | --- | --- |
| 1A | `fdu-utf1` observable oracle prerequisites and staged session design | Closed | existing `fdu-9tdm`, `fdu-o8r8` |
| 1C preparation | `fdu-tewk` audited runtime registry reuse | Closed | `fdu-utf1` |
| 1B kernel | `fdu-qzqf` exact prepared commit | Closed | `fdu-tewk` |
| 1B producers | `fdu-gpls` route every producer | Closed | `fdu-qzqf` |
| 1C | `fdu-wzu9` control and fixed partitions | Closed | `fdu-gpls` |
| 1D | `fdu-ff6r` admission, images, features, identity | Closed | `fdu-wzu9` |
| 2 opened root | `fdu-mkga` shared live state and close | Closed | `fdu-ff6r` |
| 2 discovery | `fdu-194x` progressive discovery, budget, priority | Closed | `fdu-mkga` |
| 2 reads | `fdu-r7s7` coherent projections | Closed | `fdu-mkga` |
| 2 journal | `fdu-ngnm` journal and change poll | Closed | `fdu-mkga`, `fdu-gpls` |
| 2 refresh | `fdu-3za7` multi-path refresh | Closed | `fdu-mkga`, `fdu-gpls` |
| 2 observation | `fdu-9jzp` no-gap handoff | Closed | `fdu-194x`, `fdu-ngnm`, `fdu-3za7` |
| 2 session proof | `fdu-0kv7` five session goldens and coverage closure | Closed | `fdu-mkga`, `fdu-194x`, `fdu-r7s7`, `fdu-ngnm`, `fdu-3za7`, `fdu-9jzp` |
| 2 Python | `fdu-bnsk` synchronous Python surface | Closed | `fdu-r7s7`, `fdu-ngnm`, `fdu-3za7`, `fdu-9jzp`, `fdu-0kv7` |
| 3A | `fdu-sewa` unchanged-contract measurement | Closed | `fdu-bnsk` |
| 3B contract | `fdu-m68r` joint contract | Open | `fdu-sewa` |
| 3B oracle | `fdu-yv1o` Python provider | Open | `fdu-m68r` |
| 3B application | `fdu-kh2d` coordinator, assembly, runtime, routes | Open | `fdu-m68r`, `fdu-yv1o` |
| 3C native | `fdu-hgnj` measured indexes and continuations | Open | `fdu-sewa`, `fdu-m68r` |
| 3C adapter | `fdu-2xfp` fdu provider and async bridge | Open | `fdu-hgnj`, `fdu-bnsk`, `fdu-m68r` |
| 4 semantics | `fdu-xu27` two-provider conformance and replay | Open | `fdu-yv1o`, `fdu-2xfp` |
| 4 product | `fdu-bldb` routes, lifecycle, recovery, wheel | Open | `fdu-kh2d`, `fdu-xu27` |
| 4 acceptance | `fdu-ekad` performance, size, rollout evidence | Open | `fdu-bldb` |

This graph makes the intended parallelism explicit.
After `fdu-mkga`, reads, journal, and refresh can proceed independently.
After the 3B contract, the Python reference provider and measured native indexes can
proceed independently.
The adapter waits for both.
No implementation bead depends on a documentation-only PR #47 commit or on the old
prototype epic’s implementation order.

### Remaining checkpoint slices

Each open checkpoint parent has ordered child beads sized for one focused implementation
and validation commit.
The parent closes only when all listed children and its checkpoint gate pass.

| Parent | Ordered child beads |
| --- | --- |
| `fdu-bnsk` Python surface | `fdu-seku` PyO3 values and handle → `fdu-2fhv` immutable models, wrapper, and stubs → `fdu-nsn3` GIL, lifecycle, typing, sdist, and installed-wheel proof |
| `fdu-sewa` unchanged-contract spike | `fdu-q21b` exact branch/wheel bootstrap → `fdu-x53q` disposable adapter → `fdu-cdc2` shared provider/route instrumentation → `fdu-jvpr` full installed-wheel lifecycle → `fdu-pe58` evidence publication and adapter quarantine |
| `fdu-m68r` joint contract | `fdu-hnyg` configuration and identity → `fdu-8qpb` state, queries, pages, counts, and work → `fdu-mtl6` closed conformance registry |
| `fdu-yv1o` Python reference provider | `fdu-mcbx` registry, identity, and budget → `fdu-lx1j` bounded projections, pages, order, work, and totals → `fdu-ljge` lifecycle and eight-query gate |
| `fdu-kh2d` MetaBrowser application | `fdu-i0pg` bounded assembly and coherent reads → `fdu-3aej` relay, root replacement, and shutdown → `fdu-9bzr` runtime and routes |
| `fdu-hgnj` native indexes | `fdu-5qqr` evidence-based index decision → `fdu-ixhy` exact-commit index maintenance → `fdu-lkcv` continuation authority → `fdu-a0cf` bounded projections |
| `fdu-2xfp` thin fdu backend | `fdu-jy8p` exhaustive value/query mapping → `fdu-o5ne` bounded async bridge → `fdu-9dg9` optional packaging and selection → `fdu-zop7` contract, structure, concurrency, and clean-install gate |
| `fdu-xu27` semantic agreement | `fdu-ekga` shared File Rollup packet → `fdu-m51u` path, ordering, completeness, and total semantics → `fdu-e76r` recorded-observation replay |
| `fdu-bldb` composed product | `fdu-lr6r` provider-neutral routes → `fdu-cuwr` complete installed-wheel lifecycle → `fdu-blqg` recovery matrix → `fdu-4w03` exact-revision cross-repository CI |
| `fdu-ekad` final acceptance | `fdu-giss` paired protocol and thresholds → `fdu-umwm` composed measurements → `fdu-vmzf` dependency, size, rollback, and rollout disposition |

### Ordered path to a mergeable branch

The engine is sound; what remains is composition, and it has one ordering constraint
that is easy to get backwards.

Settle the engine’s contract before aligning the other side against it.
Aligning MetaBrowser first means aligning it to a moving target, and the adapter that
proves the two agree cannot be written against either side while either is still moving.

| Order | Bead | Why here |
| --- | --- | --- |
| 1 | `fdu-pokc` tree level-advance bound | The tree projection can report `rows_visited` above the requested `max_work`. That is not merely a slow page: it is a value the contract on the other side is specified to reject, so it has to be closed here or renegotiated in both documents. |
| 2 | `fdu-shkr` total portable encoding in the MetaBrowser contract | The largest remaining divergence, and it touches a value on every row. `PortablePathEncoding`, `PortablePathIssue`, and four `portable_issue` fields still exist there and no longer exist here, so any cross-provider replay diverges on contact. |
| 3 | `fdu-a0cf` remainder | The complete-or-limit selected-tree report, and the maintained Recent and Navigation readers. Measurable today: `report_work` still charges a full pass for those views because they are not yet served from `recent_files` and `semantic_by_directory`. |
| 4 | `fdu-2xfp` thin fdu backend | The adapter proves the two surfaces agree rather than asserting it, and it depends on both sides having stopped moving. |
| 5 | `fdu-ekad` final acceptance | Wheel bytes, cold startup, peak memory, and GIL cost, once there is nothing left to change. |

Two constraints inside that order are worth stating, because each inverts if read
casually.

`fdu-3v0d` — the specified work-budget enforcement that `assemble_tree_pages` does not
implement — must land *after* `fdu-pokc`, never before.
Enforcing a bound the engine does not yet honour converts correct pages into consistency
errors, on exactly the trees where the overrun happens.

The engine has never overrun a bound in a way that lost data, and must not start.
A page stopped by the work budget carries a continuation; the failure this ordering
guards against is a rejected page, not a wrong one.
That distinction is what makes the enforcement safe to add second rather than urgent to
add first.

## Testing Strategy

### Test architecture: one trace, three oracles, few boundaries

The test suite should make the live engine easier to change, not reproduce its module
graph as another large body of tests.
PR #47 added 213 Rust `#[test]` cases, 41 sequential checks in a 2,989-line installed
wheel smoke script, and 26 tests in a 1,235-line real-watcher integration file.
Many of those tests found important defects and contain excellent oracles, but their
aggregate shape is too expensive to extend and too coupled to the prototype’s public
types.

Most underlying index mutation, roll-up, classification, query, scan, and platform logic
is already stable and extensively tested.
The new risk is the opened-root layer that keeps that logic alive: ownership, discovery
scheduling, observer handoff, refresh arbitration, atomic publication, journals,
continuations, Python conversion, and MetaBrowser lifecycle.
The new corpus concentrates there and keeps the mature module tests unless a session
demonstrably subsumes a duplicate integration case.

#### Alternatives and decision

| Design | Advantages | Costs and failure mode | Decision |
| --- | --- | --- | --- |
| Large black-box filesystem matrix | Exercises real system calls and observer backend. | Slow, timing-sensitive, hard to force rare races and recovery, and duplicates setup around each assertion. | Keep only a few boundary smokes. |
| Pervasive internal event logging behind a runtime flag | Can expose every queue, worker, and lock step. | Creates a parallel behavior vocabulary, makes harmless refactors rewrite goldens, and adds branches, serialization, and possible binary cost to release code. | Reject. |
| Fully simulated state machine | Extremely fast, deterministic, and easy to generate. | Can agree with itself while bypassing the live state, workers, locks, journal, and binding orchestration that most needs proof. | Use only as an independent oracle. |
| Full dependency injection for clock, filesystem, scheduler, executor, and queues | Can control nearly any condition. | Adds traits, generics, indirection, and test-shaped production architecture far beyond the product’s needs. | Reject. |
| Controlled transparent-box session driver | Runs the real `OpenedIndex`, workers, commit path, journal, and API code, forces important schedules, and produces one inspectable causal artifact. | Requires a deliberately small test-only control seam and a complete production value model. | Choose as the primary integration mechanism. |

The selected control is a typed value passed to a test-only constructor, not an
environment variable or process-global flag.
Parallel scenarios therefore cannot alter one another.
It can choose worker count and deterministic discovery order, install the scripted
observation-hint source, pause and release named production boundaries, and trigger
named worker or queue failures.
It cannot inject retained facts, publish a commit, mutate state, or replace an operation
result. Fixtures and ordinary production verification determine facts.

The recorder sits outside `OpenedIndex`. It calls the same five handle operations as a
client, drains exact commits through `changes`, and serializes those production values
with one test-only renderer.
“Every event” means every contract-relevant causal event: the complete action, result,
commit, state/work/recovery value, named barrier used to force the causal order, and
final joined shutdown.
Internal queue pushes, mutex acquisitions, thread IDs, and elapsed time are neither the
contract nor golden text.
When one of them matters, a programmatic invariant proves it without turning the
implementation into the expected behavior.

The rewrite uses four complementary layers:

| Layer | Purpose | Shape |
| --- | --- | --- |
| Canonical opened-root sessions | Reveal the complete behavior of representative lifecycles. | Five deterministic, bounded, human-reviewed trace artifacts. |
| Independent model and generated sequences | Search many operation combinations without recording thousands of expected files. | One recomputing model, a dependency-free fixed-seed generator, and minimized regressions promoted into the session corpus. |
| Shared provider contract packet | Prove Python and fdu providers implement the application boundary rather than merely agreeing with themselves. | One provider-independent data packet with inputs and expected semantic results, run by both providers and through fdu’s Python binding. |
| Real boundary smokes | Prove the mocks and packaged surfaces reach the real operating-system and distribution boundaries. | A few causal native-observer, installed-wheel, route, and shutdown tests; no duplicate semantic matrix. |

A narrow unit test remains appropriate for a pure parser or arithmetic rule whose whole
behavior fits in a small table.
A new live-engine test must otherwise add a missing contract variant or state edge,
exercise a real external boundary, or preserve a minimized generated failure.
If an existing session can expose the behavior by adding one action, extend it instead
of creating another fixture and setup path.

### Canonical opened-root session harness

The core golden is a transparent-box session, not a snapshot of one final report.
Each scenario records the input tree and options, every requested action, every exact
commit, complete public operation results, bounded work, recovery, and final shutdown.
The recorder wraps real typed requests, responses, change polls, state, work, and
diagnostics in a thin session envelope.
It does not define alternate `Commit`, state, read, or error values, and production code
does not emit a second diagnostic event stream.

The session runner is therefore the first demanding consumer of the proposed API. If it
must read a private field to explain an effect, serialize an internal control-flow step,
or infer a change from before/after snapshots, stop and repair the production value
model. Exact commits and bounded diagnostics should make the complete causal behavior
naturally inspectable.

The harness is staged, not front-loaded.
Each Phase 2 bead adds the typed control for the production boundary it introduces;
`fdu-0kv7` creates the runner and checked-in artifacts only after all required public
values exist. This avoids both late synchronization retrofits and an early fake
`OpenedIndex` whose test-only values could become a second contract.

The initial files are:

| File | Responsibility |
| --- | --- |
| `crates/fdu-core/src/opened/test_support.rs` under `cfg(test)` | `OpenedTestControl`, `OpenedIndex::open_for_test`, scenario builder, scripted observer-hint source, deterministic barriers, thin session envelopes over production values, normalization, and invariant collection. The typed control is per opened root and the seams control timing or named faults, not facts. |
| `crates/fdu-core/src/opened/golden_tests.rs` | Five scenario definitions driven through real operations and change polls, model comparison after every commit, complete-trace comparison, and contract-coverage closure. |
| `crates/fdu-core/tests/golden/opened-root/*.golden` | Canonical line-oriented session artifacts, one file per scenario. |
| `scripts/check-opened-root-goldens.mjs` | Artifact lint, named update workflow, size checks, and unstable-literal checks. |
| `Makefile` | `opened-root-golden` comparison and `opened-root-golden-update SCENARIO=name`; the update target refuses an omitted scenario. |

Scenario inputs are small Rust values rather than another runtime configuration format.
They are declarative action tables consumed by one generic runner, not five hand-coded
integration programs.
Adding ordinary coverage means adding an action row or fixture fact; runner, rendering,
synchronization, and validation code stay shared.
The checked-in output uses a closed, line-oriented text schema rendered in one test-only
function. This avoids adding a serialization dependency to core or a parser solely for
tests while retaining a clean, structured diff.
The shared cross-provider packet uses JSON because Python can read it without a
dependency and it is a real cross-language artifact.

Every core session record has one of these meanings:

- `scenario`: schema version, stable fixture description, options, and declared bounds;
- `action`: open, discovery step, read, poll, refresh, prioritize, observer hint, fault,
  continuation resume, or close, with the complete request;
- `commit`: the complete production commit returned through the change surface, with
  relative version, exact effective changes, impact, terminal state, issues, and work;
- `result`: the complete public response or typed error for that action;
- `barrier`: a named deterministic interleaving point reached or released;
- `final`: complete retained facts and roll-ups, journal range, continuation count,
  worker count, and shutdown outcome.

Do not record `agreement: true` in place of state, reconstruct a “cleaner” synthetic
commit, or select a few fields from a broad response.
The artifact shows the complete actual behavior; independent relations are asserted in
code beside the comparison.
For small fixtures, record full paths, values, rows, and issues rather than checksums.

#### Stable and unstable fields

Normalization occurs once while constructing the trace, never as a loose comparison
pattern:

| Field | Treatment |
| --- | --- |
| Relative paths, options, rows, state values, changes, impact, work counts, errors | Stable and exact. |
| Root directory | Rewrite only the temporary prefix to `$ROOT`; retain every relative component. |
| Opened-root identity | Allocate `session-1`, `session-2` in observation order. |
| Engine sequence | Record relative sequence exactly from zero. |
| Observation and query instants | Inject explicit fixed instants where they affect semantics. |
| Operating-system timestamps not under test | Replace at trace construction with `[TIME]`; no regex in the expected artifact. |
| Operating-system directory size, allocation, inode, and device identities not under test | Replace at trace construction with the closed `[DIR_SIZE]`, `[ALLOCATED]`, `[INODE]`, and `[DEVICE]` vocabulary. File logical sizes remain exact. |
| Durations and thread IDs | Omit; deterministic work and named barriers are the contract. |
| Platform-specific native path spelling | Keep in a platform fixture, or map only the root separator before portable projection. |

Known values never use patterns.
The artifact linter rejects machine paths, raw session identities, wall-clock durations,
and wildcard placeholders outside the closed normalization vocabulary.

#### Five canonical scenarios

The first corpus is deliberately small:

1. **Cold progressive knowledge.** Open a rich tree, commit shallow parent-first
   batches, reprioritize one subtree, read during discovery, reach a discovery budget,
   and show `present`, `absent`, and `unknown` with directory completeness and exact
   lower bounds.
2. **Exact mutation and refresh.** Exercise out-of-order ancestors, kind replacement,
   no-op verification, control-file creation/edit/removal, the fixed `all`/`unignored`
   partition, special-object admission, resource refusal after an earlier effect, and a
   deduplicated multi-path refresh receipt.
3. **Coherent projections and continuations.** Request every native projection in one
   read, assemble tree and flat pages at limits one, two, and unbounded, check exact or
   capped totals, then exercise live, stale, unavailable, and closed continuation
   results. `Unavailable` deliberately covers both foreign and evicted tokens because the
   public API does not expose handle-local retention policy.
4. **Journal and observation recovery.** Start observation before baseline, inject a
   pre-baseline event, an event during discovery, overflow, a state-only commit, an idle
   poll, a blocked poll, consumer journal loss, and provider-gap reconciliation as two
   distinct recovery boundaries.
5. **Ownership, races, and shutdown.** Clone the handle, pause prepared work at named
   barriers, interleave refresh and observation, reject stale preparation, close while a
   poll and worker are blocked, repeat close from another clone, inject a worker panic,
   and end with no continuation, waiter, or worker retained.

Each artifact should stay below 400 lines and the complete core corpus below 2,000
lines.
Each deterministic scenario should execute in less than 100 milliseconds after the
test binary starts; the full generated model corpus should remain below two seconds on a
development build. If a scenario exceeds either budget, shard by lifecycle phase before
hiding detail.

### Automatic contract-coverage closure

A small golden corpus needs an objective answer to “what did we forget?”
The runner derives coverage keys from events it actually observed; scenarios do not
claim coverage with hand-written tags.
An exhaustive matcher over public contract enums defines the required set:

- all five operations and each closed success, recovery, and typed failure result
  variant;
- every publicly observable lifecycle phase and allowed transition edge; root binding
  completes before a handle exists and therefore has no separate `opening` phase;
- complete and each partial-coverage reason;
- fresh, reconciling, stale, and partial freshness;
- `present`, `absent`, and `unknown` knowledge;
- every exact change and impact domain;
- immediate, idle, blocking, reset, unavailable, and closed poll outcomes; future and
  foreign cursors share the public `ChangeCursorUnavailable` result;
- live, stale, unavailable, and closed continuation outcomes; foreign and evicted tokens
  share the public `ContinuationUnavailable` result;
- provider gap, consumer reset, query limit, resource refusal, worker failure, and
  joined close.

Adding an enum member makes the matcher non-exhaustive at compile time.
Adding a contract outcome to the required set without reaching it fails the coverage
test and prints the missing keys and closest scenarios.
This is coverage of behavior, not lines: a session update cannot make a missing state
edge pass merely by accepting new text.

Pairwise configuration cases cover watch on/off, hidden admission, runtime registry,
fixed ignore partition, resource budget, and portable-path representability without
running their full Cartesian product.
Cargo feature combinations remain the responsibility of `make check` and
`make cross-lint`.

### Independent model and generated sequences

Build the recomputing model before the live opened root.
It owns a canonical map of paths to facts and recomputes parents, ordering, roll-ups,
control effects, completeness, state, and expected exact changes from first principles.
It must not call production mutation helpers, reducers, classification lookup, impact
derivation, or continuation code.

A dependency-free fixed-seed generator produces bounded sequences containing upserts,
removes, kind changes, out-of-order observations, control creation and removal, resource
refusal, refresh overlap, stale preparation, and state-only transitions.
After every committed step it compares the full model and engine facts, roll-ups,
coverage, state, version movement, exact changes, and impact.
It also checks algebraic relations: no-op idempotence, remove-then-add equivalence,
serial versus admitted concurrent order, maintained index versus full recomputation, and
page conservation at a stable version.

Generated traces are not all checked in.
On failure the runner prints the seed and complete scenario text, minimizes by deleting
actions while preserving the failure, and offers the minimized case for promotion into
one of the five goldens.
This gives broad combination coverage without a new test function per discovered edge.

### Deterministic concurrency and lifecycle proof

Use injected discovery order, scripted observation, bounded queues, and named barriers
to force interleavings.
The barrier seam pauses at existing boundaries—after verification, before conditional
commit, after commit, while waiting, and before worker exit—and never supplies facts or
changes the production decision.

The scenarios force an event before baseline, an event during a baseline batch,
overflow, registration gap, overlapping refresh and observation, state-only wakeup,
journal-floor reset, close during a blocked poll, and close during prepared work.
Timing-only sleeps are not proof of these conditions.
Real observer tests use a deadline and a causally observed marker; they do not duplicate
the deterministic semantic matrix.

### Projection and continuation relations

One assembly helper runs every pageable projection at limits one, two, a boundary-sized
page, and unbounded.
For each stable-version assembly it checks positive bounds, advancing continuation,
exact order, no duplicates or omissions, proportional work, and equality with the
unpaged answer.
The same table exercises stale, unavailable, query-mismatched, and closed
continuations; scenario setup distinguishes foreign and evicted causes only when that
distinction matters to the harness, never as a fabricated public result.

Product totals are separate coherent aggregate projections.
The harness checks `exact(n)` or `at_least(n)` against independent recomputation and
never infers a denominator from page-control metadata.

### Shared cross-provider contract packet

MetaBrowser’s provider registry is a runner, not the sole oracle.
Both providers could agree on the same copied bug, so the shared packet includes
provider-independent inputs and reviewed expected semantic results.
It contains:

- the actual File Rollup registry document and expected normalized identity;
- one compact corpus covering logical/canonical extensions, groups, the fixed ignore
  partition, non-ASCII names, invalid Unix bytes, Windows separators and unpaired
  surrogates;
- the eight application queries, exact/capped totals, and portable completeness;
- **order cases that a tie alone can fail.** An order is only proved by inputs whose
  answer differs between the plausible readings, so the packet carries, for each of the
  three stated orders, a fixture that distinguishes it from its most likely alternative:
  a tree at least three levels deep and wide at the top, whose row sequence differs
  between level order and pre-order; a directory whose children mix directories and
  nondirectories whose names interleave, so a dirs-first partition is distinguishable
  from one lexicographic pass; several files sharing one modification time, so a recency
  tie-break is exercised rather than assumed; an ignored directory holding the newest
  file in the corpus, so ranking cannot be reordered by ignored state and pruning is
  distinguishable from row filtering; and one corpus whose match count exceeds the row
  bound and one whose does not, read with the same query, so a rule that fires only on
  overflow cannot pass unnoticed;
- a scripted operation sequence with expected state, change cursor, invalidation, and
  settled read after every step;
- lifecycle, budget, reset, refresh, cancellation, and close failures.

The packet is authored from the contract and independent model, not generated by
executing either provider’s matcher.
MetaBrowser owns the canonical file; fdu vendors the reviewed revision for binding
tests, and composed CI directly compares the two files before running both providers.
Git history and the exact counterpart revision provide provenance; no decorative hash is
added beside data already compared byte for byte.

### PR #47 test reuse audit

The old tests are mined by oracle, not copied by file.

| PR #47 source | Assessment and rewrite use |
| --- | --- |
| `9460231` `watch/scripted_events.rs` | Strong seam. Borrow its backend-level event vocabulary, path validation, overflow/error cases, and rule that scripts provide hints which production verification must confirm. Extend it with named barriers; do not let scripts inject facts. |
| `50e078c` ancestor-impact test | Strong independent oracle. Keep the separate ancestor calculation, but compare it with impact derived from exact committed effects rather than requested operations. |
| `2ab02ee`, `a3960fb`, `c31ad3c` coherent read cases | Strong relations. Fold single-guard version/state/projection checks and pinned-time behavior into the projection session and invariant validator. |
| `a5a7ae3`, `91b6895`, `051e7cc` paging tests | Keep assembly conservation, cross-directory order, advancing position, and proportional-work relations. Replace exact remainder and signed-token tests with handle-local stale/foreign/evicted/query-mismatch cases. One table replaces most of the 714-line paging file. |
| `d19b0ce` `batched_refresh.rs` | Keep ancestor collapse, bounded rejection, empty batch, one terminal commit range, and batch-versus-independent-recompute cases. Move them into the exact-mutation session and generator. |
| `5ace86c`, `6a8ac6f`, `ff210d0`, `048b0cc` platform and admission cases | Keep invalid native paths, hidden control signals, macOS admission parity, Windows spelling, special objects, and serial/parallel equivalence as fixture rows. Avoid a separate integration file per scope axis. |
| `a07fa17`, `ac38584`, `fad3d2f`, `44e79c3`, `eaae030` state and journal cases | Keep exhaustive state vocabulary, state-only version movement, cursor/commit ordering, consumer reset, and terminal-state-at-cursor relations. Express them as automatic coverage keys and the journal session. |
| `37e791f`, `825fd92`, `c4f3343` ownership and synchronization regressions | Keep shared-authority, wait-on-the-causal-index, and select-the-semantic-batch lessons. Reproduce them with deterministic barriers, not sleep or “first dirty batch” assumptions. |
| `walk_budget.rs`, `hidden_admission.rs`, `special_objects.rs`, `plane_equivalence.rs` | The fixtures and independent maintained-versus-walked comparisons are useful. Merge their cases into the rich corpus; replace exact-prefix scope and generic promoted planes with discovery-budget truth and fixed `all`/`unignored`. |
| `catalog-predicates.json` and its generator | Keep the compact corpus and predicate boundary cases. Do not keep expected answers generated by lifting and executing the provider implementation, or the embedded provider source; those characterize one implementation rather than independently test the contract. |
| `scope-fingerprint.json` | Keep order-insensitive collection and non-ASCII encoding vectors. Rebuild expected values for the new scope schema; `max_depth` and discovery budget no longer belong to identity. |
| `tests/golden/cli-cost.tryscript.md` | Reject its product-output parsing. Move syscall/work relations to the invariant runner or performance harness; a CLI golden may show a complete stable diagnostic record only if that whole record is the public surface under test. |
| `crates/fdu-py/tests/public_smoke.py` | Keep installed-wheel isolation, public export/stub parity, one complete five-operation lifecycle, and GIL-detached concurrency. Replace the 41-check sequential grab bag and its AST “all checks were called” self-test with the shared packet plus small, independently reported tests. |
| real `watch_session_integration.rs` cases | Keep a minimal create/remove delivery and idle-no-work platform smoke. Move gaps, overflow, filters, budgets, state, ordering, and shutdown to the scripted deterministic sessions. |

The quality assessment is therefore mixed but favorable at the assertion level: PR #47
contains many precise causal regressions and several genuinely independent oracles.
Its weakness is topology—similar fixtures and surface assertions accreted across Rust,
Python, goldens, examples, and live timing tests.
The rewrite preserves the information and removes that duplication.

### Golden update and review discipline

The default target always compares.
Updating requires `SCENARIO=name`, writes one artifact, reruns its model and invariant
checks, and prints the ordinary git diff.
There is no update-all shortcut during implementation.
Before commit, the complete corpus runs and the reviewer reads every changed session as
a behavioral change.

The artifact checker enforces:

- one known trace schema and complete event shapes;
- exact stable fields and only the central unstable placeholders;
- bounded artifact and event sizes;
- no duplicate scenario names or orphaned expected files;
- no `grep`, `jq`, `head`, `tail`, or inline parsing that reduces a product response to
  selected scalars in a golden path;
- fixture-setup scripts remain allowed when they create state rather than hide output;
- every critical invariant is programmatic as well as visible in the trace;
- CI is proven to fail after a deliberate unapproved trace change.

The existing CLI tryscript corpus continues to own human-facing CLI behavior.
`fdu-9tdm` audits and repairs its surgical parsing sites before the opened-root corpus
lands; the new interactive API does not acquire a CLI solely to make it golden-testable.

### Minimal real boundary suite

Mocks and scripts stop at boundaries whose behavior they cannot prove:

1. one native observer smoke per supported platform causes a create and remove, waits on
   the causal change rather than sleeping for correctness, and proves idle observation
   performs no filesystem work;
2. one clean installed-wheel smoke imports only the wheel, performs all five synchronous
   operations, verifies public/stub parity and GIL release, then closes with no worker;
3. one MetaBrowser installed-wheel lifecycle opens cold, reads useful progress, observes
   a real mutation, rereads after invalidation, refreshes, replaces the root, and
   closes;
4. existing provider-neutral route and browser tests run against both providers without
   duplicating the semantic packet.

These tests may take seconds and run at the appropriate integration gate.
They do not carry the combinatorial correctness burden of the deterministic corpus.

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
4. Complete the Python portion of Phase 2 on the same branch and record its exact green
   checkpoint beside the already complete native checkpoint.
5. Run Phase 3A on MetaBrowser branch `codex/fdu-opened-root-e2e-spike`, pinned to PR
   #74 head `3183888` and an exact fdu wheel.
   Publish its full lifecycle and cost evidence, quarantine the reproducible adapter,
   then implement Phases 3B and 3C on that branch.
   Delete the naive adapter when the thin provider replaces it.
   Keep the Python provider as the default.
6. Add the fdu provider behind explicit configuration and run the complete Phase 5
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
| `fdu-sgp7` prioritize and close | Phase 2 operations on the one `OpenedIndex`. |
| `fdu-kl7r`, `fdu-vfyw` agreement proof | Phase 5 two-provider registry and observation replay. |
| `fdu-gy3g` File Rollup packet | Phase 5, expanded to exercise basename-derived logical extensions. |
| `fdu-2lkf` control-state scale | Phase 4 epic. Its P0 children gate this PR leaving draft. |
| `fdu-syyl`, `fdu-zibs` peer-comparison deficits | Phase 4. Memory, not wall time, is where fdu trails dust; measure it with the harness. |
| `fdu-tsdy` regular-file scan root | Outside this plan. Decide deliberately on the CLI surface plan rather than by omission. |
| `fdu-5ffm` macOS TCC exit 2 | `main` behaviour, not this branch. Tracked against the CLI UX plan beside `fdu-jej9`. |
| `fdu-livs` progressive provenance | Defer warm/mixed serving; cold streaming uses honest global source plus directory completeness. |

Implementation epic `fdu-snej` owns this plan.
The detailed children in the implementation bead graph now record the reviewed file and
function boundaries.
The documentation prerequisite is complete on this branch: the architecture directory
now has an explicit index, the engine design is a general undated authority, and the
principles and surface documents have distinct responsibilities.
Implementation began after the architecture review completed.
The Phase 1A golden prerequisite removes surgical CLI-output parsing, retains complete
product behavior in the corpus, and adds a checked observability policy before the
opened-root sessions are built.
The independent metadata oracle now replays fixed-seed operations against a canonical
from-scratch model and retains named ABA, refusal, journal-loss, native-name, and
reducer regressions.
Its first run found and fixed two newest-mtime repair defects: an incorrect early stop
above a repaired nested directory and accidental inclusion of symlinks and special
objects during full recomputation.
Changing a boundary first requires updating the architecture, this plan, and the
affected bead together.

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
- [fdu engine architecture](../../architecture/fdu-engine-architecture.md)
- [PR #44 design and research base](https://github.com/jlevy/fdu/pull/44)
- [PR #44 formal design review](https://github.com/jlevy/fdu/pull/44#pullrequestreview-5010948152)
- [PR #44 interactive-client plan at its final head](https://github.com/jlevy/fdu/blob/7f18f208dbd3ccb2002228bb52ae00c5d4ffcabb/docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md)
- [PR #44 contract-reconciliation research at its final head](https://github.com/jlevy/fdu/blob/7f18f208dbd3ccb2002228bb52ae00c5d4ffcabb/docs/project/research/research-2026-08-23-interactive-contract-reconciliation.md)
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
