# PR #47 Design and Merge-Readiness Review

**Date:** 2026-08-25

**Author:** fdu project, with Codex review assistance

**Status:** Current at `0558c7e`

## Executive assessment

[PR #47](https://github.com/jlevy/fdu/pull/47) should not merge in its current form.
It contains several sound foundations and a large amount of useful implementation, but
it is not a good merge unit and its architecture is not yet simple enough for the next
phase: progressive discovery, a long-lived interactive handle, and a no-gap handoff to
watching.

The difficulty is not just the inherent difficulty of live filesystem indexing.
The review history shows a repeatable pattern: bugs appear where one concept has more
than one owner or where the code mutates one representation and reconstructs another
afterwards. The important examples are:

- an index mutation represented by the observation that requested it rather than the
  changes that actually happened;
- independently mutable `Index` clones sharing one session and continuation authority;
- baseline, watch, refresh, re-tagging, and control-file discovery each carrying part of
  the live-state contract;
- admission rules copied across producer loops and protected by a source-text census;
- a stateless page token carrying trusted totals, request identity, version identity,
  and a path even though the immediate consumer is in-process;
- proposed progressive and existing watch “sessions” representing two halves of one
  opened-root lifecycle.

Those are architectural seams, not a random collection of missed edge cases.
More review rounds on the same shape will find more of the same.

The recommended course is to preserve this branch as an integration prototype and
extract a smaller sequence of changes from it.
The first extracted change should make the mutation and ownership model unambiguous.
The second should introduce one opened-root lifecycle that owns discovery, the index,
the journal, optional observation, and shutdown.
MetaBrowser should consume that handle through a thin adapter.
The CLI should remain one-shot by default and opt into the same handle only for an
explicit progress mode.

This is not a recommendation to discard the work.
The shared handle, coherent read, clocked journal, terminal-state capture, bounded
projections, scope/selection split, and much of the classification work are worth
keeping. The recommendation is to stop adding capabilities to this PR until their common
kernel has been simplified.

## Review scope and evidence

This review covers:

- the complete 85-commit diff from PR #47’s exact base, `7f18f20`, to head `0558c7e`;
- all 83 changed files, with 31,550 additions and 3,335 deletions;
- all 19 formal GitHub reviews, including the 18 with review bodies, all 10 PR
  conversation comments, and the remaining unresolved inline thread;
- the PR’s implementation map, integration contract, progressive-results plan, engine
  design principles, surface architecture, and relevant performance evidence;
- the current tbd graph under `fdu-u7vo`, including nested correctness and conformance
  work;
- MetaBrowser PR #74 at `0577bb1`, especially its implemented provider contract and
  opened-root lifecycle;
- the current code paths for index mutation, paging, watch delivery, scope admission,
  tag-rule rebinding, snapshots, Python conversion, and feature selection.

At the reviewed head, GitHub reports PR #47 as mergeable and all 19 checks are green.
It is still a draft.
It is stacked on the open PR #44, is 10 commits behind `main`, and is 97 commits ahead
of their merge base.
Green CI is useful evidence that the current tests and platform matrix agree with the
implementation. It does not resolve the open semantic defects described below; several
have tests that assert only that *some* delta was emitted, not that the delta says what
happened.

The reviewed head also passes `make docs-format`, the complete local `make check` gate,
and `make cross-lint` for the installed macOS and Windows targets.
The local gate covered all supported feature combinations, 179 golden cases, CLI/Python
parity, Python wheel and source-distribution checks, the MSRV build, and dependency
audits. The findings in this report are therefore contract and design gaps outside the
assertions of the current suite, not failures already reported by the standard gate.

The size of the change matters because it explains the review dynamics.
The diff adds about 21,653 lines of code and configuration, 7,984 lines of tests, and
1,913 lines of documentation.
The largest current modules are
[`index.rs`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/index.rs)
at 7,886 lines,
[`scan.rs`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/scan.rs)
at 7,188,
[`fdu-py/src/lib.rs`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-py/src/lib.rs)
at 3,396, and
[`watch.rs`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/watch.rs)
at 2,143. File length is not itself a defect, but in this case the largest files also
own several independent invariants, which makes the size diagnostically relevant.

## What the history says

The PR’s history is best understood as seven overlapping efforts rather than one
feature:

| Stage | Work added | What review subsequently found |
| --- | --- | --- |
| Shared state and reads | `IndexHandle`, scoped refresh, dirty roll-ups, telemetry, bounded rows | Warm snapshots used the wrong registry; watch mutated a cloned index; reads and cursors crossed incoherent boundaries |
| Classification | Runtime type registry, browsing groups, logical and canonical extensions | Snapshot identity and cross-engine classification needed stronger contracts and fixtures |
| Watch delivery | Polling and scripted backends, async Python adapters, journal batches | Cursor sampling skipped concurrent commits; state-only commits disappeared; reset and provider recovery were conflated; terminal state was missing |
| Tags and scope | Generic tags, promoted planes, gitignore, hidden and special-object admission | Control files disappeared under pruning; macOS and watch paths bypassed admission; rebind state and persisted control records drifted |
| Coherent queries | Bundled reads, reports, filtered pages, work accounting, GIL release | Report cost hid full walks; continuations were unbound, then forgeable, then cloneable; issuance and decode bounds still disagree |
| Retained budget | Native file cap across scan, refresh, and watch | Directory-boundary overshoot, scope disagreement, and finally a delta that reports the refused file rather than the actual mutation |
| Adoption proof | Reference embedder, identity recipes, catalog predicates | The fixture was one-sided; exact validation still differs; a two-engine observation replay is still absent |

The review rounds improved the branch materially.
In particular, the current code no longer updates a private watch clone, a journal slice
and its terminal state are read under one guard, provider recovery is distinct from
consumer-history reset, and invalidations-only delivery avoids crossing rows the
consumer will discard.
Those fixes are real progress.

The same history also shows why continuing in one branch is risky.
Each review round reached deeper into a new cross-product: cold versus warm, scan versus
refresh versus watch, exact versus partial coverage, current versus resumed reads, Unix
versus Windows, and CLI versus Python.
The PR is now serving as an integration branch while being reviewed as if it were a
merge-sized change.

## Design review

### What is well designed and should survive

#### One engine and thin surfaces

The repository’s strongest rule remains correct: the CLI and Python package should be
consumers of `fdu-core`, not alternate implementations.
PR #47 generally moves in that direction.
The Python GIL work, public scan-order configuration, and shared query values are good
examples.

#### The retained index, snapshot, and delta model

An index that maintains hierarchical roll-ups, a snapshot of that index, and clocked
effective changes is the right foundation for both a CLI and an interactive browser.
Filesystem notification events are hints; verified observations enter one mutation path.
That separation should remain the center of the design.

#### Coherent bounded reads

One request reading projections, version, cursor, lifecycle state, scope, and work under
one guard is the right answer to concurrent writers.
Bounded child and flat pages are also appropriate.
The earlier deep-clone approach caused an extreme regression, and the move to guarded
in-place reads was correct.

#### Orthogonal trust state

Phase, coverage, freshness, source, progress, and typed issues answer different
questions. Keeping them independent is better than one state enum that tries to encode
every combination. Capturing terminal state with a journal range is also correct: a
consumer must not reconstruct current truth by replaying an incomplete vocabulary of
transitions.

#### Scope versus selection

Depth, filesystem boundaries, hidden-path pruning, special-object admission, and a true
retention budget change what the index can know.
Query filters change only what one answer includes.
Keeping those axes separate is necessary for snapshot identity and honest absence.

#### Bounded failure and invalidation carriers

`all_dirty`, history reset, bounded issue lists, and explicit remainders honor the
project rule that truncation must be visible.
The invalidations-only interest is a good optimization for MetaBrowser, which rereads
coherent projections rather than applying entry replicas.

### Complexity that is inherent

Some complexity cannot be designed away:

- notification backends are lossy and platform-specific;
- a read spanning multiple projections needs an observation boundary;
- a retained partial index must distinguish absence from unknown;
- a slow change consumer needs bounded history and a reset path;
- version-pinned paging must fail or restart if the retained version changes;
- path identity, non-Unicode names, symlinks, filesystem boundaries, and control files
  have real cross-platform semantics;
- a cached answer and a freshly observed answer have different trust.

The goal is not a small implementation at any price.
It is one place for each of these facts and one transition path between states.

### Complexity introduced by the current design

#### 1. The PR follows the client contract horizontally instead of proving one vertical slice

The branch implements taxonomy, tags, promoted planes, multiple query projections,
catalog predicates, watch delivery, paging, file budgets, binding instrumentation, and
three surfaces before the progressive opened-root lifecycle exists.
The result is a large amount of finished perimeter around a missing center.

For the immediate client, the essential seam is only:

```text
open(root, config) -> opened root
opened root: read, changes, refresh, prioritize, close
```

The query algebra behind `read` can grow after the lifecycle is proven.
The initial vertical slice needs a coherent shallow directory read, roll-ups, state,
bounded invalidations, and cancellation.
It does not need every catalog predicate, sorted resumable page, per-projection timing
field, or CLI progress rendering at once.

#### 2. Mutation truth is reconstructed after mutation

[`Index::apply_validated_with`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/index.rs#L2568)
asks each mutation helper for a boolean and, when true, copies the original requested
operation into `AppliedDelta`. That assumes one requested operation maps to the same one
effective operation.
The file-cap defect proves the assumption false: an upsert can create ancestor
directories or remove a kind-changing row, then refuse the leaf file.
The journal records the refused file and omits the changes that occurred.

The deeper source of complexity is implicit parent creation.
An observation documented as verified truth can name a child whose parents are absent,
and the index synthesizes directories with default attributes.
That makes one input operation expand into hidden mutations and makes exact deltas
difficult by construction.

The simpler contract is:

- producers emit parents before children, including explicit verified parent upserts;
- a mutation helper records the exact effective operations it performs;
- coverage and lifecycle changes are part of the same commit;
- the clock advances only when that complete commit is ready;
- the journal stores that commit verbatim;
- every watch or progress batch is a projection of committed history, never a
  reconstruction from producer callbacks.

If out-of-order upserts remain a public convenience, normalization must expand them into
explicit effective operations before commit.
It cannot remain an invisible side effect reported as a boolean.

#### 3. Ownership is ambiguous

[`Index`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/index.rs#L1560)
is both independently mutable and `Clone`. Its clone copies the session identity,
journal, and
[`ContinuationAuthority`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/index.rs#L1303).
Two clones can diverge, reach the same clock, and accept each other’s continuation.
Separately, `IndexHandle::snapshot()` returns another mutable `Index`, even though the
operation is described as a coherent image for serialization.

There should be one live owner.
A cloneable public value should be a lightweight handle to that owner.
A persisted or diagnostic image should be a distinct immutable type with no live session
identity, continuation authority, watcher count, or mutable journal.
Removing `Index: Clone` also removes the clock-exhaustion probe’s current reason to
clone the whole tree; a prepare/commit mutation path can determine whether a no-op needs
a clock without mutating a copy.

#### 4. Baseline streaming and watch are modeled as separate sessions

The existing `watch_session::Session` starts only after an index has opened.
The progressive plan proposes another `Session::start` for the initial walk.
MetaBrowser, however, owns one opened root with one lifecycle.
Discovery, refresh, notification capture, change delivery, and close are not separate
products from its point of view.

Adding another session would create two owners, two cancellation paths, two stream
handoffs, and a naming collision in both Rust and Python.
The right abstraction is one `OpenedIndex` or `LiveIndex` handle whose optional
background driver moves through opening, discovering or reconciling, ready or watching,
and finally stopped.
Watch is a provider attached to that lifecycle, not the lifecycle itself.

#### 5. Admission and control-file state are distributed

Hidden and special-object admission must run in serial, parallel, macOS bulk,
reconciliation, and watch paths.
The repository now has
[`check-admission-sites.mjs`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/scripts/check-admission-sites.mjs),
which parses source text to ensure every known loop calls the expected function.
The check is useful protection for the current code, but needing it is evidence that the
invariant is not carried by the types.

Enumeration-time pruning must remain close to producers for performance: hidden subtrees
should not be statted and then discarded.
The semantic handoff can still be typed.
Producers should yield an admitted, scoped observation type that cannot be constructed
without the shared policy.
The index’s retention gate then handles the remaining semantic rules once, including
capacity and kind.

Pruned control directories have a second ownership problem.
The current vector is a monotonic history of directories ever observed, not the exact
current set of control files.
A control file is internal engine state even when it is excluded from user rows.
It should live in an exact, removal-aware control table keyed by canonical relative
path, be updated through the same commit path, and be persisted with the same bound used
on write and read.

#### 6. The page token solves a network problem inside an in-process engine

The current token carries a path, total row count, scalar totals, delivered count,
version, request-shape fingerprint, keyed tag, and checksum.
This grew because later pages must be proportional to a page while still reporting exact
whole-selection totals and an exact remainder.
It also created issuance bounds, platform encoding, key ownership, clone identity, and
token-authentication concerns.

Before preserving that design, confirm that MetaBrowser uses exact remaining rows and
whole-selection totals as product data rather than as a test invariant.
If it only needs lossless assembly, `next` versus terminal is enough.
If it displays a denominator, the first page may compute it and later pages can carry
it.

For the immediate in-process client, a bounded per-handle continuation table is simpler
than a signed stateless payload.
The external token can be a fixed-size cursor id; the table owns version, request shape,
last path, and any paid-for totals.
Eviction produces `VersionUnavailable`, and close clears the table.
A future network service can sign its own transport token at the actual trust boundary.
If stateless cursors remain a hard requirement, retained-path and token bounds must be
one enforced invariant and mutable index cloning must disappear.

#### 7. The engine knows client query names

The change layer currently names `Entry`, `Directory`, `FilteredTree`, `Rollup`,
`Navigation`, `Recent`, `Catalog`, and `Diagnostics` as dirty query kinds.
That is useful to MetaBrowser but makes the engine’s mutation layer change whenever a
client adds a projection.

A more stable core vocabulary is a small set of changed domains—topology, metadata,
classification, aggregates, and trust state—plus bounded affected paths.
The MetaBrowser adapter can map those facts to its closed query algebra.
The core may still provide high-performance native page operations where an adapter
cannot compose them without an unbounded walk or mirror.
What should not be in the core is the client’s cache-invalidation vocabulary merely
because it is the first client.

#### 8. Per-value provenance is ahead of the demonstrated need

The progressive plan makes provenance compose through every roll-up by weakest source,
oldest observation, and worst status.
Those reducers are not invertible.
Deletion and revalidation require recomputation, two index clocks cannot represent
arbitrary per-entry observation times, and the proposed `Complete | Partial` status does
not cover cancelled, failed, truncated, or refreshing states.
The open `fdu-livs` review bead already records this.

MetaBrowser’s implemented `InventoryEntry` does not carry per-row provenance.
Its immediate contract uses coherent global state and progress.
For the first integration, expose honest global lifecycle, coverage, freshness, source,
and progress, plus a simple per-directory completeness fact only if the UI consumes it.
A cold discovering tree is a lower bound.
A cache being revalidated is an estimate that may move either direction.
A watched tree can increase or decrease.
Those regimes must not share a blanket “monotonic” promise.

Per-value provenance can follow a real UI requirement and a measured storage/recompute
design. It should not block the first streaming handle.

#### 9. Optional dependencies are not optional by default for core consumers

`fdu-core` currently declares `default = ["watch", "gitignore"]` while its own comment
says library consumers should inherit neither.
A library user receives defaults unless they know to disable them.
The `ignore` feature alone is recorded as adding nine crates and about 1.06 MiB to a
stripped LTO binary.

The lean arrangement is:

- `fdu-core` has empty default features;
- the `fdu` binary explicitly enables the features the shipped CLI supports;
- the Python wheel explicitly enables the features its API exposes;
- a progressive core handle uses standard threads, locks, channels, and blocking pull;
  the Python adapter supplies the async shape without adding a Rust async runtime.

Whether the official CLI includes gitignore support by default is a packaging decision
that should be made from measured total size and user value.
It should not determine the dependency tree of an unrelated core embedder.

## Recommended target architecture

### One opened-root authority

The public interactive surface should be one cloneable handle, named distinctly from an
immutable index image and from MetaBrowser’s own `InventoryHandle`:

```text
CLI one-shot/progress                 Python / MetaBrowser adapter
          \                                  /
           \                                /
              OpenedIndex (one authority)
          read  changes  refresh  prioritize  close
                         |
              background session driver
              /                       \
      baseline discovery       optional watch capture
              \                       /
                   commit pipeline
                         |
                retained index + journal
```

The core API can be blocking and runtime-free:

```rust
let opened = OpenedIndex::start(root, config)?;
let result = opened.read(&request)?;
let batch = opened.changes(after, timeout)?;
opened.refresh(&hints)?;
opened.prioritize(&paths)?;
opened.close()?;
```

Python releases the GIL around those calls and exposes `async` adapters using the same
bounded handoff pattern already used for watch.
No Rust callback should cross into Python during scanning, and no async runtime should
become a core dependency.

The existing blocking `open()` remains as a convenience over the same mechanism: start,
wait until the requested settled boundary, return an immutable result.
The current CLI keeps using that path.
An explicit `--progress` mode reads intermediate boundaries from the handle; it does not
introduce a CLI-only producer.

### One exact commit path

Every answer-affecting change should become one value before it becomes visible:

```text
verified observation
        |
scope/admission decision
        |
prepared mutation with exact effective ops and state changes
        |
atomic apply under the single writer guard
        |
Commit { clock, ops, state, impact, stats }
        |
bounded journal -> watch/progress batches -> snapshots/read invalidation
```

No helper may mutate without contributing its exact effect to the commit.
Control-file changes, re-tagging, coverage loss, run-fact changes, and ordinary row
changes share the same clock.
A batch is derived from a journal range and the terminal state captured with that range.

The fastest way to make this tractable is to make observations explicit and ordered.
Walkers naturally observe parents before children.
Reconciliation can emit the same order.
A watch hint for a path whose ancestry is unknown should reconcile from the nearest
known ancestor and emit verified parent operations, not ask the index to invent them.

### A no-gap baseline-to-watch sequence

The opened-root driver should own the handoff:

1. Establish notification capture before publishing baseline progress.
2. Start or load the baseline and commit bounded batches to the retained index.
3. Buffer or coalesce notification hints with a hard bound while discovery runs.
4. If capture overflows, record an observation gap and reconcile a conservative scope.
5. Reconcile every queued hint after the baseline producer reaches its boundary.
6. Commit the settled state and expose its cursor.
7. Continue live delivery from that same journal and cursor.

There is no scan stream followed by an unrelated watch stream.
There is one commit sequence whose lifecycle phase changes.

### A deliberately small first read surface

The first MetaBrowser slice should support:

- a checkpoint read returning version, cursor, state, and progress in constant work;
- one bounded directory page with scalar subtree totals;
- one roll-up read;
- bounded invalidation batches and history reset;
- verified refresh;
- idempotent close.

Add native filtered/catalog pages when the adapter proves it cannot satisfy a real route
within the bounds. Add scheduling hints after cancellation and no-gap handoff work.
Add per-value provenance only after the browser uses it.
Add sorted resumable pages only after a consumer needs a sorted complete assembly rather
than a bounded ranked slice.

### Budget semantics should be a product decision

The file cap entered fdu because the Python reference provider has a 500,000-file
retention limit. That does not automatically make it a good fdu primitive.
A cap changes absence to unknown, makes the retained set history-dependent under live
changes, and complicates deletion, free-slot reuse, subtree reconciliation, and
conformance.

Before carrying it forward, decide one of these explicitly:

- MetaBrowser requires the same global regular-file cap from every provider.
  Keep the current global semantic rule, fingerprint it, and prove it with the
  two-engine replay.
- The cap is a Python-provider implementation limit.
  Remove it from the cross-provider contract and let fdu retain complete coverage by
  default.
- The product needs a real memory bound.
  Design a generic retained-entry or byte budget from measured memory rather than
  inheriting a Python file-count constant.

The current hybrid—treating the cap as a settled cross-engine semantic while the two
providers differ on subtree rewalk and free-slot behavior—is not mergeable.

## Current implementation findings

### Merge-blocking correctness findings

#### F1. `AppliedDelta` can assert a refused upsert and omit the mutations that happened

**Bead:** `fdu-a7cl`\
**Code:**
[`index.rs:2591`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/index.rs#L2591),
[`index.rs:4047`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/index.rs#L4047),
and
[`index.rs:4128`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/index.rs#L4128)

When a capped upsert creates missing ancestor directories or removes an existing row
whose kind changed, `apply_upsert` returns `true`. `apply_validated_with` then appends
the original leaf `Op::Upsert`. The file was refused and is absent; the new directories
or removed row are not in the delta.
A replaying consumer can create a phantom file and miss real changes.
This violates the engine’s defining effective-delta contract.

Fix the mutation model before fixing this case locally.
Tests must compare the complete effective operation sequence and replay it into an
independent model.

#### F2. Mutable index clones share cursor identity and authority

**Bead:** `fdu-91ru`\
**Code:**
[`index.rs:1302`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/index.rs#L1302),
[`index.rs:1559`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/index.rs#L1559),
and
[`index.rs:2047`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/index.rs#L2047)

Two cloned indexes can diverge at the same clock and validate each other’s signed
continuations. Remove live `Index` cloning and replace owned snapshots with a distinct
image type, or mint new session and continuation identities at every independent-owner
boundary. The former is simpler and aligns with one live authority.

#### F3. The engine can issue a continuation it refuses unchanged

**Bead:** `fdu-91ru`\
**Code:**
[`index.rs:1170`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/index.rs#L1170)
and
[`index.rs:1231`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/index.rs#L1231)

`EntryCursor::encode` has no issuance bound, while decode refuses more than 64 KiB of
hex. The repository accepts larger retained path encodings, and Windows UTF-16 followed
by hex expansion reaches the token limit earlier.
Use one enforced retained-path/token bound or the bounded per-handle cursor table
described above. Add exact-boundary round-trip tests on every platform representation.

#### F4. Deleting the last control file changes answers without a commit

**Bead:** `fdu-0778`\
**Code:**
[`index.rs:1704`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/index.rs#L1704)

`rebind_tag_rules` adopts newly bound rules and re-tags the index, then returns without
a commit when the new governed set is empty.
Deleting the last `.gitignore` can therefore change visible tags and promoted-plane
totals without moving the clock or notifying a consumer.
Determine whether the rule state changed, not whether the new set is nonempty, and
commit the exact affected scope.

#### F5. Pruned control-file history is unbounded and can produce an unreadable snapshot

**Bead:** `fdu-0778`\
**Code:**
[`index.rs:2348`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/index.rs#L2348)
and
[`snapshot.rs:197`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/src/snapshot.rs#L197)

Every reconciliation extends `pruned_control_dirs`; removals never remove entries.
A long-lived session can grow the vector without bound.
Snapshot loading caps the count at 1,000,000, but saving writes any representable count,
so the engine can write a snapshot it later refuses.
Replace the history with exact current control state and enforce the same bound before
persistence in both directions.

#### F6. Catalog validation still differs across the two providers

**Bead:** `fdu-8w5k`

The most recent review incorrectly claimed that Python treats `..foo` as extensionless;
both engines derive `.foo`. The remaining differences are real: fdu accepts duplicate
terminal-extension and ancestor-name values that MetaBrowser rejects, and on POSIX fdu
accepts a backslash-containing ancestor name that MetaBrowser rejects on every platform.
Decide the shared contract and add the answer-changing cases to the shared fixture.

#### F7. The remaining golden test narrows product output to selected booleans

**Bead:** `fdu-9tdm`\
**Review:**
[unresolved thread](https://github.com/jlevy/fdu/pull/47#discussion_r3858495113)\
**Code:**
[`cli-cost.tryscript.md:42`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/tests/golden/cli-cost.tryscript.md#L42)

`cli-cost.tryscript.md` invokes fdu inside inline Node programs, parses its complete
JSON and counter output, and prints selected relations.
Several content goldens similarly reduce product output to hand-picked fields.
That is a narrow assertion disguised as a golden and hides unanticipated changes.
Fixture-construction scripts are not the issue.
Move critical relations to focused integration tests and let goldens show broad, stable
product or diagnostic output directly.

### Proof and integration gaps

#### F8. There is no independent two-engine oracle

**Beads:** `fdu-kl7r`, `fdu-gy3g`, and the older `fdu-o8r8`

The reference example and local fixture test fdu’s interpretation of the contract.
They do not drive identical recorded observations through fdu and the Python provider
and compare results.
The missing replay must cover identity bytes, cap deletion and free-slot behavior,
subtree rewalk, special-object replacement, logical extension derivation, coverage, and
terminal clocks. Running two live walkers against a changing tree is not an oracle
because they observe different moments.

The File Rollup conformance packet is also not yet adequate: it lacks direct
basename-to-logical-extension cases and would pass before and after the change it is
supposed to prove.

#### F9. The transition model lacks a small independent state machine

**Bead:** `fdu-o8r8`

The engine has extensive examples but no dependency-free reference model that applies
long operation sequences and recomputes tree state and roll-ups from scratch.
The false cap delta is exactly the class of bug such a model catches.
Build this before expanding the progressive lifecycle.

### Maintainability and process findings

#### F10. Tracker status is not currently a reliable summary

The live graph is necessary but not sufficient.
`fdu-7sou`, `fdu-vfx7`, `fdu-xyvu`, and `fdu-vfyw` remain open even though their notes
say the named defects were fixed, accepted, or split into another bead.
The parent epic’s narrative still describes earlier counts and phases.
Before planning, reconcile those statuses and make each open bead describe only work
that remains.

This review itself is tracked as `fdu-8u4b`. The unresolved golden finding is now
tracked as `fdu-9tdm` so the GitHub thread is not the only record.

#### F11. Core feature defaults contradict the additive-dependency goal

**Code:**
[`fdu-core/Cargo.toml`](https://github.com/jlevy/fdu/blob/0558c7eff1b91a1dca052d4259dbe3751f6ffcd0/crates/fdu-core/Cargo.toml)

The core crate defaults to watch and gitignore support while documenting that core
embedders should inherit neither.
Make core defaults empty and let shipping surfaces opt in explicitly.
Re-run release-size and dependency-tree evidence before deciding the official CLI
package default.

#### F12. The central modules own too many invariants

`index.rs` currently owns arena storage, mutation arbitration, exact deltas, journal
retention, cursor authentication, paging, coherent read assembly, tag binding, control
state, lifecycle state, work accounting, and many tests.
`scan.rs` owns several producer backends, admission, budgeting, reconciliation, and
scheduling. Splitting files merely to reduce line counts would not help.
Split after the ownership model is fixed, along invariant boundaries such as `mutation`,
`journal`, `read`, `continuation`, and `control_state`.

## Outstanding work, ordered by decision value

### Gate 1: decide the contract before more code

- Decide whether `max_files` is a required shared semantic, a Python-provider limit, or
  a future measured memory budget.
- Decide whether exact `remaining_rows` and whole-selection totals are user-visible
  requirements.
- Decide whether initial MetaBrowser adoption needs per-value provenance or only global
  state and optional per-directory completeness.
- Decide whether core change batches expose client query names or stable changed
  domains.
- Revise PR #44’s spec before treating it as the base for implementation.

### Gate 2: repair the integrity kernel

- `fdu-a7cl`: exact effective commits under cap refusal and kind changes.
- `fdu-91ru`: one live owner and a total continuation contract.
- `fdu-0778`: exact, bounded, removal-aware control state and correct rebind commits.
- `fdu-o8r8`: independent transition reference model.
- `fdu-9tdm`: restore broad golden visibility and focused invariant tests.

No streaming lifecycle should be added until these are complete.
Streaming multiplies the number of consumers that trust the journal; it does not make a
false journal safer.

### Gate 3: prove a minimal opened-root lifecycle

- `fdu-e86o`: start discovery, read growing results, cancel deterministically.
- `fdu-a0j0`: expose the same handle in Python with the GIL released.
- `fdu-4o0m`: real mid-walk progress, async adapter, and no-gap baseline-to-watch cursor
  handoff.
- Rename or absorb the existing watch `Session`; do not expose two lifecycle types with
  overlapping names.
- Prove idempotent close and bounded slow-consumer recovery before scheduling hints.

### Gate 4: prove the actual client seam

- `fdu-kl7r`: recorded-observation replay through both engines and exact identity
  agreement.
- `fdu-8w5k`: resolve remaining shared validation semantics.
- `fdu-97dd` / `fdu-a7cl`: prove whichever budget contract Gate 1 chooses.
- `fdu-gy3g`: vendor a strengthened File Rollup packet only after it can distinguish the
  relevant semantics.
- Run MetaBrowser’s provider conformance suite against the adapter, not a parallel set
  of fdu-only examples.

### Defer until the vertical slice works

- `fdu-sgp7` prioritization; close belongs in the first lifecycle, prioritization does
  not.
- `fdu-m893` CLI progress rendering and `fdu-ey9q` progressive goldens.
- `fdu-t5h2` sorted resumable pages.
- full per-value provenance composition under `fdu-fka6` and related beads.
- further public instrumentation fields beyond what the performance harness consumes.
- additional promoted planes or taxonomy generalization until `fdu-n4gn` prices their
  default-off and enabled costs.

Lazy warm open (`fdu-1vd0` and related persisted-roll-up work) is a separate high-value
track. It is essential for an instant second open at multi-million-entry scale, but it
should compose with the handle after the live mutation kernel is sound rather than land
inside the same PR.

## Recommended delivery plan

### 1. Freeze PR #47 as the integration prototype

Do not add the progressive session, prioritization, sorted paging, or more query kinds
to this branch. Keep it available as a source of tested code and design evidence.

### 2. Amend and land the architecture decision separately

Revise the stacked PR #44 or replace it with a short decision document covering the four
Gate 1 questions, the one-owner model, and the exact commit pipeline.
The spec should describe the minimum MetaBrowser slice, not every foreseeable provider
capability.

### 3. Extract a core-integrity PR from `main`

Land only:

- one non-cloneable live owner, cloneable lightweight handles, and a distinct immutable
  image;
- exact mutation effects and replayable commits;
- exact control-file state;
- a simpler or total continuation implementation;
- the independent transition model and focused regressions.

This PR should not add an interactive surface.
Its success criterion is that every committed delta replays to the exact retained state.

### 4. Extract an opened-root vertical slice

Add the runtime-free `OpenedIndex` lifecycle with discovery, coherent checkpoint and
directory reads, bounded change pulls, verified refresh, no-gap observation handoff,
cancellation, and idempotent close.
Keep one-shot `open()` as a wrapper.

### 5. Add the Python adapter and run MetaBrowser conformance

Expose the same handle with GIL-free blocking methods and async Python wrappers.
Run the recorded two-engine packet and MetaBrowser’s provider contract.
Add only the native query operations that conformance and measured route cost prove
necessary.

### 6. Add CLI progress as an additive consumer

Once the core and Python consumer agree, add `--progress` using the same handle.
The ordinary CLI remains blocking and byte-compatible.
Progress output gets broad, deterministic goldens; monotonic cold-discovery invariants
live in focused tests.

### 7. Revisit optional capabilities with evidence

Prioritization, exact per-row provenance, retained budgets, sorted pages, more planes,
and lazy snapshot blocks each get a separate decision and measured acceptance criterion.

This sequence does not require preserving PR #47’s commit boundaries.
Cherry-pick or reimplement the smallest coherent pieces and preserve the tests that
assert valid contracts.
The integration branch can then close when its useful work has landed in reviewable
units.

## Merge criteria

The interactive work is ready to merge when all of the following are true:

- every mutation has one exact, replayable commit and the independent model agrees after
  generated operation sequences;
- there is one live owner and no independently mutable clone shares session authority;
- every issued continuation is consumable, bounded, and tied to one handle and question;
- control-file creation, change, and deletion update exact state and always clock
  answer-affecting rebinds;
- baseline discovery and watch share one lifecycle and one no-gap journal sequence;
- cancellation and concurrent close join background work deterministically;
- partial coverage, stale data, and unknown absence are distinguishable at every read;
- the two providers replay identical observations to identical semantic results and
  identity bytes for the shared contract;
- no unresolved GitHub review thread remains and open beads describe only current work;
- `fdu-core` retains an empty-feature build without watch, ignore, Python, or
  async-runtime dependencies;
- release binary and dependency growth are measured and accepted at the surface that
  opts in;
- `make check`, `make cross-lint`, the cross-engine conformance packet, and the
  MetaBrowser provider suite all pass at the exact candidate head.

## Final judgment

The core product direction is good: a retained hierarchical index with coherent reads
and a resumable commit journal is exactly what an interactive filesystem client needs,
and the CLI can remain an ordinary consumer of it.
The current implementation has demonstrated that many of the individual mechanisms can
work.

The present design is nevertheless more complex than it should be.
The branch tried to make the whole client contract native before establishing one owner,
one commit truth, and one lifecycle.
The resulting defects are concentrated evidence of that ordering mistake.
Correcting the order is more valuable than correcting the next six edge cases inside the
same shape.

Treat PR #47 as a successful prototype and an unsuccessful merge unit.
Keep its proven parts, simplify the contract to the immediate vertical slice, repair the
mutation kernel, and let one opened-root handle serve discovery, reads, refresh,
changes, and shutdown.
That architecture is flexible enough for MetaBrowser, additive for the CLI, runtime-free
for Rust embedders, and substantially easier to make correct.

## Delivery Decision After Review

The implementation will use one fresh fdu branch and one long-lived draft PR rather than
several separately merged fdu PRs.
The branch starts from current `main` and contains no PR #47 history.
This changes the delivery mechanics, not the architecture or ordering recommended above.

This is an explicit project-owner decision made after considering the report’s preferred
merge topology. The cumulative branch gives the two-repository effort one fdu head and
one MetaBrowser counterpart pin while the provider contract is changing, and avoids
stacked-PR base churn.
It also preserves the report’s review-size concern: checkpoints improve review timing
and bisectability but do not make the final merge unit smaller.
The final reviewer must assess the accumulated diff, and an independently understandable
or green phase is a stop condition that reopens this decision.

The single-PR choice reintroduces review-size risk, so four controls are part of the
decision:

- each phase lands as a distinct commit group with an exact green checkpoint;
- work does not advance while a phase acceptance gate is red;
- the PR remains a draft until the complete MetaBrowser integration phase passes;
- a second agent reviews the full accumulated PR, and every finding is tracked and
  addressed before it is marked ready.

Phase 1 is further divided into four named gates: observable oracle, exact commit truth,
control state, and live identity/feature floor.
Phase 3 begins with a disposable adapter against the unchanged MetaBrowser contract so
measured route cost precedes joint contract changes.

MetaBrowser is a separate repository, so its client-side contract changes stay on PR
#74. Both PR descriptions record the exact counterpart revision used by the integration
suite.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
