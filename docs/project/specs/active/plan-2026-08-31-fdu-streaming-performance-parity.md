# Feature: Streaming Performance Parity Without One-Shot Overhead

**Date:** 2026-08-31 (last updated 2026-09-01)

**Author:** fdu project, with Codex assistance

**Status:** In Review

## Overview

The opened-root work adds exact commits, progressive discovery, change polling, refresh,
and observation to fdu.
Those capabilities belong to long-lived consumers.
Existing one-shot commands still build a detached index and print one answer, but the
current branch makes them pay much of the long-lived mutation cost.

This plan restores one-shot performance to `main` while preserving the exact streaming
contract. It fixes two correctness defects first, then separates fact mutation from
optional commit consequences at one lifecycle boundary.
The implementation keeps one retained fact model and one public mutation contract.
Detached cold bootstrap may construct that ordinary index representation directly;
opened roots, refresh, observation, and later mutations continue through the exact
streaming reducer. It does not fork the engine into a fast CLI implementation and a
correct streaming implementation.

The work is delivered on `codex/streaming-performance-parity`, stacked directly on
[PR #51](https://github.com/jlevy/fdu/pull/51). PR #51 is stacked on
[PR #50](https://github.com/jlevy/fdu/pull/50), which is stacked on the
[opened-root rewrite, PR #48](https://github.com/jlevy/fdu/pull/48). The new pull
request uses `claude/one-shot-commit-cost` as its GitHub base so the stack remains
explicit and reviewable.

## Decision Summary

| Question | Decision |
| --- | --- |
| Does streaming inherently require the one-shot regression? | No. The regression comes from constructing consequences for consumers that cannot observe them. |
| Do we create a second fact engine for the CLI? | No. Detached bootstrap builds the ordinary `Index` directly with the shared admission, classification, and roll-up rules. Every subsequent mutation uses the existing reducer. |
| Where is optional work selected? | Once at the lifecycle entry point. Detached one-shot scans choose the directory-group builder; opened and public streams choose the causal reducer. There is no per-entry CLI check. |
| What does detached one-shot construction retain? | Facts, roll-ups, scope, issues, and provenance required by the returned index. It retains no exact change stream, impact set, journal entry, or compatibility delta. |
| What does an opened root retain? | Exact commits, bounded impact, lifecycle state, clock, and the bounded journal required by change consumers. |
| What does arbitrary public mutation retain? | Exact atomic validation and an exact commit outcome. It does not receive scanner-only trust. |
| How does the one-shot builder avoid the ancestry overlay? | Parent-first directory groups carry one parent path and component-only children. A transient path-to-identity map resolves each group once; public observations cannot construct this private input. |
| Is `AppliedDelta` kept? | No eager compatibility projection. As of the plan date there is no repository tag, GitHub release, crates.io package, PyPI project, or non-test repository consumer. Streaming consumers use exact `Commit` values. |
| How is parity decided? | Paired, interleaved release measurements against the pre-rewrite `main` control, with exact-result oracles and allocation counters. |
| Is wall-clock timing added to `make check`? | No. Deterministic allocation and zero-work guards enter the gate; timing remains in the controlled performance loop. |

## Goals

- Fix the cache-scope and path-canonicalization defects found in the PR #51 review
  before changing performance paths.
- Preserve exact commits, progressive opened-root discovery, refresh, change polling,
  journal overflow behavior, observation, and control-state semantics.
- Restore the default CLI and detached-index paths to performance parity with `main`.
- Make streaming-only work unreachable from ordinary one-shot CLI invocations.
- Reduce path allocation, reallocation, copying, and cloning to the minimum required by
  each consumer contract.
- Improve arbitrary public batch application where the same profiles identify avoidable
  work, without weakening atomic validation.
- Record enough counters and profiles to identify the next cost after every accepted
  change.
- Remove redundant compatibility and projection machinery when no released consumer or
  persisted format requires it.

## Non-Goals

- Removing the opened-root API or weakening its exact change semantics.
- Replacing the shared index, reducer, roll-up, or control-state model with a CLI-only
  engine.
- Changing default CLI output, query semantics, exit behavior, cache policy, or
  correctness.
- Making public callers responsible for asserting that their paths or ancestry are
  trusted.
- Completing the unrelated control-retention budget and MetaBrowser integration work
  tracked by the opened-root plan.
- Adding a noisy elapsed-time threshold to shared CI.
- Keeping an unreleased compatibility projection solely because tests currently mention
  it.
- Accepting a speed change whose mechanism, semantic oracle, or measurement regime is
  unknown.

## Background

### Stack and starting point

PR #51 removes several costs introduced by the opened-root rewrite: repeated commit
pipeline derivation, redundant canonical path rebuilding for walker paths, empty control
projection, and control observation for ordinary one-shot reports.
Those changes cut the PR-base whole-scan time by roughly half, but they do not restore
`main` performance.

The PR review compared the exact PR head with its declared base and with the pre-rewrite
control, `b75bf85`. The runs used the same 119,368-entry real tree.
The host was busy, so the timings are exploratory attribution evidence rather than
claim-grade results.

| Revision | Median wall time | Median engine component |
| --- | ---: | ---: |
| `main` control | 350 ms | 132 ms |
| PR #51 base | 1,699 ms | 1,478 ms |
| PR #51 head | 848 ms | 628 ms |

The PR head is about 2.4 times the control wall time and 4.7 times the control engine
component. A public 100,001-operation delta batch measured 433 ms at the PR head versus
56 ms on `main`.

The structured records separate three decisions:

- [exp-071](../../experiments/exp-071-pr-51-halves-its-base-regression-but-does-not-restore-main-p.md)
  accepts PR #51’s improvement over its base;
- [exp-073](../../experiments/exp-073-pr-51-remains-above-the-pre-rewrite-whole-scan-control.md)
  keeps parity with `main` blocked;
- [exp-072](../../experiments/exp-072-attribute-the-pr-51-residual-to-path-keyed-ancestry-prefligh.md)
  attributes the leading residual cost.

All three are marked exploratory because the host-pressure check refused a claim-grade
verdict.

### Measured cost ladder

Counter-guided disposable experiments removed one cost at a time from the PR head while
holding filesystem work, accepted operations, roll-up merges, and the final digest
constant.

| Experimental state | Allocation events |
| --- | ---: |
| `main` | 1,459,192 |
| PR #51 head | 2,716,389 |
| Do not materialize `AppliedDelta` | 2,604,110 |
| Do not publish detached commit consequences | 2,127,817 |
| Do not build the path-keyed ancestry overlay | 1,830,432 |
| Move owned prepared scanner paths | 1,710,339 |
| Do not copy paths into unused effects | 1,590,362 |

The profiles and timing ladder change the ordering stated in the opened-root plan:

1. `StructuralOverlay` ancestry preflight is the dominant CPU cost on both detached
   scanning and a large public batch.
2. Per-batch impact publication is the next visible whole-scan CPU cost.
3. Prepare, effect, and compatibility-projection path copies account for most of the
   remaining allocation gap, although filesystem latency hides part of their wall-time
   cost on the measured tree.
4. Oversized commits are cloned for journal retention before the capacity check drops
   them.

Removing detached publication and the ancestry overlay brought the exploratory
whole-scan result to 384 ms, close to the 351 ms control.
Moving scanner-owned paths and removing unused effect copies reduced allocation further
without a comparable CPU change on that subject.

### Correctness defects that precede optimization

**Snapshot scope projection.** `scope_serves` accepts a controls-on snapshot for a
controls-off request.
Public `CachePolicy::Auto` then reaches exact reconciliation with the stored scope and
returns `ScanScopeMismatch`. `CachePolicy::Only` returns the controls-on index,
including retained control state and ignored classification, as if it had the requested
controls-off scope.

Directional reuse is valid only when a report-only consumer projects the superset to the
weaker view before it escapes.
An `Index` returned to a public caller must have the requested scope exactly.

**Encoded path canonicalization.** `Path::components()` hides interior current-directory
components and repeated separators.
The new `canonical_relative_path` fast lane can therefore return the original bytes of
`a/./b` or `a//b` while claiming the path is canonical.
Path equality masks the defect, so the regression test must compare encoded path bytes
or platform-native units.

## Design

### Preserve one mutation kernel

The design separates two questions that the current pipeline combines:

1. Which verified facts and state transitions must the index apply?
2. Which consequences must this consumer retain after the mutation?

The first answer never varies by lifecycle.
The shared mutation kernel updates entries, roll-ups, controls, issues, provenance,
freshness, and `ApplyStats`. The second answer is chosen once for the batch:

```text
filesystem or public input
  -> provenance-specific preparation and validation
  -> shared fact and roll-up reducer
  -> lifecycle-selected consequence sink
  -> detached outcome, exact commit, or journaled commit
```

The selected sink cannot change fact semantics.
Tests replay the same accepted observations through detached and exact modes and compare
the complete index digest and stats.

The design principle that no observable mutation bypasses `Commit` remains intact.
Stats-only application is pre-publication baseline construction; opened-root and public
post-baseline mutations still publish exact commits through the same reducer.

### Consumer matrix

| Consumer | Batch provenance | Atomic ancestry preflight | Exact paths and impact | Journal |
| --- | --- | --- | --- | --- |
| Default CLI and blocking detached open | Trusted scanner preparation | Numeric or typed proof, no path-keyed overlay | No | No |
| Opened-root discovery | Trusted scanner preparation | Numeric or typed proof, no path-keyed overlay | Yes | Yes |
| Opened refresh and observation | Verified live preparation | Required where the batch can contain arbitrary topology | Yes | Yes |
| Public arbitrary `Index` mutation | Untrusted public preparation | Required | Yes | No unless explicitly requested |

The default CLI does not select a streaming mode merely because the binary was compiled
with watch or gitignore support.
The execution plan derives the lifecycle from the requested operation, as it already
does for retained serving indexes and control observation.

### Fix cache-scope reuse at the ownership boundary

Snapshot loading distinguishes two operations:

- **Return an index.** Require exact semantic scope.
  A controls-on snapshot does not directly satisfy a controls-off public open.
- **Consume a report projection.** A controls-on snapshot may supply controls-off facts
  only if the report path projects away control state and retags the consumed view
  before exact reconciliation or return.

The implementation requires exact scope for every public `Index`-returning path.
A controls-off `Auto` report scans cold when only a controls-on snapshot exists because
it must reconcile. A no-scan `Only` report may consume the stronger snapshot: every
report view reads the all-entry facts rather than control state or the ignored
partition, no `Index` escapes, and the returned report is retagged with the requested
controls-off scope. This explicit projection preserves cache-only reads after watch
sessions without weakening public index ownership.

### Separate public normalization from scanner preparation

Public observation paths always run the complete relative-path validator and normalizer.
Their tests cover `.` components, repeated separators, parent traversal, absolute paths,
empty paths, non-Unicode components, and platform separators.
The public boundary performs one component pass into a path buffer pre-sized from the
input’s encoded length.
It skips current-directory components, collapses repeated separators through component
iteration, and rejects parent, root, or platform-prefix components during that same
pass. This keeps the empty root path available where the engine contract uses it and
preserves non-Unicode native components without a temporary component vector.

The scanner does not send encoded path strings through that public API. It constructs a
private `ScannerBatch` from directory entries and already canonical parent paths.
The type owns its paths, has private fields, and can be created only by scanner code
that proves:

- every path is relative and canonical by construction;
- a parent entry already exists or is named by an earlier operation in the batch;
- no operation escapes the configured depth, filesystem, symlink, or admission scope;
- the batch records enough parent identity that application does not repeat a path-tree
  search to prove ancestry.

This is a safe internal proof, not an `unsafe` assertion or a public trust flag.

### Select consequence work once per batch

The implementation begins with the smallest internal abstraction that compiles the
detached branch away from the per-entry loop.
Two shapes are allowed for the first spike:

- a private zero-sized `NoConsequences` or `ExactConsequences` sink selected at the
  batch entry point; or
- two small batch entry points sharing a generic `apply_one` reducer.

A mode checked for every entry is rejected.
A duplicated reducer is rejected.
Choose the form with the smaller diff and equal generated behavior after profiling; do
not keep both abstractions.

Detached application records stats but does not allocate `MutationEffects`, copy paths
into `EffectiveChange`, derive `Impact`, advance a consumer-visible journal clock,
construct `Commit`, or construct `AppliedDelta`. Baseline establishment remains the
single transition from a building index to a usable detached index.

This stats-only mode applies only while constructing or reconciling a baseline before
the index is returned.
A public mutation submitted after a detached `Index` becomes observable still receives
the exact atomic validation and commit outcome in the consumer matrix above.

Exact application records only the consequences its public contract exposes.
State-only and fact-only changes still mint exact commits where a live consumer can
observe them.

### Replace path-keyed ancestry work for scanner batches

Arbitrary public batches need all-or-nothing validation before mutation.
They retain a transactional preflight, but the representation should use existing entry
identity and batch-local operation indices wherever possible instead of cloning full
`PathBuf` keys into a `BTreeMap`.

Scanner batches have a stronger contract.
Preparation resolves every parent to either an existing `EntryId` or an earlier batch
operation. Application consumes that proof and cannot discover an unknown parent midway
through mutation. This removes `StructuralOverlay` from detached and opened discovery
without creating a partial-apply failure path.

If the first proof representation is more complex than the overlay it replaces, stop and
measure a narrower representation: same-parent runs, a batch-local numeric overlay, or
resolved parent handles.
Complexity is part of the accept decision.

### Bound exact impact without rebuilding path sets

`Impact` is produced only for exact consumers.
Its domains are accumulated as flags.
Dirty paths remain bounded:

- stop collecting paths as soon as the public bound requires `all_dirty`;
- use entry or parent identity while deriving ancestors;
- materialize owned paths only for the bounded result that escapes;
- do not build a full `BTreeSet<PathBuf>` and discard it after overflow is known.

The independent commit model remains the oracle for topology, metadata, classification,
aggregates, trust, state, and dirty-path overflow.

### Remove eager compatibility projection

`AppliedDelta` is an entry-only projection of exact `Commit` changes.
Repository code outside tests does not consume it, and, as of 2026-08-31, the project
has no tag, GitHub release, crates.io package, or PyPI project.
The compatibility boundary is therefore **DO NOT MAINTAIN**. Tests and documentation
move to exact commits, and `ApplyOutcome` stops constructing a second owned path vector.

If release evidence found before implementation contradicts that boundary, retain the
source-level accessor as a lazy projection invoked by the caller.
Eager construction on baseline and opened internal paths remains forbidden.

### Avoid journal clones that cannot survive

Journal retention computes `retained_cost` before cloning.
A commit larger than the capacity advances the journal floor and clears retained history
without duplicating its paths first.

For retained commits, start with one clone only if profiles show it is not material.
Introduce shared `Arc<Commit>` storage only if retained-commit cloning remains a
measured cost. The plan does not add shared ownership as preventive complexity.

### Backward compatibility requirements

- **Internal code:** DO NOT MAINTAIN. Replace redundant internal projections and update
  all repository callers together.
- **Library APIs:** DO NOT MAINTAIN for `AppliedDelta`; no released version or named
  consumer exists. Preserve exact `Commit`, opened-root, blocking-open, and query
  functionality described by the current stack.
- **Server APIs:** N/A. fdu does not own a server API.
- **Plugin and extension APIs:** N/A.
- **File formats:** SUPPORT CURRENT SNAPSHOTS. Scope acceptance must never reinterpret a
  stored snapshot; no format change is required by this plan.
- **Persisted client state:** N/A beyond snapshots.
- **Database schemas:** N/A.

### Complexity budget

- No new crate, public feature flag, mutation engine, or CLI mode.
- One private batch-provenance boundary and one private consequence boundary.
- No per-entry lifecycle branch.
- No full-path clone unless the index or an escaping exact commit owns that path.
- No compatibility projection without a named consumer.
- No retained collection whose bound or overflow behavior is unstated.
- Delete superseded helpers and tests in the same change that removes their contract.
- Prefer resolved IDs, batch indices, flags, and bounded vectors over path-keyed ordered
  sets in hot paths.

## Performance Protocol

### Jobs

The implementation uses distinct jobs because they answer different questions:

| Job | Contract measured |
| --- | --- |
| `default-tree` | The command a user runs, including planning, detached scan, render, and persistence policy |
| `cold-scan-index` | Detached scanner and retained-index construction without render noise |
| `opened-discovery` | Progressive exact commits and journal retention during baseline discovery |
| `delta-apply-large` | Atomic exact application of one large arbitrary public batch |
| `delta-apply-batched` | Exact application and impact overflow across realistic repeated batches |

`opened-discovery` and the delta jobs need exact digest and commit-oracle checks before
their timings are accepted.
A generated fixture alone may screen a hypothesis; at least one nominated real tree
participates in an accept decision.

### Counters and profiles

Runtime counters distinguish work that should be zero from work that should become
cheaper:

- scanner, live, and public batch counts and accepted operations;
- ancestry preflight overlay inserts, path comparisons, and resolved-parent proofs;
- effect paths recorded and their owned bytes;
- impact candidates, ancestor visits, retained dirty paths, and `all_dirty` transitions;
- compatibility-projection materializations until exp-078 removes both the projection
  and its now-dead counter;
- journal retained, cloned, oversized, and dropped commits;
- allocation events, reallocations, allocated bytes, and frees scoped to the engine
  component.

Profiles use the existing runtime toggle and sampling or callgrind tiers described by
the instrumentation playbook.
Counter code remains off by default and must not add an allocation or lock to the
measured path.

Before each structural edit:

1. record the exact control and candidate commits, subject fingerprint, platform, host,
   cache state, and job;
2. capture counters and a profile that names the mechanism;
3. register the hypothesis and predicted reachable share of the gap;
4. run the semantic oracle before timing;
5. compare paired and interleaved release binaries;
6. record the result in the experiment ledger, including rejected experiments;
7. re-profile the accepted result before selecting the next change.

A busy host may produce exploratory attribution.
It cannot produce a parity verdict.

### Acceptance thresholds

The one-shot work is complete only when both a control-free real tree and a control-rich
real source tree meet all of these conditions against the pinned pre-rewrite control
`b75bf85`. A current-`main` comparison is also recorded before merge so later unrelated
changes cannot hide a regression:

- median `default-tree` and `cold-scan-index` wall and component ratios are at most
  1.03;
- the 95% interval’s upper bound is at most +5%, so parity is not a lucky median;
- allocation events, reallocations, and allocated bytes are each at most 1.05 times the
  control after fixed harness cost is removed;
- exact digest, report, exit, scope, and filesystem-state oracles agree;
- detached counters report zero effect paths, impact derivations, and journal clones;
  the compatibility projection is absent; and path-keyed ancestry-overlay inserts are
  zero.

Each individual tuning experiment still follows the project accept rule: at least a 3%
median improvement and a 95% interval entirely below zero.
Structural changes may be evaluated as one pre-registered composite when intermediate
conversion work disappears in the end state.

There is no pre-rewrite opened-root timing.
Its acceptance is semantic equality plus a strict resource bound: exact streaming adds
no duplicate compatibility path vector, no clone for an oversized journal entry, and no
unbounded impact set.
Public large-batch application must no longer spend most of its component time comparing
ancestry paths; the final target is set from the post-proof profile and recorded floor
rather than invented here.

## Implementation Plan

### Phase 1: Correctness and a trustworthy baseline

- [x] Add failing controls-on to controls-off snapshot tests for `CachePolicy::Auto` and
  `CachePolicy::Only`, covering both report-only consumption and public index return.
- [x] Require exact scope for returned indexes; keep or remove report-only directional
  reuse based on a measured, explicit projection.
- [x] Add encoded-byte path tests for current-directory components and repeated
  separators, plus existing escape, non-Unicode, and platform cases.
- [x] Restore complete public path normalization and introduce no scanner fast lane
  until its private invariant is tested.
- [x] Add the five performance jobs and consequence/provenance counters needed to
  distinguish detached, opened, and arbitrary public work.
- [x] Record fresh `main`, PR #51, and correctness-fixed baselines under the performance
  protocol.

On 2026-09-01, the four initial scope tests failed against the unrestricted directional
admission rule: `Auto` reached reconciliation and returned `ScanScopeMismatch`, while
the public `Only` API served the incompatible index.
Exact public admission fixed both.
The full gate then exposed the report-only constraint: watch sessions write controls-on
snapshots that ordinary CLI cache-only reports must still consume.
The final boundary keeps public ownership exact, uses cold fallback for scanning
reports, and limits the directional projection to no-scan reports.
A multi-view parity test compares that projection with a cold controls-off report.
The complete `make check` handoff gate passed after the projection fix.
The encoded-path regression then failed on the inherited fast lane: the exact commit
retained `dotted/./file.txt` bytes even though path equality matched `dotted/file.txt`.
The replacement canonicalizer validates and rebuilds in one pre-sized pass, and the
regression checks exact commit changes, compatibility operations, and dirty impact paths
in encoded form. The minimal and all-feature `fdu-core` suites pass after the fix.

The Phase 1 harness now has five exact jobs: `default-tree`, `cold-scan-index`,
`opened-discovery`, `delta-apply-large`, and `delta-apply-batched`. The two synthetic
delta jobs use an independent Python oracle for the final index and commit shape; opened
discovery validates its final index with an independent filesystem scan.
The pre-rewrite control supports the two one-shot jobs, while PR #51 and this branch
support all five. Three exploratory interleaved records establish the local baseline:

- [exp-074](../../experiments/exp-074-pr-51-residual-reproduced-on-the-current-registry-tree.md)
  reproduces the PR #51 regression on the current 11,142-entry Cargo registry tree:
  `default-tree` is 7.68% slower and `cold-scan-index` is 8.21% slower than the
  pre-rewrite control by median wall time.
- [exp-075](../../experiments/exp-075-scoped-counters-stay-below-the-exploratory-acceptance-thresh.md)
  screens the runtime-gated instrumentation itself.
  Three uncontrolled pairs move in opposite directions across the two jobs, with the
  slower result below the 3% experiment threshold; this is a bound for attribution, not
  a timing claim.
- [exp-076](../../experiments/exp-076-correctness-fixes-preserve-the-streaming-performance-baselin.md)
  compares instrumented PR #51 with the correctness-fixed branch.
  Every semantic oracle passes, and the four-pair intervals for the five jobs cross
  zero, so Phase 2 treats PR #51 and the correctness-fixed branch as the same
  performance baseline.

Scoped counters identify work by lifecycle rather than by call site.
A detached cold scan of 11,141 observed entries builds 11,141 effective paths, visits
66,903 impact ancestors, retains 15,914 dirty paths, and materializes 338 compatibility
deltas even though it retains no journal commit.
One exact 100,001-operation public commit builds the same number of effective and impact
paths, then clones the commit before the journal rejects it as oversized.
The equivalent 4,096-operation batching case retains 25 commits and drops nine older
ones, so it remains the control for bounded exact history rather than sharing the
oversized shortcut.

Sampling profiles confirm that these counts name material CPU mechanisms.
On `cold-scan-index`, path iteration and comparison account for 7.67% of all samples and
allocator work for another 6.21%; the path share alone is the size of the local
wall-time gap. On `delta-apply-large`, the path layer is 45.47% of samples, led by
component iteration and comparison, while `opened-discovery` spends 16.09% in the path
layer and 7.23% in the allocator.
Probe-side digesting is reported separately and is outside the component timer; profiles
are attribution evidence only, never timing verdicts.

Phase 1 passes when the focused tests fail on PR #51, pass on the branch, all existing
engine and opened-root model tests pass, and the baseline evidence can assign every
large cost to a named component.

### Phase 2: Lifecycle specialization and ownership

- [x] Add the private consequence sink and make detached application stats-only without
  duplicating the reducer.
- [x] Remove eager `AppliedDelta` construction and migrate exact consumers to `Commit`.
- [x] Add the private owned scanner batch with resolved-parent proof; remove the
  path-keyed `StructuralOverlay` from detached and opened discovery.
- [x] Move prepared scanner paths across boundaries once and stop recording effect paths
  when the selected sink cannot expose them.
- [x] Bound impact accumulation and avoid cloning journal entries that exceed capacity.
- [x] Run and record one experiment after each independently measurable change; reject
  any abstraction that does not improve the named job or simplify a proven boundary.

[exp-077](../../experiments/exp-077-select-detached-consequences-once-per-batch.md)
accepts one private, batch-selected consequence sink.
Detached scoped allocations fell 33.7% and allocated bytes fell 24.5%; `default-tree`
wall and component time improved 6.57% and 7.24%. The detached path now constructs no
effective-change path, impact, commit, or journal state, while the exact jobs preserve
their independent engine and commit digests.

[exp-078](../../experiments/exp-078-remove-the-eager-compatibility-projection.md)
accepts removing the unreleased `AppliedDelta` projection.
A 100,001-operation exact batch eliminates 100,002 scoped allocations, improves wall
time 1.55% with a paired 95% interval entirely below zero, improves component time
2.38%, and reduces peak RSS 7.09%. The generic harness classifies the sub-3% wall result
as too small for added complexity; the operator accepts it because the candidate deletes
128 net lines and a duplicate owned path vector.
Python `since()` preserves its existing entry-operation output by projecting exact
changes only when crossing the language boundary.

[exp-079](../../experiments/exp-079-resolve-scanner-parents-before-mutation.md) accepts
a private owned scanner batch and resolved-parent proof.
On the stable 11,141-entry corpus, opened-discovery wall time improved 9.50%, with a
paired 95% interval from -10.89% to -8.14%, while scoped allocations fell from 489,514
to 405,751. Cold scoped allocations fell from 162,071 to 109,568; default and cold wall
time were noninferior to the immediate control.
The scanner paths record zero ancestry-overlay insertions, while public, refresh, and
watch observations retain the general atomic preflight.
Exact engine and commit digests remain unchanged.

[exp-080](../../experiments/exp-080-skip-oversized-journal-clones.md) moves the existing
journal-capacity decision ahead of commit cloning.
The 100,001-operation job eliminates 100,003 scoped allocations and improves wall time
3.46%, with a paired 95% interval from -4.32% to -2.61%; component time improves 4.71%.
The batched retained-history case also improves, while opened discovery is unchanged.
The journal still retains every commit that fits, but no longer copies a commit it must
reject immediately.

Two follow-up ownership reductions were rejected.
[exp-081](../../experiments/exp-081-borrow-impact-paths-until-the-bounded-result-escapes.md)
removed 8.2% of opened scoped allocations but produced no supported wall-time gain and
regressed the large exact-update component.
[exp-082](../../experiments/exp-082-move-scanner-commits-directly-into-the-journal.md)
removed nearly every scanner journal clone and roughly 10.3% of opened scoped
allocations, but opened wall time was unchanged and the candidate added a second result
form. Both spikes were removed.

[exp-083](../../experiments/exp-083-skip-unignored-roll-up-maintenance-in-control-free-scopes.md)
tested the remaining whole-scan allocation gap on the 113,794-entry control-rich source
checkout. Skipping the redundant `unignored` reducer when controls are disabled removed
114,782 component allocations and 36.8 MB of requested allocation per scan;
repeat-profile allocation, reallocation, and byte totals all came within 1.02 times the
pre-rewrite control.
The wall result did not clear the experiment rule: `default-tree` improved 1.61%, while
`cold-scan-index` changed 0.47% with its interval crossing zero.
The spike was removed.

The larger subject also changes the campaign conclusion drawn from the first tree.
Before exp-083, the complete branch was 5.39% slower on `default-tree` and 4.79% slower
on `cold-scan-index`; with the rejected spike applied, a direct diagnostic against the
pinned pre-rewrite control still measured regressions of 4.92% and 4.01%. The post-spike
counter profile leaves roughly 0.22 extra allocation and reallocation events and 90
requested bytes per entry.
That evidence motivated
[exp-084](../../experiments/exp-084-compact-optional-fixed-partition-storage.md), a
pre-registered composite of the control-free lane and compact optional directory-only
storage for the second reducer.
The spike removed another exact 56 requested bytes per entry, brought whole-process
allocation, reallocation, and byte ratios within 1.02 of the pre-rewrite control, and
cut `default-tree` peak RSS 18.16%. Its `default-tree` wall improvement was 2.63%, with
a paired 95% interval from -3.17% to -1.19%, while `cold-scan-index` remained
inconclusive. That misses the 3% structural-change gate, so the 260-line optional-state
and materialization design was removed.

Allocation-stack traces after exp-084 find no remaining duplicate clone: retained
allocations are the entry arena, names, extension interning, child maps, and roll-up
maps also owned by the pre-rewrite engine.
The remaining actionable difference is architectural rather than an isolated heap event:
one-shot scans now cross the streaming producer/preparation boundary even when the
caller requests neither controls nor consequences.
Any next experiment must combine a compact control-free representation with one
scan-level choice of a direct non-streaming producer/reducer lane; neither rejected
representation alone may be smuggled into the branch.
The composite must still clear the 3% wall gate, preserve exact digests, meet the 1.05
allocation ratios, and prove opened-discovery non-regression.

H99 (`fdu-hjez`) was that pre-registered composite.
The completed-index heap trace identifies the boundary representation precisely: private
scanner batches retain `Vec<ObservationOp>` even though every scanner operation has
`Expectation::Any`. At the exp-084 high-water mark, those batch buffers retained 25.6
MB; the pre-rewrite walker transported compact `Vec<Op>` values.
It kept scanner batches compact, converted to conditional-capable observations only when
the public streaming `scan` API crossed its boundary, and combined that change with
exp-084’s compact fixed-partition storage.

[exp-085](../../experiments/exp-085-compact-scanner-batches-and-optional-fixed-partitions.md)
rejects the complete composite under its preregistered gate.
Compact scanner transport alone was neutral on `default-tree` at +0.11%, with a 95%
interval from -0.88% to +1.46%, and directionally improved `cold-scan-index` 1.24%, with
its interval crossing zero.
The composite improved `default-tree` 2.56%, with a paired interval from -3.33% to
-0.13%, but stayed below the 3% structural threshold.
`cold-scan-index` improved 1.63% by median, with an interval from -2.74% to +0.34%. Peak
RSS fell 19.11%, confirming the representation benefit, but the complete 490-line spike
was removed because its wall-time gate failed.

The two rejected partition experiments and the neutral scanner-only diagnostic narrow
the next admissible target.
Another retained-shape edit is unlikely to supply a material wall gain.
The remaining untested architectural option is a true one-shot producer/reducer lane
selected once by the execution plan: it must consume scanner facts without retaining
batch or public-observation storage, while the streaming and opened APIs continue to use
the current batch contract.
That lane requires a fresh profile and preregistration before implementation.

The fresh H103 sampling profile assigns 48.63% of self time to kernel calls, 7.09% to
the allocator, and only 0.99% directly to index symbols.
The scanner-only producer component is already statistically indistinguishable from the
pre-rewrite control, and an adaptive diagnostic confirms that both revisions use the
same six-worker policy and APFS bulk backend.
Sampling alone therefore cannot justify a second reducer: most index work is inlined
into callers and appears only as inclusive time.

[exp-086](../../experiments/exp-086-scanner-phase-counters-expose-preparation-without-observer-c.md)
adds three off-by-default elapsed counters to close that visibility gap.
With `FDU_COUNTERS=1`, ten scans spend 285,370 microseconds preparing scanner batches,
822,869 microseconds reducing them, and 7 microseconds projecting the empty control
table: 28.5 ms, 82.3 ms, and less than 0.001 ms per scan, respectively.
The disabled-instrument screen is neutral on `default-tree` at -0.12%, with a paired 95%
interval from -3.06% to +2.40%. Preparation is therefore a plausible target; control
projection is not.

H104 applies that evidence to the optional-path request.
Only a detached, control-free index still under construction may consume a trusted
scanner batch in one pass: it resolves each parent and mutates immediately, so a failed
internal invariant discards the owned index instead of requiring batch-atomic rollback.
Opened discovery, public `scan`, refresh, observation, and arbitrary public mutation
retain the current prepared and conditional paths.
The experiment combines that fused lane with exp-084’s compact control-free partition
representation because the campaign’s final allocation and byte ratios cannot pass
without it; neither rejected change may land by itself.

The composite is accepted only if `default-tree` improves at least 3% versus the
instrumented `c7b2120` control with the paired interval below zero, `cold-scan-index`
moves in the same direction, whole-process allocation, reallocation, and requested-byte
ratios versus `b75bf85` are at most 1.05, and public scan plus opened discovery preserve
their exact semantic and resource gates.
The phase counters must show that preparation was eliminated rather than shifted, and
the complete composite is removed if any gate fails.

[exp-087](../../experiments/exp-087-fuse-detached-control-free-scanner-preparation-and-reduction.md)
rejects H104. The fused-only diagnostic was neutral, and the full composite improved
`default-tree` 1.11%, with a paired interval from -2.47% to +0.40%. `cold-scan-index`
changed -0.20%, with an interval from -1.19% to +2.12%. Although repeat-10 counters
reduced preparation to zero and restored allocation, reallocation, and requested-byte
ratios versus `b75bf85` to 1.016, 1.020, and 1.005, the removed consumer work overlapped
producer I/O and did not shorten the critical path.
The complete 460-line composite was removed.

H105 targets a simpler batching regression revealed by the same counters.
The parent-before-child correctness fix publishes a causal scanner fragment before its
new directories become claimable.
On the 113,794-entry subject, that produces about 2,650 baseline applies per scan; the
configured 1,024-operation target would require roughly 112 full batches.
The public and opened streaming paths need the causal fragments, but a one-shot builder
does not need to reduce each fragment separately.

The experiment concatenates adjacent causal fragments only inside `scan_into_index` and
its diagnostic twin, up to the existing configured batch target, before passing them to
the unchanged atomic scanner reducer.
Channel order already proves every concatenated parent precedes its child, and the
reducer already supports parents created earlier in the same batch.
Public `scan`, opened discovery, refresh, and watch retain their current publication
cadence. The candidate is accepted only if `default-tree` improves at least 3% with the
paired interval below zero, `cold-scan-index` moves in the same direction, the exact
digest and report stay unchanged, and baseline applies fall within 10% of the
configured-batch minimum.
Otherwise the coalescer is removed.

[exp-088](../../experiments/exp-088-coalesce-causal-scanner-fragments-in-the-one-shot-builder.md)
rejects H105. The coalescer reduced baseline applications from about 2,670 to about 124
per scan, but `default-tree` changed +0.13%, with a paired 95% interval from -1.08% to
+2.29%, and `cold-scan-index` was flat.
Preparation rose from about 31.7 ms to 105.7 ms per scan because the earlier-parent
proof reverse-scans a larger prepared batch.
The candidate was removed.
Together with producer-only parity, this result rules out causal publication frequency
and reducer-call count as the primary remaining wall-time cost.

The next experiment must first attribute per-entry baseline mutation costs that exist
after the rewrite but not in the pre-rewrite control.
In particular, initial revision bookkeeping is a candidate only if counters show it runs
at corpus scale and a differential profile or bounded diagnostic can account for a
material share of the remaining gap.
Streaming and arbitrary public mutation must retain revision semantics; any one-shot
specialization must leave the completed baseline in a valid initial revision state.

The H106 source comparison rejects revision bookkeeping as that candidate: both the
pre-rewrite and current engines increment the same parent `children_revision` on each
insert.
Matched eight-second sampling runs with counters disabled instead put the current
main-thread consumer in `scan_into_index` for 1,182 samples, versus 822 for the
pre-rewrite control.
Scanner preparation accounts for 345 current samples, but H104 already showed that
eliminating this consumer work alone does not shorten the critical path.
The differential therefore points to the interaction between consumer work and the
parent-before-child publication barrier, not to a new per-entry revision operation.

H106 first tests that interaction without designing a second reducer.
A producer-only diagnostic suppresses the early fragment flush before discovered
directories become claimable, while retaining the configured batch limit.
Its order-insensitive compact summary must still match a separate exact causal
validation scan; no index is built from the unordered diagnostic stream.
The diagnostic earns a correctness-preserving one-shot builder experiment only if
`cold-scan-producer` component time improves at least 3% with the paired 95% interval
below zero and publication batches fall toward the configured-batch minimum.
Otherwise the spike is removed and unordered publication is ruled out as a material
mechanism. Any follow-up builder must remain private to a detached control-free scan,
buffer children until their real parent arrives, preserve exact digests and reports,
keep every public and opened stream causal, and independently clear the normal 3%
`default-tree` gate.

[exp-089](../../experiments/exp-089-suppress-causal-publication-in-a-producer-only-scan.md)
rejects H106. Suppressing the causal early flush changed `cold-scan-producer` component
time +0.68%, with a paired 95% interval from -1.04% to +2.13%; whole-process wall
changed +0.38%, with an interval from -0.12% to +0.96%. Every unordered compact summary
matched its separate exact causal validation scan.
The spike was removed, and the pending-child one-shot reducer it was meant to justify
will not be built. H104 through H106 now rule out scanner preparation, reducer-call
frequency, and causal publication as separate 3%-class mechanisms.

The counter-disabled differential also exposes an instrumentation defect: the profile
command forces counters on, and H103’s per-batch elapsed timers add work to the very
consumer profile used to select the next experiment.
Before the campaign attempts a larger structural representation change, profiling must
support an explicitly labelled counter-disabled mode while timing and oracle validation
remain unchanged. The raw sampling workaround used for H106 is evidence for that
requirement, not a replacement for a reproducible harness path.

The counter/oracle instrumentation prerequisite is now complete.
The profiler accepts independent `--counters` and `--oracle` switches, the probe’s
`--no-oracle` mode summarizes a completed index from its stored root roll-up without a
path or digest walk, and every profile artifact records both choices.
Timing runs reject an explicitly disabled oracle, while old immutable controls without
the new label remain valid.
The default enabled mode still produced the exact subject digest and engine-scoped
component counters.

The first supported clean capture sampled 23,663 stacks over the same 113,794-entry
subject with both switches disabled.
The probe/oracle layer fell to 0.06% of samples; the two leading symbols were `__open`
at 34.21% and `getattrlistbulk` at 33.91%, followed by semaphore waiting at 6.91%.
Allocator symbols accounted for 5.84% in aggregate, while the remaining scanner and
index work was dispersed across worker, entry-recording, path, collection, and roll-up
frames. This honest profile identifies no independent leaf optimization with enough
reachable share to close the residual gap.

The FullIndex diagnostics prerequisite is also complete.
Cold FullIndex scans now retain the same opt-in `fdu-scan-diagnostics-v1` trace already
available on the transient Summary plan, while the ordinary entry point still selects
the no-diagnostics scanner.
Exp-090 interleaved diagnostics off and on in one immutable binary over this subject:
wall changed -3.48%, with a paired 95% interval from -11.88% to +1.43%, exact tallies,
and every resource gate held.
Cache-only opens perform no scan, and warm reconciliation does not use the cold-scan
producer contract; neither emits a misleading partial trace.

H104 through H106 already removed scanner preparation, application frequency, and causal
publication independently without moving wall time.
The next admissible experiment is therefore the existing H86 structural bootstrap
representation (`fdu-xde5`), not another boundary micro-tuning pass.
For a detached one-shot build, it combines batch-shaped parent/name records, compact
arena storage, directory child slices, and one bottom-up roll-up; opened discovery,
refresh, observation, and arbitrary public mutation retain the exact streaming reducer
and commit path. H86 is evaluated as one preregistered structural decision because
partial forms retain conversion costs the completed representation removes.
Before implementation, its bimodal arena ceiling must name the comparison mode, and its
accept set must include this real subject, byte-identical engine digests across worker
counts, opened-path non-regression, the current 3% wall gate, and the parity allocation
limits.

That preregistration is now fixed in
[the campaign-2 H86 section](plan-2026-08-23-fdu-performance-campaign-2.md#h86-preregistration-one-decision-two-evidence-stages).
The initial preregistration limited the optimized route to one private,
controls-disabled, detached cold-bootstrap choice.
It proposed retaining a complete compact index and promoting once to the ordinary
mutable layout if a later public commit required mutation; it was not a report-only
approximation. All causal and exact streaming producers remained on the existing
scanner-batch reducer, and the initial checkpoint fell closed to that path for
controls-enabled or otherwise unproved requests.
The separately preregistered controls checkpoint below later proved the same private
directory-group boundary for controls-enabled one-shot scans without changing those
public producers.

The first implementation checkpoint validates the lifecycle split but is not the H86
verdict. A naive detached builder retained every directory group until worker join and
then constructed the index.
It removed per-file paths and reduced roll-up merges, but also serialized work that the
current producer and consumer overlap; steady exploratory `cold-scan-index` samples
moved from roughly 270 ms to 410 ms.
That form is rejected.

The replacement publishes one parent path with component-only children before making
those child directories claimable.
A private builder consumes the groups while the same generic filesystem walker
continues. Direct file and directory-self contributions fold during the walk; the final
reverse pass visits directories only and borrows their retained roll-ups instead of
cloning them. At this checkpoint, controls-enabled scans and all public or opened
streaming paths still use the scanner-batch reducer.

On the 113,794-entry MetaBrowser subject, the latest twelve-pair uncontrolled
controls-disabled exploratory run changed `cold-scan-index` wall time -0.31% by median,
with a paired 95% interval from -7.17% to +1.22%, against the immediate `c6380f7`
control. This is practical timing parity, not claim-grade acceptance.
Moving each incoming name into its entry instead of cloning it twice and retiring
directory lookup keys as soon as their listings arrived removed another allocation per
entry. Scoped counters now show 923,671 allocations against 1,107,018, 101,952
reallocations against 212,083, 164,601,289 allocated bytes against 217,146,323, and
129,013 roll-up merges against 1,217,448. Peak RSS moved -2.65% in the exploratory pair.
The compact retained entry layout, single name arena, sorted child slices, promotion
boundary, `default-tree` gate, opened-path non-regression, historical controls, quiet
Darwin verdict, and Linux stage remain open.
The raw artifacts are under `/tmp/fdu-streaming-parity/results/`; temporary absolute
paths are evidence locations, not durable repository references.

The next checkpoint extends the private builder to a controls-enabled one-shot scan;
this extension is preregistered before its implementation.
A fresh controls-rich run on the same subject retained only 51 control sources but
crossed 2,654 scanner batches, 6,024,294 scoped allocations, 601,749 reallocations, and
491,242,604 allocated bytes.
Control projection alone recorded 399,849 microseconds.
The repeated projection and subtree reclassification, rather than reading the 51 small
files, is therefore the named mechanism.

Each detached directory group will carry its optional verified control operation.
The consumer applies that directory’s complete control state before inserting any
sibling; the existing parent-before-child publication barrier then guarantees that every
descendant sees all governing controls.
Each entry is classified once from its retained parent and the complete table.
Controls-disabled work still pays no control path, while opened discovery, refresh,
observation, and arbitrary public mutation remain on the causal scanner reducer.

The checkpoint must match the exact scanner reducer at explicit worker counts one
through four, including retained facts, control identities, ignored classifications, all
and unignored roll-ups, report completeness and errors, scope, freshness, state, clock,
and the first public mutation after bootstrap.
Fixtures cover root and nested rules, negation, an ignored parent, a non-file control
path, source and pattern limits, and the capability-disabled build.
Before it can be kept, a twelve-pair controls-rich `cold-scan-index` comparison against
`c6380f7` must improve wall and component time by at least 3% with both paired intervals
below zero; the predeclared reachable-share prediction is at least 25%. Scanner batches
and scanner control-projection time must be zero on the private route, the detached
build count must be one, exact/public routes must record zero detached builds, and no
resource metric may cross the existing H86 limits.
This checkpoint does not relax the final historical-parity, default-command,
compact-layout, RSS, quiet-host, or Linux gates.

The controls-aware checkpoint passes its predeclared exploratory screen.
Across twelve paired uncontrolled trials, controls-rich `cold-scan-index` wall time fell
33.55%, with a paired 95% interval from -36.41% to -33.14%; component time fell 47.43%.
Scoped allocations fell from 6,024,294 to 987,134, reallocations from 601,749 to
104,398, and allocated bytes from 491,242,604 to 133,101,059. Scanner projection,
preparation, and reduction counters were zero, the private detached build count was one,
and the exact tree digest matched.
The fixture matrix and both all-feature and capability-disabled scanner suites pass.

The differential fixture also exposed a correctness defect in the specialized scanner
oracle: a non-file `.gitignore` correctly produces the documented `ControlRemove`, but
baseline preparation rejected that operation even though it accepted a control upsert at
the same fixed path.
Preparation now validates and accepts the fixed-path removal as control-only work, while
an invalid removal path still fails.
The detached builder and the scanner oracle agree on the non-file source, control
limits, ignored state, reports, and the first exact public mutation at worker counts one
through four.

The final complexity pass also shares worker-pool setup, adaptive scaling, termination,
diagnostics, panic handling, joins, and error ordering between streaming and detached
walks.
[Exp-098](../../experiments/exp-098-share-pool-orchestration-through-a-dynamic-consumer.md)
rejects the first trait-object consumer after whole-process wall regressed 0.82%, with a
paired interval from +0.45% to +3.00%, even though scan component time was unchanged.
[Exp-099](../../experiments/exp-099-monomorphize-shared-concurrent-walk-consumption.md)
keeps the shared source with a generic consumer: wall changed +0.16%, with an interval
from -1.46% to +0.81%, and component time changed +0.16%, with an interval from -1.56%
to +1.06%. The accepted form is a complexity decision under the +3% noninferiority
margin, not a speed claim.

A fresh exploratory lifecycle comparison separates construction, save/join,
serialization, and the default report.
Against the preserved pre-rewrite binary, `cold-scan-index` wall changed +0.93%, with a
paired 95% interval from -5.63% to +3.83%; component time changed -0.39%, with an
interval from -3.02% to +4.04%. `cold-open-save` wall changed +1.11%, with an interval
from -0.53% to +3.40%. Those medians are practical parity, while their intervals
narrowly miss the final +3% noninferiority bound.
A clean sampling profile attributes about 69% of both `cold-scan-index` and
`default-tree` samples to `open` and `getattrlistbulk`, about 6% to allocator symbols,
and less than 0.5% to index symbols; it identifies no independent CPU hotspot capable of
moving wall by 3%.

The run began under an uncontrolled load and ended at 100% host CPU, so its
`default-tree` interval is too wide for a verdict: wall changed -0.84%, with an interval
from -19.32% to +10.08%. The isolated snapshot-save component also remains inconclusive;
its apparent +16.58% change spans -20.43% to +32.37%, while that job’s wall includes an
untimed setup scan that crossed the same load change.
Peak RSS remains materially higher than the historical control: +21.75% for
`cold-scan-index`, +19.43% for `cold-open-save`, and +16.75% for `default-tree`. The
retained-layout attribution now explains most of that gap.
A full allocation-stack high-water capture found 113,793 live allocations of exactly 280
bytes, or 31,862,040 bytes, matching one boxed current `Entry` for every nonroot entry.
The current entry is 56 bytes wider than the historical entry, which accounts for
6,372,408 bytes on this tree.
The same capture found two equal groups of 12,846 320-byte allocations, or 4,110,720
bytes each, consistent with the `all` and `unignored` extension-map node planes.
The release binary’s stripped frames prevent assigning those two groups to Rust symbols,
so that map attribution remains size-and-cardinality evidence rather than a symbolized
proof. One wider roll-up plane plus one additional map-node plane accounts for 10.48 MB
of the observed 10 to 13.8 MB candidate-control difference.

Snapshot and report handoff do not amplify the regression.
`cold-open-save` adds 11.7 MB over the control’s cold-scan peak and 12.5 MB over the
candidate’s; the candidate-control difference remains 13.8 MB. The `default-tree`
difference narrows to 10.1 MB. In clean repeated profiles, all allocator symbols account
for 5.63% of `cold-scan-index` samples and 5.90% of `default-tree` samples, while direct
free symbols account for 1.00% and 1.39%, respectively; no destructor-specific Rust
frame is a hotspot. The retained entry and roll-up layout, not serialization, report
construction, or teardown, is therefore the next memory boundary.

This evidence does not relax H86’s wall-time gate.
At this checkpoint, the preregistration treated the complete retained-layout decision as
removing the per-entry box, storing names once, moving directory-only state off file
entries, and avoiding a second extension-map plane where the partitions are identical.
Repeating the rejected optional-roll-up change alone remains inadmissible.
The current candidate has practical historical wall parity and direct index symbols
remain below 0.5% of clean samples, so a packed representation earns a place in this
campaign only if the complete form also clears the preregistered `default-tree` wall
target and exact lifecycle oracles.
Quiet-host noninferiority and the H86 RSS gate remain open; this checkpoint is not the
final H86 verdict.

The measured sequence separated the retained-layout mechanisms before adding another
conditional roll-up representation.
The first arm moved directory-only child, roll-up, revision, and completion state behind
one directory allocation and stored arena entries inline instead of boxing every entry.
Across twelve paired uncontrolled trials against the exact `88304cb` control,
`cold-scan-index` wall changed -3.18%, with an interval from -6.81% to +0.61%, while
`default-tree` changed -0.83%, with an interval from -3.33% to +0.74%. Peak RSS fell
29.00% and 24.63%, respectively, but the arm failed the preregistered `default-tree`
wall gate and is rejected as a standalone layout.

The second arm keeps the same `Index` and public mutation contract while storing a
detached directory’s children as name-sorted entry identifiers.
The entry owns the only retained name; lookup uses binary search, ordered iteration
borrows the name from the child, and the first arbitrary mutation promotes only the
touched parent to the existing keyed map.
Opened discovery and ordinary public indexes start with the keyed representation, so the
streaming reducer does not acquire a second topology or a global promotion pass.
The exact ordering and first-mutation fixture passes, as do the focused index, detached
scanner, and opened-engine suites.

The composite clears the local structural screen in exploratory evidence.
`cold-scan-index` wall fell 5.87%, with a paired 95% interval from -15.86% to -3.16%,
and peak RSS fell 45.03%. `default-tree` wall fell 7.70%, with an interval from -10.16%
to -3.77%; its measured component fell 7.71%, and peak RSS fell 37.79%. Opened discovery
changed -1.40%, with an interval from -3.51% to +1.95%, inside its +3% noninferiority
bound, while peak RSS fell 15.37%. Its exact counter run recorded 4,837,756 scoped
allocations against 4,939,511 for the control, a 0.979 ratio, with identical engine and
commit digests and zero detached builds inside the opened route.
The one-shot counter run removed 206,188 scoped allocations and 23,353,892 allocated
bytes from the exact control while preserving all retained counts and the tree digest.
On the deterministic 2,080-entry slope fixture, detached growth is now 10,671
allocations, or 5.13 per entry, while opened growth is 50,413, or 24.24 per entry.
The platform ceilings remove the same two detached representation allocations and one
opened arena allocation from the prior measured slopes, and the existing injected
one-allocation-per-entry negative case keeps every runner’s bound tight.

The host was not quiet: an unrelated test process held one core and load exceeded the
protocol limit during the composite run.
The result therefore justifies retaining the composite for full correctness validation,
not a claim-grade H86 verdict.
A clean post-change sample attributes 3.82% of `cold-scan-index` and 3.94% of
`default-tree` samples to allocator frames, down from 5.63% and 5.90%; index frames
remain below 0.4% and filesystem work remains about 71%. Because the measured
representation already exceeds both wall and RSS targets, adding conditional roll-up
state now would spend complexity without a demonstrated remaining need.
That is an evidence-driven reduction from the original four-part layout proposal, and
the quiet-host and Linux stages remain mandatory before H86 closes.
The source selected for validation also changes the newly allocated directory payload to
compact storage in place rather than allocating and replacing that box.
The exploratory binary still paid the transient replacement, so its result is
conservative and the quiet stage must measure the final binary identity.

The local structural verdict uses `c6380f7646524b51dbfcfec7e2efac49bf89d34b` as its
immediate immutable control and `b75bf85a33edd9fe65d97df9395072797e54426e` as the
historical parity control.
It requires at least twelve valid quiet, warm-steady, paired and interleaved trials on
the 113,794-entry MetaBrowser checkout: `default-tree` must improve at least 3% with its
paired interval below zero, `cold-scan-index` must move in the same direction, peak RSS
must fall at least 20%, the two one-shot jobs must meet this plan’s historical parity
limits on both real subjects, and opened discovery must remain within the +3%
noninferiority and 1.05 allocation bounds while recording zero detached-builder uses.
Exact engine, report, scope, snapshot, and worker-count differential oracles remain
mandatory.

The original Linux floor and RSS claims are a second evidence stage, not relaxed local
targets. Its `arena_spike` reference uses a preregistered low-churn warm-steady
preparation: three complete spike warmups immediately before every retained sample and
no intervening full-index or deliberate memory-churn process.
Every arm records both `p95/median` and `max/min`; H86 requires candidate ratios at or
below 1.5 and 2.0, respectively.
If the prepared spike still has `max/min` above 2.0, the floor ratios stay unresolved
instead of selecting one timing cluster after the run.
A Darwin acceptance may keep the implementation in this stacked pull request, but it
does not close the Linux H86 epic.

The Linux stage has now run, and it rejects.
[exp-102](../../experiments/exp-102-h86-linux-evidence-stage-relative-gates-pass-floor-gates-fai.md)
measured candidate `5d7b86f` against the immediate control `c6380f7` on the
450,001-entry generated subject over twelve paired interleaved trials with zero invalid
samples, exact engine digests at worker counts one through four, and no post-run tree
drift. The relative gates pass: `cold-scan-index` wall fell 18.16% (95% interval -24.25%
to -13.72%) with peak RSS down 49.4%, `default-tree` wall fell 31.70% (-34.31% to
-29.15%) with peak RSS down 35.9%, `opened-discovery` improved 10.73% against a +3%
noninferiority bound, and every candidate `p95/median` is at or below 1.109 with
`max/min` at or below 1.324. The floor gates fail.
Against a `parfloor stat` parallel syscall floor of 316.4 ms and an `arena_spike` cell
of 362.8 ms and 30.5 MiB, the candidate’s `cold-scan-index` is 4.86x the syscall floor
against the 1.4x gate and 5.03x spike RSS against the 3x gate, and `default-tree` is
2.60x and 6.59x. Both floor cells are stable -- `arena_spike` `max/min` 1.204 and
`parfloor` 1.391 -- so the unresolved-ratio escape hatch does not apply and the ratios
reject rather than abstain.
The cell, its preparation, and its raw samples are recorded in
[the Linux floor cell note](../../research/research-2026-09-02-linux-floor-cell-for-h86.md).
That note also carries the reusable mechanism: `parfloor` at 316 ms against
`arena_spike` at 363 ms means an index-shaped retained result costs about 15% over raw
parallel enumeration, so the remaining 2.6x on `default-tree` is consumer-side and is
not in the syscall layer.
The run is `exploratory` stage on an `uncontrolled` shared KVM host, which is sufficient
to reject a floor ratio measured against same-session denominators and is not a
quiet-host verdict; the quiet-host stage remains open, and `fdu-xde5` and the campaign’s
Linux floor claim remain open with it.

After the journal preflight, the leading exact-update profile cost is the
`StructuralOverlay` required to prove arbitrary public mutations atomically before state
changes. Scanner discovery no longer pays for that boundary, and the remaining public
path has different correctness obligations; changing it is outside this campaign unless
a profile and independent model identify a simpler proof with a material remaining gap.

A separate earlier 12-pair comparison with the preserved pre-rewrite binary measured the
whole stack rather than attributing one commit.
On this corpus, `default-tree` is 6.07% faster, with a paired 95% interval from -7.68%
to -0.59%; `cold-scan-index` is 1.50% faster by median, with an interval from -5.64% to
+0.17%. This reaches the one-shot wall-time parity target on the first pinned subject.
At that checkpoint, the second subject, deterministic allocation guards, and full
handoff gates remained open before the campaign’s final parity bead could close.

Phase 2 passes when detached streaming counters are zero, exact commit and journal
oracles remain unchanged, and profiles no longer identify ancestry-path comparison or
unused consequence construction as the leading detached cost.

### Phase 3: Close the gap and keep it closed

- [ ] Re-profile all five jobs and rank only mechanisms that can reach the remaining
  parity gap.
- [ ] Iterate on the leading measured cost until the one-shot thresholds pass or two
  consecutive profiles find no mechanism capable of reaching 3%; any proposed target
  revision requires a separate design decision with evidence.
- [x] Add deterministic per-entry allocation guards for detached construction and opened
  discovery, including injected negative cases that prove one extra allocation per entry
  fails each ceiling.
- [x] Add zero-work assertions for detached effect, impact, journal, delta, and ancestry
  counters to `make check` without adding a timing gate; the same test proves opened
  discovery records no detached-builder work.
- [ ] Run `make check`, `make cross-lint`, the exact-commit independent model,
  opened-root goldens, and the paired performance protocol.
- [ ] Record every accepted and rejected experiment, update the opened-root plan’s live
  status, and close the linked beads only after the stacked PR’s CI passes.

## Bead Graph

Epic `fdu-748k` owns this plan.
The two correctness fixes can proceed independently; performance work begins only after
both close. Later blockers preserve experiment attribution by changing one structural
boundary at a time.

| Bead | Priority | Work | Blocked by |
| --- | --- | --- | --- |
| `fdu-vev7` | P0 | Fix controls-on snapshot reuse for controls-off public opens | — |
| `fdu-lksd` | P1 | Canonicalize encoded public observation paths before mutation | — |
| `fdu-01d0` | P0 | Profile detached, opened, and large-batch mutation with scoped counters | `fdu-vev7`, `fdu-lksd` |
| `fdu-wy89` | P0 | Make detached application skip exact commit consequences | `fdu-01d0` |
| `fdu-nrdl` | P0 | Replace scanner ancestry overlay with a resolved-parent proof | `fdu-wy89` |
| `fdu-1jz6` | P1 | Remove duplicate path ownership from exact impact and journal publication | `fdu-nrdl` |
| `fdu-lj4h` | P0 | Prove one-shot parity and add deterministic regression guards | `fdu-1jz6` |

Existing regression bead `fdu-pro1` now points to this spec and remains open until
`fdu-lj4h` proves its acceptance criteria.

## Testing Strategy

Correctness uses red-green tests at the public boundary:

- snapshot scope tests exercise every controls-on and controls-off direction under
  `Auto`, `Only`, and cold fallback;
- path tests compare encoded representation, not only `Path` equality;
- the independent reference model compares exact facts, roll-ups, clock, effective
  changes, impact, journal loss, and state after every operation;
- scanner-proof tests deliberately construct missing-parent, wrong-order, scope-escape,
  and stale-parent cases through the internal test boundary and require rejection before
  mutation;
- detached and exact sinks replay the same trace and produce the same index digest and
  stats;
- opened discovery, refresh, observation, overflow, cancellation, and change polling
  retain their existing integration and golden coverage;
- the default CLI goldens remain byte-for-byte unchanged.

Performance tests use exact semantic oracles.
Allocation guards use a fixture large enough that fixed harness allocations cannot hide
one extra allocation per entry.
The large public batch and repeated-batch jobs separately expose one-time overflow
behavior and per-batch path-set behavior.

## Delivery and Stacked Pull Requests

The GitHub stack is:

1. opened-root rewrite;
2. PR #50, control-state scale design;
3. PR #51, first whole-scan allocation fixes and control-observation gate;
4. `codex/streaming-performance-parity`, this plan and its implementation.

The fourth pull request remains draft until Phase 1 correctness passes.
Commits remain reviewable in this order:

1. plan, experiment baseline, and bead graph;
2. cache-scope correctness;
3. encoded path canonicalization;
4. measurement jobs and counters;
5. detached consequence sink and compatibility removal;
6. scanner ancestry proof and owned-path handoff;
7. exact impact and journal-copy cleanup;
8. parity evidence and deterministic guards.

If PR #51 changes, rebase this branch onto its updated head and rerun the correctness
and baseline phases.
Do not retarget this pull request to `main` while its parents remain open; that would
turn the stacked diff into the whole opened-root rewrite.

## Rollout Plan

The branch changes no default CLI surface.
The detached path becomes the ordinary one-shot implementation after correctness, model,
allocation, and performance gates pass.
Exact opened-root and public mutation paths remain opt-in through their existing APIs.

The pull request stays in the stack until PR #50 and PR #51 merge.
GitHub then retargets or rebases the child against the merged parent as needed,
preserving one reviewable functional delta.

## Acceptance Criteria

- Both correctness defects have focused regression tests that fail on PR #51 and pass on
  the new branch.
- Public returned indexes always match the requested semantic scope.
- Public paths are canonical in encoded representation.
- The one-shot CLI cannot reach streaming-only consequence work.
- Exact `Commit`, opened-root, observation, refresh, and bounded-journal functionality
  is unchanged under the independent model and integration tests.
- One-shot wall time, engine time, allocation events, reallocations, and allocated bytes
  meet the stated parity thresholds against `main` on both nominated real subjects.
- Every structural performance decision has a before profile, semantic oracle, paired
  comparison, and experiment-ledger entry.
- Deterministic guards fail when one per-entry allocation or one detached streaming path
  is deliberately restored.
- `make check`, `make cross-lint`, and stacked-PR CI pass.

## Open Questions

- Does a numeric resolved-parent proof or same-parent-run representation produce the
  smaller implementation after the first scanner spike?
  Both preserve the required boundary; profile and diff size decide.
- After detached parity, does arbitrary public batch preflight still justify replacing
  its path-keyed overlay with an ID-keyed transaction plan?
  Treat this as a separate exact-streaming experiment so it cannot delay CLI parity
  without evidence.
- Does retained exact-commit cloning remain material after oversized clones and eager
  compatibility projection are removed?
  Add `Arc<Commit>` only if the profile says yes.

## References

- [fdu design principles](../../architecture/fdu-design-principles.md)
- [fdu engine architecture](../../architecture/fdu-engine-architecture.md)
- [opened-root inventory engine plan](plan-2026-08-25-fdu-opened-root-inventory-engine.md)
- [progressive results plan](plan-2026-08-11-fdu-progressive-results.md)
- [performance campaign 2](plan-2026-08-23-fdu-performance-campaign-2.md)
- [performance loop](../../guides/performance-loop.md)
- [performance instrumentation playbook](../../guides/performance-instrumentation-playbook.md)
- [performance experiment ledger](../../reports/report-2026-08-10-fdu-performance-experiments.md)
- [PR #51](https://github.com/jlevy/fdu/pull/51)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
