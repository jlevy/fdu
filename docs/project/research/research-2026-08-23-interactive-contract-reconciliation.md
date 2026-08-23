# Research: Reconciling the Interactive-Client Contract with the Inventory-Engine Research

**Date:** 2026-08-23

**Author:** fdu project

**Status:** Complete

## Overview

Two documents now describe the same integration from opposite banks.
[The interactive-client integration spec](../specs/active/plan-2026-08-23-fdu-interactive-client-integration.md)
(fdu PR #44) records what a measured end-to-end exercise found the engine still owes an
embedded consumer, and deliberately declines to specify the metabrowser side of the
seam.
[Metabrowser’s inventory-engine research](https://github.com/jlevy/metabrowser/pull/74)
(metabrowser PR #74) is that other side, written by that project’s maintainers against
PR #44’s exact commit: a Metabrowser-owned provider boundary with two backends, a shared
semantic contract, a query and event model, and a list of corrections to the fdu plan.

This review reads them together and answers four questions: where they agree, where they
genuinely differ, which position should win each difference, and what that does to the
integration spec.

The headline is that the architecture is not in dispute.
Both documents place the seam at the retained engine rather than the walker, keep
callbacks out of the FFI boundary, keep classification and aggregation in Rust, and keep
wire formats out of fdu.
What remains are eight genuine differences, each adjudicated below with the case for and
against. The net effect on PR #44 is a set of amendments — two features shrink, two
readiness verdicts flip, four tracked beads join the critical path, and two open
questions close — none of which reverses its direction.

One fact gives the metabrowser research its weight and is worth stating first: every
checkable claim it makes about fdu source was verified here at the PR #44 commit
(`64398b7`), and all of them hold.
Two of them are limitations fdu’s own doc comments already record, bead numbers
included. A review that accurate earns adoption of its corrections; the sections below
are about *which* corrections, and in what order.

## What the Research Gets Right About the Code

| Claim in PR #74 | Verified against | Holds? |
| --- | --- | --- |
| `children()` clones every child and each directory child’s complete extension map | `IndexHandle::children` materializes an owned `RollUp` per directory child, and `RollUp.by_ext` is an unbounded `BTreeMap<String, ExtTally>` (`index.rs`) | Yes |
| The logical-extension algorithms disagree: File Rollup Format takes up to two eligible trailing components; fdu takes the final suffix and folds only `.tar.*` | `derive_ext` in `classify.rs`: `archive.tar.gz` yields `.tar.gz`, everything else the last suffix, so `release.v2.zip` is `.zip` where the format says `.v2.zip` | Yes |
| A directory’s provenance is its own, not its subtree’s, so a revalidated directory can contain cached descendants | The `Index::provenance` doc comment says exactly this and forbids reading it as a subtree guarantee (`fdu-fka6`, `fdu-b1ts`) | Yes — already tracked |
| Cached-to-revalidated transitions do not advance the clock, appear in `since()`, or reach the delta sink | Same doc comment: “treat this as a poll-only view” (`fdu-jxs0`, `fdu-livs`) | Yes — already tracked |
| Roll-up counts cannot distinguish a symlink-only subtree from an empty one | `contribution()`: `Symlink \| Other` contribute `InternedRollUp::default()`, so both subtrees report zero files and zero bytes | Yes |

Two notes on scope. Symlinks and special objects *are* retained entries — they appear in
listings and carry kinds; only the aggregates are blind to them, which is precisely the
listing-time “empty” ambiguity the research names.
And the two provenance rows are not newly discovered gaps: the
[progressive-results plan](../specs/active/plan-2026-08-11-fdu-progressive-results.md)
already designs subtree composition (weakest source, oldest observation, worst status,
merged through the existing reducer path) and transition emission on the session stream,
confirmations as well as corrections.
What the research changes about them is their **schedule and grading**, not their design
— PR #44 consumed that plan without sequencing its beads, and graded the resume cursor
“Ready” while trust changes are invisible to it.
That correction is accepted below.

## Where the Documents Agree

The agreement list is long enough that the tensions need framing against it.
Both documents independently arrive at:

- **The seam is the retained engine, not the walker.** PR #74’s “reject the walker swap”
  reads like a rebuttal of PR #44 but is aimed at metabrowser’s own H40 hypothesis ("a
  native parallel walker"); PR #44’s seam table already assigns fdu the walker, the
  aggregates, *and* the watcher as one unit.
  The two seam tables describe the same seam.
- **No callbacks across the FFI boundary.** Pull-based bounded batches, GIL taken per
  batch, a slow consumer degrades to a truncated feed and resyncs, never a stalled
  producer. PR #74’s `events()` contract is PR #44’s streaming-boundary section restated
  as a protocol.
- **One delta shape for boot fill and live changes**, and one resumable cursor over it.
- **Shared reads during a write lead everything.** Both lists put `fdu-gav9` first; PR
  #74’s dependency order and PR #44’s Phase 0 agree to the item.
- **A runtime registry, classification in Rust, dual populations maintained rather than
  recomputed, bounded outputs with stated remainders.** The research’s “no unrequested
  extension-map copy” and the spec’s “bounded rows with a stated remainder” are the same
  rule applied at different altitudes.
- **Breadth-first as operational policy**, outside cache identity, with final state
  independent of order.
- **The watch additions**: per-batch dirty aggregate sets, scoped refresh as the
  hint-ingestion primitive, an explicit polling backend, an async adapter over the
  blocking iterator.
- **Cache honesty through labeled provenance** — approximate-then-converge as the
  product behavior, “fast and labelled” rather than fast and wrong.
- **The measurement discipline**: paired interleaved trials on real corpora, floor
  ratios where the probe supports the host, no wall-clock gates in shared CI. PR #74’s
  three-layer frame (engine, server, browser) extends the fdu loop rather than
  contradicting it, and its warning that an engine win can hide an end-to-end regression
  is the right adoption gate.
- **The Python engine survives as oracle and fallback**, not as a parallel state layer.

None of this needed reconciling.
The provider protocol itself — `InventoryBackend`, the coordinator, the seven
projections, the wire mapping — is metabrowser-side architecture that PR #44 explicitly
declined to specify, and it is the piece PR #44’s “adoption sequence belongs to that
repository” sentence was waiting for.
The right reading is complementary halves, not competing designs.

## The Two Documents as Designs

**The integration spec’s strengths** are evidence and discipline: every gap is
demonstrated rather than asserted, every capability lands engine-first under the parity
harness, and the additions stay generic to any embedder.
Its limits follow from its vantage point.
Readiness was graded as “does the API exist and mean the right thing,” not “can a server
be built on it as it stands,” which is how `children()` and the resume cursor earned
“Ready” verdicts a client-side review could overturn.
The client-side coherence problem — several reads composing one response must observe
one version — is invisible from the engine bank, because each Rust read is individually
coherent. And the boot-fill streaming section inherited more of metabrowser’s
walker-yields-everything shape than the retained-engine architecture wants.

**The research’s strengths** are structural: it makes parity *executable* (two
providers, one contract, three oracles), it fixes a coherence bug class by construction
and names the live instance in its own code (the rollup payload/ETag race), it bounds
the FFI event stream by consumer interest, and it resolves the registry release-cadence
question cleanly. Its limits are the mirror image.
It is silent on the reducer cost its own demands add — subtree provenance and leaf
counts are maintained per-directory state on the same ancestor-merge path planes and
groups multiply, the path campaign 2 intends to replace — and one demand (lazy warm
serving) is sequenced earlier than its own scale evidence supports.
Its protocol sketch also carries more generality than two in-tree providers need, which
is a cost the “internal, changes atomically” framing keeps survivable but only if it is
held to.

## The Eight Differences, Adjudicated

### 1. Hidden files: prune as scope, not a second tag plane

The spec proposed `hidden` as a second tag rule with a maintained plane and left “does
show-all justify a third plane” open.
The research prunes hidden names at scope with an exact-name allowlist, because the
product has no hidden toggle and pruning avoids the I/O and memory of walking paths
never shown.

**Adopt the research’s position.** Hidden trees — `.git`, caches, virtualenvs — are
routinely the largest part of a working tree, and a tag plane does not avoid walking
them; it requires it.
The one named consumer of the hidden feature wants pruning, which triggers the spec’s
own axis test: the hidden *plane* now has no consumer and joins the tag registry in the
non-goals. What ships is scope-level hidden exclusion with an allowlist, fingerprinted
into snapshot identity like any retained-set change, and exactly one tag rule:
gitignore. The spec’s populations become what the research calls them — a named profile
(`all`, `unignored`) rather than a general complement algebra — with the partition
property kept general internally, where it costs nothing.

Two implementation consequences the revision must carry.
Control files are themselves dot-named, so `.gitignore` must be readable — and `.git`
detectable for repo-root semantics — without becoming retained entries; the research
words this correctly.
And fdu’s own CLI default is unchanged: a du replacement counts everything, so hidden
exclusion is opt-in scope configuration there, surfaced under the normal parity rules.

The case against: a future client with a show-hidden toggle pays a rescan on toggle
instead of flipping planes.
Accepted — the toggle has no consumer today, and taxing every scan with hidden-subtree
I/O to keep a hypothetical toggle cheap is the wrong default.
Evidence reopens the question.

### 2. What streams across the boundary: interests, not the whole walk

This is the deepest difference, and it is about content, not transport.
Metabrowser today streams every entry because its state lives *above* the stream: the
Python index is built from walker yields, so the yields must carry everything.
The spec keeps state below the stream but inherited some of the full-stream instinct —
its session section has the boot fill flowing as entry deltas in the same shape as
watch. The research points at the consequence: serializing every cold-scan entry into
Python “when no consumer needs it” recreates, on the event channel, the per-entry FFI
cost the retained-engine boundary exists to remove.
That is walker-swap thinking relocated, and the research is right to refuse it.

**Adopt interest-scoped events.** With the index native and read-anytime, the client
needs bounded reads of what is visible, a signal naming which aggregates went stale (the
dirty set the spec already specifies), progress and trust transitions, and entry-level
deltas only for scopes that genuinely mirror — expanded prefixes, the catalog.
The transport keeps every property the spec established: pull, bounded, batched,
gap-then-resync, never a callback.
What changes is the default content: dirty sets and status flow always; entries flow by
subscription. An interest change re-baselines through a coherent read and resumes from
that read’s cursor, which is the research’s rule and the right one.

The case against: interests are subscription semantics — session state, re-baseline
rules, more tests — and at the 120k orientation scale a firehose the client drops is
affordable. Accepted at 120k; not at the home-folder scale both projects aim at, where
the firehose is millions of conversions nobody reads.
The middle path is to keep the vocabulary **closed and small** — status/progress, dirty
aggregates, entries-under-prefixes, catalog pages — and to note that a v1 shipping only
the first two is coherent, because read-on-dirty covers visible rows without any entry
stream at all. Whether prefix entry deltas ever pay for themselves over read-on-dirty is
a measurement for the serving benchmark’s live-change dimension, not a design argument.

### 3. Reads: versioned, bounded, batched — and the `children()` verdict flips

The research’s coherent-read rule comes from a live defect in its own codebase: the
rollup route samples a revision for its ETag before dispatching a query whose view can
observe later writes, so the body and the version can disagree.
The rule that retires the class: a payload and its version are one atomic result, minted
inside the same read boundary, and the ETag derives from the returned version — never
from a counter sampled beforehand.

fdu’s Rust reads are individually coherent under one guard, but that is not the same
property. Three things are missing, and all three become spec items:

- **The version on the result.** Every query result carries the clock it was read at,
  plus the scope and registry fingerprints that scope its meaning.
  Cheap, general, and what makes client-side caching honest.
- **Several projections under one guard.** A route composing a listing, filtered totals,
  and navigation tallies from separate Python calls can straddle a commit; nothing ties
  the pieces to one version.
  A batched read evaluating multiple queries under one read guard — returning one
  `(clock, cursor, state)` — is the primitive the adapter needs, and it also collapses
  per-call FFI overhead.
  The alternative, compose-then-check-and-retry, can livelock under sustained watch
  commits; the guard-held bundle is simpler to reason about and to test.
- **Bounds on the hot listing.** `children()` returns every child and copies each
  directory child’s complete extension map across the boundary.
  The research’s split is right: child rows carry scalar totals, classification
  identity, and provenance; the extension breakdown is its own bounded query for the one
  directory being inspected.
  Child rows gain a bound, a page cursor, and a remainder, closing the spec’s open
  question ("does `children()` need its own bound") as yes.

Adopt also the research’s work-counter rule — each result reports entries visited, rows
returned, bytes copied, lock wait — because it converts “no hidden O(index) pass” from a
review principle into an assertable contract.
The seven-projection algebra itself stays metabrowser-side; fdu ships generic primitives
sufficient to answer each projection in work proportional to output, which is the
existing two-tier cost rule extended to the new surface.
Two verdicts in the spec’s contract table flip from Ready to gap: `children()` (this
section) and the resume cursor (§5).

### 4. Classification: the format is the authority, and the algorithm moves into the dialect

The logical-extension divergence is verified and cannot be repaired by a registry alone:
facts derived differently stay different.
The resolution is the one fdu’s own code asks for — `derive_ext`’s comment says
generalizing compound stems “belongs in the rule dialect, not in a hand-maintained list
here.” So the compound-extension algorithm becomes part of the dialect: the File Rollup
Format rule (up to two eligible trailing components), implemented in Rust, selected by
the active registry, and covered by its fingerprint.
fdu runs the format’s conformance corpus — metabrowser remains its authoritative host;
fdu exposes the parser and classifier entry points so the corpus executes unchanged —
and the corpus, not either engine’s intuition, pins the interaction this review flags as
underspecified: matching precedence over compound tails (whether `release.v2.zip` still
matches a rule keyed `.zip`).

The registry handoff adopts the research’s shape wholesale, because it closes the spec’s
open question exactly: metabrowser supplies registry bytes and expected identity at
session open, fdu parses, validates, indexes, fingerprints, and echoes the identity;
disagreement fails the open.
A metabrowser vocabulary change is then a metabrowser release, never an fdu release,
which is the decoupling the spec wanted and could not get from a shared file.

One decision stays fdu’s own and does not gate the integration: whether the *compiled
default* registry migrates to the format’s derivation.
That changes extension buckets in fdu’s own views and goldens (`.min.js` appearing as a
bucket is a browsing judgment), so it follows the normal golden-review path on its own
schedule. Carrying two derivations, scoped by registry identity, is a real but small
bifurcation — and it is the same statement as “the registry versions the cache.”

### 5. Provenance and the trust clock: right demand, already-designed mechanism

The research calls subtree provenance and clocked trust transitions “integration
prerequisites, not later polish.”
The demand is correct and the spec’s grading was wrong: a browser resuming an SSE cursor
either clears approximation marks too early or never learns to clear them, because the
transition that should clear them never reaches the feed.
The engine’s own doc comment says the same thing in different words.

But nothing here needs redesign.
Progressive results already specifies composition through the reducer path and
transition emission on the session stream; the four beads exist (`fdu-fka6` P0,
`fdu-b1ts` P1, `fdu-livs` P1, `fdu-jxs0` P2). The amendment is scheduling: those beads
join the integration’s critical path alongside the session, `fdu-jxs0` rises from P2 to
match its new position, and the contract table’s “Ready — document the mapping” verdict
on the resume cursor becomes a gap until trust rides the clock.

Two refinements from the research are worth adopting while landing it.
First, trust transitions must be **aggregated to be clockable**: a revalidation sweep
flips sources on millions of entries, and per-entry events would flood the feed it is
supposed to serve. Subtree composition is exactly the aggregation that turns the sweep
into a bounded number of “this subtree is now verified” events — a
count-of-unverified-descendants reducer yields both the composed value and its
zero-crossing event, so the two demands are one mechanism, which is the strongest reason
to land them together.
(Progressive results already notes these aggregates are not invertible under deletion or
revalidation; the recompute path is part of the same work.)
Second, **coverage is only monotone during additive discovery**: errors, invalidation,
and reconciliation move it both ways, so phase and cause travel with the value.
That sharpens the progressive-results monotonicity contract from an absolute claim into
a labeled one, and the session’s progress surface should say which phase it is in.

The case against prerequisite status — ship first, let the client poll provenance for
visible rows — recreates the split-brain the doc comment names (feed silent, poll sees
transitions) on the exact path the client depends on.
The cost of doing it right is bounded by designs already written; take it.

### 6. The no-gap handoff: an outcome became an algorithm

The spec states the property — the walk’s completion clock is the watch’s resume clock,
tested for the no-gap property.
The research is right that this is “a desired outcome, not a complete algorithm,” and
supplies the sequence: start capturing watch events before or atomically with baseline
discovery; accumulate them in a bounded native log while the walk or revalidation runs;
reconcile every captured event against observation expectations; publish complete/fresh
only when reconciliation reaches a known cursor; on overflow or an unreliable backend,
invalidate the affected scope and verify it.

Adopt as the specified implementation of the session-to-watch bead’s third requirement,
cold and warm alike — capture precedes revalidation, not only first discovery.
The test sharpens accordingly: a mutation landing during the walk appears in the walk or
in the feed, never in neither and never torn across both.
There is no serious case against; the only cost is writing it down, and the failure it
prevents is silent.

### 7. Lazy warm open: a measured gate, not a precondition

The research places “cached shallow results readable before full index materialization”
inside its pre-adoption fdu work.
Push back on the sequence, not the substance.
The numbers: snapshot load plus revalidation measures 0.090 s + 0.106 s at 120k entries;
metabrowser’s shipped ceiling is a 500k-file cap; and the ~11 s figure that motivates
lazy open is a 5.4M-entry snapshot, a scale the current product refuses to serve.
Persisted reducer state and bulk arena load (H33/H34) may buy most of the warm win
without block-lazy loading (H35) at all.
So first adoption runs on snapshot-load-plus-revalidate; the serving benchmark’s
warm-usefulness dimension — the research’s own instrument — decides when the lazy tier
is required, presumably when the caps lift and the default flips at scale.
Ownership stays with progressive results (`fdu-hd96`).

The case against deferral: warm open is the moment the research’s product argument cares
most about, and adopting with a warm regression would be felt daily.
Answered by the same instrument — at current ceilings the measured warm path carries two
orders of headroom against today’s 2.2 s first row, and if the benchmark disagrees at
500k, the gate catches it before the default changes.
That is what the gate is for.

### 8. The small facts, adopted

- **Leaf counts.** Roll-up state gains symlink and special-object counts (or one
  non-directory leaf count), so “empty” is decidable from the aggregate at listing time.
  Snapshot version bump; the partition property extends to the new fields.
- **Checked arithmetic.** Adopt as engine discipline: sums never wrap silently;
  mechanism is the engine’s choice.
- **Explicit `as_of` on recency and age queries.** Adopt — replay oracles and honest
  ETags both need time to be an input, not an ambient read.
- **Path identity.** The engine is already native end to end; what is owed is a
  documented Python contract keeping raw identity available beside any rendered string.
  The host wire encoding is metabrowser’s.
- **Four-kind semantics.** Already true in the engine (`File`, `Dir`, `Symlink`,
  `Other`); the contract states it rather than builds it.
- **Caps become budgets.** Metabrowser’s `max_files=500_000` and `max_depth=20` exist
  because its walker could go no further; under the provider they become explicit
  resource budgets producing typed partial coverage, never implicit correctness caps.
  Nothing to build on the fdu side; the sentence matters.

## The Cost Neither Document Prices

Every adopted demand that adds maintained per-directory state — the unignored plane
(spec), groups (spec), composed provenance (research and progressive results), leaf
counts (research) — multiplies the ancestor-merge path.
That is the path exp-064’s H94 took from 43.73% to 14.07% of profile, and the path
campaign 2 plans to delete rather than tune in its content-tier instance of H86
(`fdu-cq7t`). The spec already flagged planes-times-groups for exactly this reason
(`fdu-n4gn`): measure against the replacement shape, on a dense real subject of at least
50,000 entries nominated by `make perf-subjects`, because exp-065 showed sparse
generated corpora flatter this class of change.

The amendment is scope: `fdu-n4gn` measures the full union — planes, groups, provenance
composition, leaf counts — because the reducer will carry all four together or not at
all, and a cost acceptable for each alone can be wrong in combination.
This is also the item to hand back across the seam: the research’s Phase 5 campaign
inherits the same question at the server layer, and its plan should say so.

## The Shape of the Combined Design

Stated once, as one design rather than two documents:

1. **The boundary**: a Metabrowser-owned provider protocol with a coordinator above it
   and two providers below; fdu stays application-independent; the adapter translates
   per bounded batch and holds no state, no mirror tree, no second watcher.
2. **State**: one native retained index below the seam; a sparse overlay above it for
   application decorations, joined only onto rows being serialized, never contributing
   to totals.
3. **Classification**: File Rollup Format and its conformance corpus are the shared
   authority; the registry travels at session open with identity echoed back; the
   compound-extension algorithm lives in the dialect; fingerprints version every cache.
4. **Scope and populations**: hidden names pruned by scope with an exact-name allowlist,
   fingerprinted; gitignore tagged with full negation semantics; two maintained
   populations (`all`, `unignored`); execution policy (order, workers, batching) stays
   out of cache identity.
5. **Reads**: bounded, paged, scalar projections; several projections evaluable under
   one read guard; every result stamped with clock, cursor, state, fingerprints, and
   work counters; ETags derived from returned versions only.
6. **Events**: one clocked stream carrying data deltas, dirty aggregates, trust
   transitions, progress, and reset markers; interest-scoped with a closed vocabulary;
   pull-based and bounded; a gap is explicit and forces re-read, never silence.
7. **Sessions**: read-anytime with `prioritize`; capture-before-baseline handoff into
   resident watching, cold and warm; monotone lower bounds claimed only in the additive
   discovery phase and labeled as such.
8. **Warm opens**: snapshot load plus revalidation now; persisted reducers, bulk load,
   and lazy blocks when the warm-usefulness measurements demand them.
9. **Verification and adoption**: conformance corpus, recorded-observation replay, and
   filesystem scenarios as the three oracles; the research’s acceptance gates; paired
   three-layer measurements with floor ratios on dense real subjects; the default flips
   only on an end-to-end win with no regressed layer.

## Amendments to the Integration Spec

The concrete revision list for PR #44, in its own phase order:

1. **Phase 0 unchanged.** Both documents independently rank `fdu-gav9` first; `fdu-4vkz`
   (`--order`) stays — the research neither needs nor objects to it, and the parity rule
   it serves is fdu’s own.
2. **Phase 1 shrinks.** Gitignore is the only tag rule; hidden becomes scope
   configuration (exclusion plus exact-name allowlist, fingerprinted; control files
   readable and `.git` detectable without retention).
   `fdu-mvt3` and `fdu-7rwf` re-scope accordingly; the third-plane open question closes
   as no.
3. **Phase 2 gains conformance and moves earlier.** `fdu-ctp5` absorbs the File Rollup
   logical-extension algorithm as a dialect property and adds execution of the
   metabrowser-hosted conformance corpus; the registry handoff is
   supplied-at-open-with-identity-echo; the registry-cadence open question closes.
   Sequence this phase before or beside Phase 1: every cross-engine oracle depends on
   classification agreement, and planes need it only at validation time.
4. **New: the coherent read surface.** Scalar bounded/paged child rows with remainder; a
   batched multi-projection read under one guard returning clock, cursor, and state;
   per-result work counters; fingerprints on results.
   New beads; the contract table’s `children()` verdict flips to gap.
5. **New: roll-up leaf counts** for symlinks and special objects, with the partition
   property extended and a snapshot version bump.
6. **Phase 4 grows.** `fdu-fka6`, `fdu-b1ts`, `fdu-livs`, and `fdu-jxs0` sequence with
   the session; `fdu-jxs0` rises from P2; the resume-cursor row flips from Ready to gap
   until trust transitions share the clock; the session bead (`fdu-4o0m`) specifies the
   capture-before-baseline handoff and the closed interest vocabulary; monotonicity
   language is scoped to the additive discovery phase.
7. **Phase 5 sharpens.** The cross-engine fixture (`fdu-vfyw`) becomes the three-oracle
   stack, and dual live engines over a changing tree are explicitly ruled out as an
   oracle; `as_of` parameters land with the recency surface; the Python path-identity
   contract is documented.
8. **`fdu-n4gn` extends** to the four-way reducer-cost union above.
9. **Naming.** Three things now answer to “session”: the watch session, the progressive
   scan session, and the research’s `InventorySession`. The two fdu surfaces should
   carry distinct public names before Python ships them.

## What to Send Back to the Metabrowser Review

**Endorse**: the boundary and the ownership split; extraction-first with the ETag fix
folded in; the sparse overlay; explicit-then-auto provider selection with reported
reasons; the acceptance gates; the three-layer paired measurement frame.
The research’s next step “revise fdu pull request 44 around the inventory-engine
boundary” is answered by the amendment list above.

**Push back, with reasons**:

- **Lazy warm serving** moves out of the pre-adoption work and behind the
  warm-usefulness gate (§7). The substance is agreed; the sequence was ahead of its
  scale evidence.
- **Interests stay a closed vocabulary** of roughly four values, and a first version
  carrying only status and dirty aggregates is legitimate; whether prefix entry deltas
  beat read-on-dirty is the live-change benchmark’s question.
- **The reducer cost of the new contract is not free**, and the plan on both sides
  should point at the same loop job (§ on cost above); a contract demand that adds
  maintained per-directory state buys its place with a measurement like any other
  change.
- **Hold the protocol’s generality line.** Two in-tree providers shipping together need
  a sealed internal contract; capability negotiation should stay reserved for genuine
  platform absences (watch backend availability), not become a branching axis in the
  coordinator.

**Offer**: the conformance corpus running against fdu’s classifier is the cheapest joint
deliverable with the highest information yield — it needs only the parser half of
`fdu-ctp5` and none of the metabrowser extraction, and it de-risks classification for
both plans at once. Propose it as the first cross-repository artifact.

## Sequencing Across the Two Repositories

The critical paths do not serialize.
fdu proceeds `fdu-gav9` → registry and conformance (with planes-as-rescoped beside it) →
coherent reads and directory facts → trust on the clock → session with interests and the
handoff → watch lifecycle → the reference embedder.
Metabrowser proceeds contract freeze (consuming this reconciliation) → Python provider
extraction with the ETag fix → the fdu adapter → oracles → the performance campaign →
the default decision.
The one early joint artifact is the conformance run; everything else crosses the seam
only through documents until the adapter phase.

## Open Questions

- Do prefix-scoped entry deltas ever beat read-on-dirty for expanded folders, or do
  dirty sets plus bounded reads carry the live UI alone?
  The serving benchmark’s live-change dimension decides.
- Does the compiled default registry adopt the File Rollup derivation, and when?
  fdu-only; golden churn and analysis-tier impact reviewed on its own schedule.
- Where does the conformance corpus live in fdu CI — a pinned vendored snapshot with a
  refresh job, or a fetched artifact?
  Supply-chain rules favor the pinned copy.
- Free-threaded CPython wheels (from the spec’s open questions) stand unchanged.

## References

- [The interactive-client integration spec](../specs/active/plan-2026-08-23-fdu-interactive-client-integration.md)
  (fdu PR #44, reviewed at `64398b7`) — the fdu-side contract this review amends
- [Metabrowser’s inventory-engine research](https://github.com/jlevy/metabrowser/pull/74)
  (metabrowser PR #74) — the client-side architecture this review adopts and answers
- [Progressive results plan](../specs/active/plan-2026-08-11-fdu-progressive-results.md)
  — the session, provenance composition, and lazy-open designs the adjudications lean on
- [Design principles](../architecture/fdu-design-principles.md) — the axis test, cache
  honesty, and truncation rules applied throughout
- [Campaign 2 plan](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md) and
  [the metadata-walk floor report](../reports/report-2026-08-23-metadata-walk-floor.md)
  — the ancestor-merge replacement and floor context behind the cost section
- Source verified at `64398b7`: `index.rs` (`children`, `contribution`,
  `Index::provenance` and its two tracked limitations), `classify.rs` (`derive_ext`),
  `watch_session.rs` (`Session::new`)
- Beads: `fdu-gav9`, `fdu-4vkz`, `fdu-mvt3`, `fdu-7rwf`, `fdu-ctp5`, `fdu-b2vy`,
  `fdu-e2p7`, `fdu-n4gn`, `fdu-mz1a`, `fdu-fh0k`, `fdu-rhu3`, `fdu-97pb`, `fdu-4o0m`,
  `fdu-16l7`, `fdu-tib6`, `fdu-knyw`, `fdu-vfyw`, `fdu-fka6`, `fdu-b1ts`, `fdu-livs`,
  `fdu-jxs0`, `fdu-e86o`, `fdu-a0j0`, `fdu-hd96`, epic `fdu-u7vo`

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
