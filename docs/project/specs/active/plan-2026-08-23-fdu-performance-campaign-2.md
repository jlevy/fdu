# Feature: Performance Campaign 2 — Spend the Loop Where the Floor Says It Pays

**Date:** 2026-08-23

**Author:** fdu project

**Status:** Active. This plan owns the current prioritization; the queue orderings in
[the structural review](../../research/research-2026-08-14-structural-performance-review.md)
and
[the consumer structural-headroom review](../../research/research-2026-08-15-consumer-structural-headroom.md)
are inputs it supersedes.

## Overview

Campaign 1 ran the loop sixty-four times and made fdu roughly twice as fast, one
relative verdict at a time.
[The metadata-walk floor report](../../reports/report-2026-08-23-metadata-walk-floor.md)
then measured the denominator those verdicts never had: what the machine charges for the
work fdu cannot avoid, per tier, per subject.

Campaign 2 is the same loop pointed by that denominator.
Effort goes where the measured distance to the floor is large, stops where it is small,
and every hypothesis states up front which gap it closes and how much of it.
The campaign also has something campaign 1 never defined: a way to end.
A tier in a regime is *closed* when it reaches its floor threshold or when two
consecutive re-screens produce no hypothesis whose mechanism names at least the accept
gate of remaining headroom — and a closed tier is a result, recorded like any other.

## Goals

- Every open hypothesis carries a tier, a subject class, and the share of that tier’s
  floor gap it targets, so the queue can be ordered by arithmetic rather than appetite
- The index tier lands within 1.4× of the parallel syscall floor on the primary Linux
  subject, via one structural experiment rather than seven gated increments
- The aggregate tier is declared closed for warm Linux once the same change lands,
  unless a re-screen finds a mechanism the floor arithmetic admits
- The content tier loses its two known redundancies: classification recomputed per open,
  and sidecar restoration costing 8× a metadata record
- The warm story is re-posed as adoption plus journal scoping, because the floor report
  reconfirmed what the cache cost model already measured: on a warm tree, a metadata
  snapshot cannot beat the walk it still owes
- Cold-regime and peer claims move from scouting to evidence: bare metal for the
  device-latency class, a quiet host and a real-tree subject set for the rankings

## Non-Goals

- Competing on ripgrep’s job.
  A search tool classifies from `d_type` and skips the metadata call that is 91% of this
  workload’s kernel cost; that gap is semantics, not implementation, and no fdu change
  should chase it
- Warm-Linux syscall micro-optimization.
  Batching is bounded at 9% and measured 6–8× slower through io_uring; the `getdents64`
  elision is under 1% of aggregate wall; narrower records were refuted four ways
  (H62–H65). These stay closed unless a re-screen names a new mechanism
- Windows performance.
  It builds and passes tests; the campaign claims nothing more
- Beating dumac on macOS beyond the measured tie, ahead of the `searchfs` spike that
  could actually move the per-directory floor
- Any change that trades output exactness, the delta contract’s arbitration of live
  mutations, or the fail-closed cache rules for speed

## Background

### What the denominator changed

Distance to the floor, warm Linux, from
[the floor report](../../reports/report-2026-08-23-metadata-walk-floor.md) and
[the headroom review](../../research/research-2026-08-15-consumer-structural-headroom.md):

| Tier | ×floor | The gap is |
| --- | --- | --- |
| Aggregate (`--view summary`) | **1.20** synthetic, **1.59** real (`/usr`) | per-entry name and path handling; the real-tree tax lands here |
| Index (default tree) | **2.68** on the 420k subject; ~4.3× on the 450k generated tree | the consumer representation: boxed entries, twice-stored names, per-op `PathBuf`s, per-entry ancestor merges, one serialized writer (~38% of elapsed), a 3.3× latency tail |
| Index, as `arena_spike` builds it | **1.06** | the measured ceiling for the representation change |
| Snapshot load (`--cache only`) | 0.88 µs/entry against 1.18 to walk and rebuild | re-derivation: roll-ups re-merged and extensions re-interned per record |
| Content, warm open | ~34% classification + 25 µs/file sidecar restore | recomputing what the index and sidecar already hold |

