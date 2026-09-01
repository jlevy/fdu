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
The implementation keeps one fact model and one reducer.
It does not fork the engine into a fast CLI implementation and a correct streaming
implementation.

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
| Do we create a second fact engine for the CLI? | No. All paths use the same reducer and roll-up logic. |
| Where is optional work selected? | Once per prepared batch, from the serving lifecycle and requested public outcome. Never from a per-entry CLI check. |
| What does detached one-shot construction retain? | Facts, roll-ups, scope, issues, provenance required by the returned index, and `ApplyStats`. It retains no exact change stream, impact set, journal entry, or compatibility delta. |
| What does an opened root retain? | Exact commits, bounded impact, lifecycle state, clock, and the bounded journal required by change consumers. |
| What does arbitrary public mutation retain? | Exact atomic validation and an exact commit outcome. It does not receive scanner-only trust. |
| How does the scanner avoid the ancestry overlay? | A private prepared-batch type proves canonical owned paths and parent resolution before mutation. Public observations cannot construct it. |
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
- [ ] Add the private owned scanner batch with resolved-parent proof; remove the
  path-keyed `StructuralOverlay` from detached and opened discovery.
- [ ] Move prepared scanner paths across boundaries once and stop recording effect paths
  when the selected sink cannot expose them.
- [ ] Bound impact accumulation and avoid cloning journal entries that exceed capacity.
- [ ] Run and record one experiment after each independently measurable change; reject
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

Phase 2 passes when detached streaming counters are zero, exact commit and journal
oracles remain unchanged, and profiles no longer identify ancestry-path comparison or
unused consequence construction as the leading detached cost.

### Phase 3: Close the gap and keep it closed

- [ ] Re-profile all five jobs and rank only mechanisms that can reach the remaining
  parity gap.
- [ ] Iterate on the leading measured cost until the one-shot thresholds pass or two
  consecutive profiles find no mechanism capable of reaching 3%; any proposed target
  revision requires a separate design decision with evidence.
- [ ] Add deterministic per-entry allocation guards for detached construction and opened
  discovery, including a negative fixture that proves each guard fails.
- [ ] Add zero-work assertions for detached effect, impact, journal, delta, and ancestry
  counters to `make check` without adding a timing gate.
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
