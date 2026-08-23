# Research: The Interactive Contract, Verified Against Source

**Date:** 2026-08-23

**Author:** fdu project

**Status:** Complete

## Overview

fdu and metabrowser are one design held in two repository contexts.
The split exists so each side can be worked at depth without carrying the other’s code
in context; it is not an interface between two owners, and nothing here needs
negotiating. What it does need is for the two documents to agree on what is true, which
is what this record establishes.

[The integration spec](../specs/active/plan-2026-08-23-fdu-interactive-client-integration.md)
states what an embedded consumer needs from fdu, measured end to end.
[The metabrowser plan](https://github.com/jlevy/metabrowser/blob/954b6ed/docs/project/specs/active/plan-2026-08-23-pluggable-inventory-engine.md)
states the consumer side: a sealed contract, two providers, one coordinator.
Both were written against the other, and the architecture agrees — the seam is the
retained engine, the FFI boundary is pull-based with no callbacks, classification and
aggregation stay in Rust, wire formats stay out of fdu.

This document holds three things the specs should not have to carry: what was verified
against fdu source, one correction that changes what gets built, and the decisions that
are now settled with their evidence.

The correction is the reason to read on.
Both documents described the compound-extension divergence in a way that, implemented
literally, would break classification for the exact names it was meant to fix.

## What Was Verified Against Source

Checked at `64398b7`, the commit the metabrowser review read:

| Claim | Verified against | Holds? |
| --- | --- | --- |
| `children()` clones every child and each directory child’s complete extension map | `IndexHandle::children` builds an owned `RollUp` per directory child; `RollUp.by_ext` is an unbounded `BTreeMap<String, ExtTally>` | Yes |
| A directory’s provenance is its own, not its subtree’s | `Index::provenance`’s doc comment says so and forbids reading it as a subtree guarantee (`fdu-fka6`, `fdu-b1ts`) | Yes, already tracked |
| Cached-to-revalidated transitions never reach the clock, `since()`, or the delta sink | Same doc comment: “treat this as a poll-only view” (`fdu-jxs0`, `fdu-livs`) | Yes, already tracked |
| Roll-up counts cannot distinguish a symlink-only subtree from an empty one | `contribution()` gives `Symlink \| Other` a default roll-up, so both report zero files and zero bytes | Yes |
| The logical-extension algorithms disagree | Partly — see below.  The raw derivations differ; the canonical results already agree, and that distinction decides the implementation | **Needs restating** |

Symlinks and special objects are retained entries carrying kinds; only the aggregates
are blind to them, which is exactly the listing-time “empty” ambiguity.
The two provenance rows are not new findings — progressive results already designs
subtree composition and transition emission, and the beads exist.
What changed there is grading and schedule: the resume cursor was marked Ready while
trust changes are invisible to it.

## The Correction: Two Levels, Not One Algorithm

Both documents said fdu derives `.zip` where File Rollup Format derives `.v2.zip`, and
concluded that fdu’s derivation should adopt the format’s rule.
The first half is true and the conclusion does not follow.

**What fdu does today**, run on a fixture holding `release.v2.zip`, `bundle.umd.min.js`,
`plain.zip`, `app.js`, and `archive.tar.gz`:

```text
--view types          --view extensions
archive     3 files   .js       2 files
javascript  2 files   .tar.gz   1 file
                      .zip      2 files
```

`release.v2.zip` classifies as `archive` and buckets as `.zip`; `bundle.umd.min.js`
classifies as `javascript` and buckets as `.js`. **Those are already the answers the
format wants.** Its packet requires `release.v2.zip` to derive `.v2.zip` *and
suffix-match canonical* `.zip`, and `bundle.umd.min.js` to derive `.min.js` *and
suffix-match canonical* `.js`. The format has two levels — a raw logical extension of up
to two eligible trailing components, and a canonical extension that drives rule matching
and roll-up bucketing.
fdu has one, and the one it has is the canonical one.

**So the gap is not a wrong derivation.
It is a missing level**, and the naive fix breaks the working half.
`classify_path_with_prefix` looks rules up by exact key in `RULES_BY_EXTENSION` with no
suffix fallback: change `derive_ext` to return `.v2.zip` and the lookup for key `v2.zip`
misses every rule, so an archive becomes `unknown:.v2.zip`. `ext_bucket` wraps the same
function, so the `.zip` roll-up bucket splits at the same time.
One edit, two regressions, both in the names the change was for.

What fdu builds instead is the pair:

- a **raw logical extension** — up to two eligible trailing components, per the format’s
  rule — exposed as its own value on entries and in the projections that want it;
- **canonical suffix matching**, so a raw extension that matches no rule falls back to
  its trailing component for rule lookup and for the roll-up bucket.

Canonical results then stay exactly what the fixture above shows, which is the property
to pin by test: adopting the raw level must not move a single existing bucket or type
row. The raw value surfaces where metabrowser needs it — navigation tallies, literal
filters, recent and catalog rows, and unknown `remaining_types` keys — and the parity
fixtures assert that separation directly.

Two consequences worth carrying into the spec.
The conformance packet has to gain direct basename-to-logical-extension cases before it
can serve as fdu’s oracle, since today it tests matching rather than derivation; the
metabrowser plan already commits to that.
And `derive_ext`’s own comment — that generalizing compound stems “belongs in the rule
dialect, not in a hand-maintained list here” — is still the right destination for the
eligibility rule, now as one of two levels rather than as a replacement.

## Settled Decisions

Each of these is now fixed on both sides, with the reason it went that way.

**Hidden paths prune at scope.** Not a second maintained tag plane: an allowlist-based
scope rule, fingerprinted into snapshot identity.
A tag plane still requires walking the hidden trees it exists to exclude, and those
trees — `.git`, caches, virtualenvs — are routinely the largest part of a working tree.
The plane had no consumer once the product had no hidden toggle, which is the spec’s own
axis test. Gitignore is the sole tag rule; `all` and `unignored` are the maintained
populations, named by profile rather than exposed as a complement algebra.
Control files stay readable without being retained.
fdu’s CLI default is untouched — a du replacement counts everything, so hidden exclusion
is opt-in scope configuration there.

**The first change stream carries no entry rows.** It carries cursor and version, dirty
directory prefixes or named projections, lifecycle and trust transitions, progress, work
counters, an all-dirty marker when the dirty set exceeds its bound, and a reset marker
on a cursor gap. The coordinator reads coherently when something visible goes dirty and
resumes from that read’s cursor.
This is the decision that keeps the retained-engine boundary honest: streaming every
cold-scan entry into Python would rebuild, on the event channel, the per-entry FFI cost
the boundary exists to remove.
Prefix and catalog entry deltas are optional optimizations that enter the contract only
if a live-change A/B beats invalidation-plus-bounded-read after binding copies and
browser convergence are counted.

**Reads are versioned, bounded, and bundled.** Every result carries the engine version
and change cursor captured at one boundary, lifecycle and coverage and freshness and
source and progress, scope and registry fingerprints, and work counters — entries and
directories visited, rows returned, lock wait, bytes copied across the binding.
Several projections evaluate under one guard so a composed response cannot straddle a
commit, and the HTTP cache key derives from the returned version rather than a revision
sampled before dispatch.
Child rows carry scalar directory facts only; extension breakdowns come from a separate
bounded roll-up projection.
This flips two of the spec’s readiness verdicts: `children()` needs a scalar paged form
with a remainder — closing its open question as yes — and the resume cursor is not ready
for SSE until trust rides the same clock.

**Trust is aggregated onto the data clock.** A revalidation sweep flips sources on
millions of entries, so per-entry provenance events would flood the feed they serve.
An unverified-descendant count per directory yields both the composed subtree value and
a bounded event at its zero crossing, so subtree provenance and clocked trust are one
mechanism rather than two.
Coverage is monotone only during additive discovery; every other transition carries its
phase and cause, which sharpens progressive results’ monotonicity contract from an
absolute claim into a labeled one.
The recompute path matters: these aggregates are not invertible under deletion or
revalidation.

**The registry travels at open.** Metabrowser owns File Rollup Format and its registry;
either backend receives the same immutable packet and expected identity, validates once,
returns the identity it indexed, and fails the open on disagreement.
A vocabulary change is then a metabrowser release and never an fdu release, which is the
decoupling a shared file could not give.
The conformance packet is exported at a reviewed revision and committed into fdu, with
CI verifying manifest and hashes locally — no network fetch, sibling checkout, or third
package, which is also what fdu’s supply-chain rules prefer.

**Warm open starts at persisted reducers and bulk load.** Lazy blocks become required
only if that path misses the standing warm-usefulness budget at a supported scale.
The numbers behind the sequencing: snapshot load plus revalidation measures 0.090 s +
0.106 s at 120k entries, metabrowser’s shipped ceiling is a 500k cap, and the ~11 s
figure motivating lazy open is a 5.4M-entry snapshot — a scale the product currently
refuses to serve. It is an optimization decision, not a semantic prerequisite.

**The no-gap handoff is a sequence, not an outcome.** Capture watch events before or
atomically with baseline discovery; accumulate in a bounded native log while the walk or
revalidation runs; reconcile every captured event against observation expectations;
publish complete and fresh only when reconciliation reaches a known cursor; on overflow
or an unreliable backend, invalidate the affected scope and verify it.
Cold and warm alike — capture precedes revalidation, not only first discovery.
The property to test is that a mutation landing during the walk appears in the walk or
in the feed, never in neither and never torn across both.

**Small facts, adopted without argument.** Roll-up leaf counts for symlinks and special
objects, so “empty” is decidable from the aggregate.
Checked arithmetic, with overflow an explicit failure.
An explicit `as_of_ns` on recency and age queries, so replay oracles and ETags both
treat time as an input.
A documented Python contract keeping raw path identity available beside any rendered
string. Four-kind semantics stated rather than built — the engine already has them.
Metabrowser’s `max_files` and `max_depth` caps become explicit resource budgets
producing typed partial coverage rather than implicit correctness caps.

**Naming.** Three things would otherwise answer to “session”: the watch session, the
progressive scan session, and the consumer-side handle.
Metabrowser renamed its own to `InventoryHandle`; fdu’s two need distinct descriptive
names before they enter the Python binding.

## The One Thing Still Unpriced

Every maintained addition above lands on the same path: the unignored population,
browsing groups, composed subtree provenance, and non-directory leaf counts all add
per-directory state that the ancestor merge carries.
That is the path exp-064’s H94 took from 43.73% to 14.07% of profile, and the path
campaign 2 plans to delete rather than tune in its content-tier instance of H86
(`fdu-cq7t`).

So the four are priced as one union, not four increments — the reducer will carry all of
them or none, and a cost acceptable for each alone can be wrong in combination.
Measured against H86’s replacement shape rather than today’s walk, on a dense real
subject of at least 50,000 entries nominated by `make perf-subjects`, because exp-065
showed sparse generated corpora flatter exactly this class of change.
`fdu-n4gn` carries it; the metabrowser plan prices the same union from the server layer.

This is the only item in the contract that is neither settled nor cheap, and it is the
one that should decide the final representation.

## What fdu Builds

Amendments to the integration spec, in its own phase order:

1. **Phase 0 unchanged.** `fdu-gav9` (shared reads during a write) leads both plans;
   `fdu-4vkz` (`--order` on the surfaces) stays, serving fdu’s own parity rule.
2. **Phase 1 shrinks.** Gitignore is the only tag rule; hidden becomes scope
   configuration with an exact-name allowlist, fingerprinted, control files readable
   without retention. `fdu-mvt3` and `fdu-7rwf` re-scope; the third-plane question closes
   as no.
3. **Phase 2 gains the second extension level and the packet, and moves earlier.**
   `fdu-ctp5` adds the raw logical extension, canonical suffix matching, the vendored
   conformance packet with local hash verification, and the registry handoff with
   identity echo. Sequence before or beside Phase 1: every cross-engine oracle depends on
   classification agreement, and planes need it only at validation time.
4. **New — the coherent read surface.** Scalar paged child rows with remainder; a
   bundled multi-projection read under one guard returning version, cursor, state, and
   fingerprints; per-result work counters.
5. **New — roll-up leaf counts**, with the partition property extended and a snapshot
   version bump.
6. **Phase 4 grows.** `fdu-fka6`, `fdu-b1ts`, `fdu-livs`, `fdu-jxs0` sequence with the
   session as one mechanism; `fdu-jxs0` rises from P2; the session bead specifies the
   capture-before-baseline handoff; monotonicity language scopes to additive discovery.
7. **Phase 5 sharpens.** The cross-engine fixture becomes the three-oracle stack —
   conformance packet, recorded-observation replay, filesystem scenarios — with dual
   live engines over a changing tree explicitly ruled out as an oracle.
8. **`fdu-n4gn` extends** to the four-way union above.

## How It Gets Built

Three vertical spikes rather than either full surface built against an assumed protocol,
with both design documents refined first:

1. **Classification and coherent read.** Extend and export the packet, vendor it into
   fdu, pass it unchanged, and serve one directory page plus one roll-up through the
   real PyO3 handle with a single version, cursor, and state record.
2. **Live lifecycle.** Capture-before-baseline under mutation, aggregated trust zero
   crossings, bounded dirty sets with read-on-dirty, gap and reset behavior, and the
   reducer union’s real cost on a dense subject.
3. **Production surface and adoption.** The full projections and the stateless adapter,
   the three oracles, then the engine, server, and browser comparison before any default
   changes.

A failed spike edits both documents and is rerun; spike code is promoted only once its
semantics, bounds, and measurements survive that loop.
The virtue of this order over phase-by-phase construction is that the riskiest
assumption — that the protocol shape survives contact with the real PyO3 handle — gets
tested in spike 1 instead of after both surfaces are built.

## Open Questions

- What does the four-way reducer union actually cost against H86’s replacement shape?
  The one unpriced item, and the one that should choose the representation.
- Do prefix-scoped entry deltas ever beat read-on-dirty for expanded folders?
  Deferred to a live-change A/B; the v1 contract carries no entry rows either way.
- Does fdu’s compiled default registry adopt the format’s raw level for its own views,
  or expose it only to registry-supplied consumers?
  fdu-only, and now low-stakes: canonical results do not move either way.
- Free-threaded CPython wheels, unchanged from the spec’s own list.

## References

- [The integration spec](../specs/active/plan-2026-08-23-fdu-interactive-client-integration.md)
  — the fdu-side contract these decisions amend
- [The metabrowser plan](https://github.com/jlevy/metabrowser/blob/954b6ed/docs/project/specs/active/plan-2026-08-23-pluggable-inventory-engine.md)
  and
  [its research](https://github.com/jlevy/metabrowser/blob/954b6ed/docs/project/research/research-2026-08-23-fdu-metabrowser-inventory-engine.md)
  — the consumer side of the same design
- [Progressive results](../specs/active/plan-2026-08-11-fdu-progressive-results.md) —
  the session, provenance composition, and lazy-open designs these decisions lean on
- [Design principles](../architecture/fdu-design-principles.md) — the axis test, cache
  honesty, and truncation rules applied throughout
- [Campaign 2](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md) and
  [the metadata-walk floor report](../reports/report-2026-08-23-metadata-walk-floor.md)
  — the ancestor-merge replacement behind the unpriced union
- Source verified at `64398b7`: `index.rs` (`children`, `contribution`,
  `Index::provenance`), `classify.rs` (`derive_ext`, `ext_bucket`,
  `classify_path_with_prefix`, `RULES_BY_EXTENSION`)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