Three levers are settled for warm Linux and must not be re-run without a new mechanism:
syscall batching (9% ceiling, measured 6–8× slower twice), terminating-`getdents64`
elision (<1% of aggregate wall), and representation-narrowing on the summary path
(H62–H65, four refutations).

### The corpus rule is now arithmetic

A generated corpus hides about 15 points of fdu’s distance from the floor and moved a
peer ranking from 12–26% ahead to 12% behind.
The effect is concentrated in exactly the per-entry name and path handling this
campaign’s centerpiece deletes, which cuts both ways: generated-tree evidence
*understates* what the structural experiment is worth, and any accept decision scored
only on `gen_tree.py` is scored on the flattering half of the distribution.
Campaign 2 therefore runs its accept measurements with at least one nominated real tree
in the paired set, and treats generated subjects as screening.

### Why the warm story is re-posed rather than optimized

Two measurements agree — the cache cost model on `/usr` and the floor report on the 420k
subject — that a warm metadata revalidation stats every entry regardless of what the
snapshot holds, so the snapshot is additive cost for a one-shot metadata query, and even
the no-scan `only` tier barely beats rebuilding.
That is a theorem about change information, not a defect to tune: directory mtimes do
not propagate child edits, so Ω(N) stats is the warm floor for sweep-based freshness.

What remains worth doing warm is therefore exactly two things.
**Adoption** — persist roll-ups and the interner (H92), make the format directly usable
(H78, then H35/H61) — so load approaches memcpy and a warm open costs the stat floor
plus nothing. **Journal scoping** — FSEvents replay per
[the fsevents plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md) — which is the
only mechanism on any platform that beats the stat floor itself, by making revalidation
O(changes). Everything between those two — a faster full-fidelity replay loader — is
spending effort inside a bound the physics already set.

## Design

### The consolidated loop

Campaign 2 adds three protocol elements to
[the performance loop](../../guides/performance-loop.md); the loop document carries the
normative wording.

1. **Floor-normalized accounting.** A hypothesis states its tier’s current ×floor and
   the share of the remaining gap its mechanism can name.
   On a tier at or under 1.25×, that statement is the justification to run at all.
   After any accepted change, the tier’s ×floor is re-measured (`make perf-floor`) and
   the queue re-screened — the shared-cost rule, now with a budget: hypotheses aimed at
   the same µs/entry divide one number, and the record has confirmed that competition
   three times (H13/H18, H74/`fdu-91ts`, H89/H86).
2. **Two tracks.** *Tuning* experiments change one variable and face the 3% gate.
   *Structural* experiments change a representation, run once as a composite (exp-022
   precedent), and are gated on the differential oracle plus pre-registered targets —
   including tail spread, because the index tier’s 3.3× spread is user-visible latency
   the median cannot see.
   Forcing a structural change through per-piece 3% gates measures conversion costs the
   end state deletes, and is how a measured ~4× would be rejected seven times.
3. **Real-subject accept evidence.** At least one nominated real tree in the paired set
   for any accept decision; generated corpora screen but do not decide.
   The nominated set and its fingerprints are loop infrastructure (`fdu-lk9u`).

### Termination

A tier in a regime is closed when either holds:

- its measured ×floor on the nominated real subjects is at or under its threshold —
  **1.25× aggregate, 1.4× index, stat-floor-bound warm open, sidecar-load-bound warm
  content** — or
- two consecutive post-landing re-screens produce no hypothesis whose named mechanism
  reaches 3% of the tier’s remaining gap.

Closure is recorded in the hypothesis registry with the re-screen that established it,
so the next campaign inherits a boundary instead of a feeling.

## Implementation Plan

Phases 0 and A–C are parallel where their beads say so; D and E follow their gates.

### Phase 0: Instruments (multipliers, each an afternoon-scale item)

- [x] `fdu-tyjx` — the aggregate-tier probe job with a tallies oracle.
  **Landed.** `perf_probe summary` drives the transient plan through `prepare_report`,
  and the `aggregate-summary` job measures it with `Job.oracle = "tallies"`. The blocker
  recorded against this bead — that the planner was `pub(crate)`, so an example could
  not reach the tier at all — had been removed in the meantime by `fdu-z7sp`, which
  exported `prepare_report` for an unrelated reason.
  The tier now has a `component_ns`: about 5 ms below wall on a 5,838-entry subject,
  which is most of what exp-043 and exp-044 were arguing over.
- [x] `fdu-lk9u` — nominate the real-tree subject set.
  **Landed.** `make perf-subjects` observes a host’s nominated trees and writes a
  redacted, committable document; `make perf-subjects-check` reports drift.
  A subject may decide an accept when it is dense and at least 50,000 entries; a set may
  carry a ranking claim when its deciding subjects span three of four characters; below
  either bar it screens.
  The nominations file holds absolute paths and is gitignored, so what is committed says
  what a claim rests on without saying where it lives.
  Each host nominates its own, because `root_id` hashes a path — this repository now
  carries the Darwin/arm64 set: a 60k source checkout, the 175k rustup store and the
  159k sealed system frameworks decide, and a 5.8k cargo registry cache screens.
- [~] `fdu-33ri` — `make perf-floor`: run the floor spikes and the tiers across the
  nominated subjects and emit the ×floor table, the campaign’s scoreboard.
  **The Linux half landed** (2026-08-28,
  [the scoreboard report](../../reports/report-2026-08-28-linux-floor-scoreboard.md)):
  `make perf-floor SUBJECTS="label=/path"` builds `parfloor` and `arena_spike`, runs
  them interleaved with the aggregate and index tiers under the shared tally oracle, and
  emits the ×floor table with each tier marked against its termination threshold.
  It reproduced the by-hand report’s aggregate ratio on an independent host — 1.58× on
  `/usr` against the recorded 1.59× — which is the first time any ×floor number in this
  campaign has been measured twice, and it put the index tier at 2.82–2.93×. Two things
  it added that the by-hand session could not.
  `arena_spike` is **bimodal** on a 76k subject (a ~63 ms and a ~150 ms mode, selected
  by how much memory the preceding process churned; 1.09× and unimodal on a 48k one), so
  H86’s pre-registered peak-RSS and tail-spread targets need to name which mode they
  mean — and `p95/median` reads a calm 1.16 on that same distribution, because both
  humps are individually narrow, so the spread is what catches it.
  And the index tier’s spawn wall exceeds its own component timer by 149 ms against a
  110 ms component, so 57% of a spawn-timed index number is the probe’s own oracle
  rather than the engine, which sharpens `fdu-4xtm` from a tidiness item into a
  correctness one for this tier.
  **The macOS half remains, and remains a decision rather than effort.** `parfloor.c` —
  the denominator every ×floor threshold below is defined against — is Linux-only:
  `SYS_getdents64` and `statx` have no Darwin equivalents.
  A macOS scoreboard therefore needs either a `getattrlistbulk` port of the floor or a
  different floor set (`arena_spike` and `peerwalk` are portable; `dumac` is the
  practical anchor) with the regime difference recorded.
  That is a decision this plan should make deliberately rather than let a harness make
  by falling back.
- [ ] `fdu-5yjk` — extend scan diagnostics to the FullIndex plan, the plan users run and
  the one Phase B rebuilds; instrument before restructuring.
- [ ] `fdu-4xtm` — a `--no-oracle` probe mode and engine-phase counter scoping, so
  attribution runs stop counting the harness (the oracle is ~39% of probe instructions
  and 46% of its allocation events).
- [ ] `fdu-c65j` — adopt samply so Linux profiling stops depending on callgrind’s
  serialized world.
- [x] `fdu-mx1w` — a ledger job for the **default command**, `fdu <dir>`: scan, index,
  rendered tree, snapshot write.
  **Landed** as two jobs, `default-tree-first` and `default-tree`, over a
  `perf_probe default-tree` mode that drives `prepare_report` exactly as the command
  line does; exp-066 is the baseline on the 175k rustup store, and it shows the repeated
  run rewriting a 13.9 MB snapshot it never reads on every trial.
  None of the 66 artifacts before it measured this path.
  `cold-scan-index`, the proxy every cumulative checkpoint uses, is the probe’s walk
  plus index build and excludes both the render and the write — and the cache-layers
  plan already priced that write at roughly a third of a default run on `/usr`. Two
  defects found in the PR #38 review lived in exactly that blind spot and were judged on
  this job the night it landed: `fdu-2um8` (the cold-scan path rewrote an identical
  snapshot on every run; exp-067, `default-tree` −10.6%) and `fdu-n75m` part 1 (the
  rendered report was withheld until the write, its `F_FULLFSYNC` and the index teardown
  completed; exp-068, time to first byte −7.5% to −12.5%). Parts 2 and 3 of `fdu-n75m`
  are durability decisions and remain.
  The `fdu-default-tree` contract already exists; nothing has ever been recorded through
  it.

### The macOS agenda

The phases above are ordered by a Linux floor that does not exist on macOS, and this is
the host the project’s performance bar is set on.
[The strategy review](../../reports/report-2026-08-23-research-loop-strategy-review.md)
derives a macOS ordering from what is measurable here and states the case against each
item; the beads carry it under the `macos-agenda` label, and
[the runbook](../../guides/performance-loop-runbook.md) is how an unattended agent runs
one round of it.

- **Tier 1, unattended, in order:** `fdu-mx1w` (landed), `fdu-2um8` (skip the identical
  snapshot rewrite), `fdu-n75m` part 1 (flush the render before the join), `fdu-pdne`
  (PGO, screen only), `fdu-78q6` (sidecar restore, on the metabrowser clone).
- **Tier 2, instruments:** `fdu-9hdc` (a `getattrlistbulk` floor, so `fdu-33ri` can ship
  two scoreboards with the regime difference recorded), `fdu-4xtm`, `fdu-5yjk`,
  `fdu-0pzh` (measure only), and promoting `host_regime` into the artifact schema.
- **Tier 3, with a person:** `fdu-xde5` (H86), `fdu-jxhk` (its content-tier instance),
  `fdu-6kyn` (an `unsafe`-versus-dependency policy), `fdu-9716` (`searchfs`), `fdu-n75m`
  parts 2 and 3 (durability policy), the FSEvents journal (Phase D), and Phase E’s
  other-host work.

### Phase A: Constants with confirmed mechanisms (tuning track)

- [ ] `fdu-tk1b` — Linux cold thread policy (H76/H84). The adaptive unlock calibrated
  against APFS regimes never fires on Linux; guest-cold, sixteen workers beat four by
  32% at the floor itself and diskus’s 3×-cores default is the whole remaining
  scalar-class cold gap (~22%). Gated on `fdu-tyjx`; bare metal confirms before the
  constant ships as evidence.
- [ ] `fdu-pdne` — PGO screen (H93), one afternoon, release builds only if it clears.
- [ ] `fdu-6kyn` — hardware CRC32C behind runtime detection, the H88 follow-up.

### Phase B: The structural experiment (the campaign’s centerpiece)

- [ ] `fdu-xde5` — H86, run as **one** experiment: worker-local arena entries, one name
  arena, children as sorted arena slices, per-directory tallies, one bottom-up roll-up,
  batch-shaped observations, no per-entry channel crossing on the cold path.
  Children: `fdu-2ubt`, `fdu-prph`, `fdu-weey`, `fdu-fnfc`, `fdu-uv0s`; absorbs
  `fdu-0pzh` (the channel disappears or gets its bound) and the H89 headroom (interning
  from batch context); decides `fdu-refc` (S6’s tally representation) as part of the
  layout rather than separately.
  Floor-anchored targets, superseding the piecemeal predictions: index tier ≤1.4× floor
  and RSS ≤3× `arena_spike` on the primary subject; aggregate ≤1.25× on the real
  subjects (the name-handling tax is in scope); p95/median wall spread ≤1.5× where it is
  3.3× today — now a recorded field on both arms rather than a figure to re-derive by
  hand, and printed in the ledger beside any verdict whose tail reaches 1.5×;
  `assert_same_image` at every worker count; at least one real tree in the accept set.
  macOS validation follows the exp-054 pattern before any macOS number is claimed.
  The content tier has the same shape one level down — Phase C’s `fdu-cq7t` follow-on is
  H86’s instance there — and exp-065 sharpened what “at least one real tree” is for: a
  generated subject is depth-inflated and 22.6× sparse, which flatters exactly the
  per-file bookkeeping this deletes.
  A structural result measured only there would overstate by the width of exp-064’s two
  readings, 13.4 points against 2.4.
  [The evidence-scope plan](plan-2026-08-23-experiment-evidence-scope.md) turns that
  requirement into a checked `verdict.scope`, and this is the experiment it is most
  likely to bind first.
- [ ] Post-landing re-screens, in order: `fdu-h7sw` (H85 — expect the arena to have
  consumed it; screen against −20%, not 3%), `fdu-sk7v` (H66 — the directory-only
  transient tree may be moot at 1.06×), snapshot economics (below), and the tier
  scoreboard itself.

### Phase C: The content tier (independent of B)

**The structural item is the one to run.** H94 and H95 (exp-064) took the cheap form of
this tier’s problem and were confirmed; exp-065 then measured them against main 44
commits later on two subjects.
The warm win is real and transfers — −25.78% on `content-cache-hit` over dense real
source — and the cold `content-basic` figure recorded beside it does not: −13.40% on the
generated subject, −2.38% on a real one.
Plan Phase C against the warm number.

- [ ] `fdu-cq7t` follow-on — **the content-tier instance of H86**, and the reason this
  phase is not finished.
  Key roll-ups by `EntryId` and defer to one bottom-up pass, the shape that won −51.9%
  on snapshot load in `fdu-91ts`. H94 made the per-file ancestor walk cheap; this
  deletes it. Same argument as H86 on the index tier, same reason not to gate its pieces
  separately, and it should be measured on a dense subject because a sparse one flatters
  exactly this kind of change.
- [ ] `fdu-926e` — classification keyed by the interned `ExtId`, not recomputed per
  open. **Largely closed by exp-064, and its priority rested on a number that was
  wrong.** The bead claimed ~34% of a warm content open from a flat callgrind profile;
  the caller tree put classification at 11.11% inclusive, and H95’s indexed tiers have
  since taken −41.4% absolute off `classify_path_with_prefix`. What remains is the
  double classification in `apply_analysis`’s staleness guard, which is a
  public-contract change (`AnalysisCandidate` is constructible by callers) for a corner
  of 11%. Re-scope or close; do not carry it as P0.
- [ ] `fdu-78q6` — the sidecar restore path (H83): 25 µs/file against 3 µs for a
  metadata record, same re-derivation shape as the snapshot loader had; expect the same
  class of fix. Now the largest unexamined item on this tier after the structural one.
  First increment landed (exp-069, H102): the file map ordered by path bytes instead of
  components took `content-cache-hit` −31% and `content-query` −67% on a dense 52k-file
  checkout, to about 7.8 µs/file; the next is the `Path::hash` cost on the roll-up map.

### Phase D: The warm end-state (after B, because the representation decides the format)

- [ ] `fdu-yr23` — persist roll-ups and the interner (H92): load becomes adoption.
- [ ] `fdu-pdra` — the directly-usable snapshot format (H78), then H35 block checksums
  and H61 base-plus-overlay as its own gated steps.
- [ ] Re-pose snapshot economics on the new representation and record the answer: with
  load near-free, a warm open costs the stat floor — at which point the fsevents plan is
  the only remaining warm lever, and its Phase 0 spike gets scheduled on the macOS
  agenda rather than deferred by default.

### Phase E: Platforms, cold truth, and release evidence

- [ ] `fdu-lf3v` — one bare-metal Linux box; settles H28/H73 inode ordering, the cold
  thread constants, and the io_uring cold verdict as evidence rather than sign.
- [ ] `fdu-9716` — the `searchfs` spike (H77), the only mechanism under macOS’s
  per-directory open floor; standalone instrument first, dumac beside it.
- [ ] `fdu-druf` — H70’s quiet-host confirmation, unless `searchfs` obsoletes openers.
- [ ] `fdu-ow8y` — the quiet-host peer cell, plus the record spec’s Phases B–D, with
  peer rankings claimed only per `fdu-lk9u`’s real-subject rule.
- [ ] macOS robustness residue: `fdu-9tul` (threshold margin), `fdu-f6n7` (narrow
  attribute set) — re-screened after B, which may eat the second.

## Testing Strategy

Nothing changes about how a verdict is earned: paired, interleaved, oracle-checked
trials under [the loop](../../guides/performance-loop.md), recorded as schema-validated
artifacts, ledger and evidence page regenerated together.
What this plan adds is scope discipline — the floor statement and subject class are part
of the hypothesis, the structural track’s oracle and targets are pre-registered, and
`make perf-floor` re-derives the scoreboard after every landing so drift between the
strategy and the record is visible in review.

## Open Questions

- Where the index tier’s threshold truly sits: 1.4× is derived from `arena_spike`’s
  1.06× plus a priced contract band (arbitration, progressive publication, error
  provenance). If the landed contract costs more than 0.34×, the price gets written down
  and defended, not hidden — but the threshold may move with reasons.
- Whether `fdu-sk7v` (H66) survives Phase B at all, or the arena makes the transient
  tree the same plan with a smaller retention flag.
- Whether Linux ever gets a journal analog worth gating on (fanotify needs privilege;
  btrfs/ZFS diff is H47’s niche), or whether the resident watch session is the whole
  Linux answer to the stat floor.

## References

- [The metadata-walk floor report](../../reports/report-2026-08-23-metadata-walk-floor.md)
  — the denominator
- [The consumer structural-headroom review](../../research/research-2026-08-15-consumer-structural-headroom.md)
  — the ceiling measurement and the prior queue
- [The structural review](../../research/research-2026-08-14-structural-performance-review.md)
  — S1–S7
- [The performance loop](../../guides/performance-loop.md) — protocol and registry
- [The cache layers plan](../done/plan-2026-08-15-fdu-cache-layers-and-defaults.md) —
  the cost model this plan’s warm posture rests on
- [The fsevents plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md) — the journal
  rung
- [The record spec](plan-2026-08-15-fdu-performance-record-and-report.md) — evidence
  completion
- Beads: `fdu-xde5`, `fdu-tyjx`, `fdu-lk9u`, `fdu-33ri`, `fdu-4xtm`, `fdu-tk1b`,
  `fdu-926e`, `fdu-78q6`, `fdu-yr23`, `fdu-pdra`, `fdu-h7sw`, `fdu-sk7v`, `fdu-lf3v`,
  `fdu-9716`, `fdu-ow8y`, `fdu-mx1w`, `fdu-2um8`, `fdu-n75m`

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
