# Research: The Interactive Browser Use Case — Progressive Results, Warm Opens, and What FSEvents Is Actually For

**Date:** 2026-08-11

**Author:** fdu project

**Status:** Proposed

## Overview

fdu’s design so far has been driven by a one-shot CLI: run a command, print a verified
number, exit. An interactive browser over a multi-million-entry tree is a different
program with a different contract, and working it through changes the priority order of
the existing backlog rather than adding new items to it.

The concrete subject is metabrowser opening a home folder — measured on this host at
**4,366,510 files and 1,016,449 directories, 224 GiB**, which fdu currently walks cold
in 791 seconds.

The conclusion in one line: **the cache and the walker serve the browser’s two hard
moments, and FSEvents serves a third one that only exists because a browser is allowed
to be honestly stale in a way a one-shot CLI is not.** All three matter, in a definite
order, and the order is not the one the CLI’s numbers implied.

## What metabrowser already proves

Its `inventory.py` has independently derived most of fdu’s architecture in Python, which
is strong evidence the design is right and equally strong evidence about where the
engine boundary belongs:

| metabrowser mechanism | fdu equivalent |
| --- | --- |
| BFS queueing to `DEFAULT_FIRST_RENDER_DEPTH`, “so a request landing ~500 ms into boot finds the visible part of the tree already populated” | `ScanOrder`, breadth-first by default — **landed on this branch**, though strict only with a single worker (see below) |
| Post-order finalize via `pending_children_count` refcounting | `merge_upward` roll-ups — the same atomic-refcount design the original research took from dut |
| Generation counters so stale writes lose, “race-free without locks” | `EntryId` generation plus revision counters, the ABA guard |
| `max_files` 500,000 and `max_depth` 20, flipping status to `truncated` | no equivalent today; whether a session needs bounded memory and a `Truncated` status is still open in the progressive-results plan |

That last row is the tell.
The caps are not a product decision; they are a Python walker’s speed limit made visible
to the user. fdu walks 5.4M entries today and would walk them roughly twice as fast with
the adaptive pool (`fdu-tt2j`), so adopting fdu removes the reason the caps exist.

## The three moments, and what serves each

A browser opening a huge folder has three deadlines, not one.
Conflating them is what makes “cache or FSEvents?”
look like a single question.

### T0 — first paint, target under 100 ms

The user needs the top ~50 directories with *something* in the size column.

Sizes are recursive, so nothing in the filesystem can supply them quickly: listing `~`'s
immediate children takes about a millisecond, but knowing `Library/` is 53 GiB requires
having walked all of it.
So at T0 there are exactly two possibilities — render skeletons, or read a cache that
already knows.

- **Cold**: skeletons, then progressive fill.
  Traversal order is the whole game (below).
- **Warm**: the snapshot already holds per-directory roll-ups.
  First paint is a read.
- **FSEvents contributes nothing here.** The journal records *what changed*; it has
  never held a size.

**Winner: the cache — and specifically a format that can answer for the top level
without materializing 5.4M records.** At today’s ~2 µs/record a full load of this tree
is ~11 seconds, which is not a first paint.
This is precisely backlog items H33 (persist per-directory reducer state), H34 (bulk
load), H16 (answer from roll-ups without building the index) and H35 (block format with
a tail index and lazy decompression).
The browser use case is the strongest argument for them yet, and it should move them up
the queue.

### T1 — trustworthy display, target about a second

The user needs the displayed numbers to be true, or honestly labeled.

- **Full sweep**: 5.4M stats.
  Minutes. Unaffordable on every open.
- **Rescan**: the same minutes, for the same reason.
- **FSEvents replay**: ~200 ms fixed, naming only the directories that changed since the
  last open, after which only those are re-verified.

**Winner: FSEvents, decisively, and this is the answer to whether it is worth
building.** It is the only mechanism that makes “verify on open” affordable at this
scale.

Note what changed relative to the CLI analysis.
For a one-shot command the journal looked marginal, because a CLI must verify *before*
it prints and the verification is on the critical path.
A browser may legitimately paint cached values labeled “as of two minutes ago,
refreshing”, and converge asynchronously.
That difference in contract moves the replay off the critical path entirely, where its
fixed ~200 ms cost stops competing with first paint and starts competing with *minutes*.

**So FSEvents is worth more to metabrowser than to the fdu CLI** — and, unlike the CLI
case, it is worth having even for modest trees, because a long-lived browser re-opens
and refreshes constantly and each refresh is a background 200 ms rather than a
foreground rescan.

### T2 — depth on demand, as the user navigates

The user opens `Library/`, then `Caches/`, then one directory inside it.

A browser displays a few hundred entries at a time and never needs 5.4M in memory at
once.
Warm, this is the lazy block format again — load the blocks for the directory being
viewed. Cold, it is the background walk continuing to fill in.

**Winner: the lazy format, plus fast walking as the fallback.**

## Every use case, and what each actually needs

The browser is one of four consumers, and they disagree about almost everything except
correctness. Setting them side by side is what makes the design decisions fall out
instead of being argued.

|  | **One-shot CLI** (`fdu ~`) | **Interactive browser** | **Whole-drive audit** | **Resident watch** |
| --- | --- | --- | --- | --- |
| What is being optimised | time to *final* answer | time to *first useful* answer | time to complete, at 5M+ | steady-state per-event cost |
| Partial results | invisible — nothing prints until the end | **the entire product** | progress reporting | n/a, index stays fresh |
| Traversal order | irrelevant to output; prefers locality | **wants shallow-first** | irrelevant | n/a |
| Tolerates labeled staleness | **no** — must verify before printing | **yes** — “as of 2 min ago, refreshing” | yes, with a caveat line | no, it is live |
| Cache value | negative below ~150k entries | **decisive** — only it can paint at T0 | decisive | it *is* the cache |
| FSEvents value | marginal; on the critical path | **high** — off the critical path | high | already covered by the live watcher |
| Dominant cost | the walk | the *first paint*, then the walk | I/O latency; in-flight depth | escalation rarity |
| Memory | bounded by tree | bounded by tree; also wants lazy load | 2.65 GB at 5.4M — matters | steady-state resident |

Three things follow that were not obvious when only the CLI existed.

**Staleness tolerance is the real axis, not tree size.** A one-shot command must verify
before it prints, so every verification cost lands on the critical path and the cache
must beat a rescan outright to be worth reading.
A browser may paint cached values with an honest label and converge behind the user’s
back, so the same cache and the same journal are worth far more to it.
This single difference explains why FSEvents looked marginal in the CLI analysis and
looks decisive here, without either conclusion being wrong.

**Traversal order is a consumer contract, not an engine detail.** It cannot change the
final index — proven by test, both orders produce identical engine digests — so it is
purely a question of what a consumer sees *while* the walk runs.
A CLI sees nothing, so it should take whichever order is cheapest.
A browser sees everything, so it needs shallow-first.
Hardcoding either one serves one consumer and taxes the other.

**Progressive results and caching are complements, not substitutes.** The cache answers
the second open; progressive results answer the first, and every cache miss, and every
subtree the journal says changed.
A design with only one of them has a hole exactly where the other would have been.

## What order costs, measured

Breadth-first is now the default, and the trade was measured rather than assumed
(exp-012: 60,067-entry tree, sixteen interleaved paired trials per job):

Wall time, breadth-first versus depth-first:

| job | median change | 95% interval | evidence |
| --- | ---: | --- | --- |
| `cold-scan-index` | −0.58% | [−2.50%, +1.20%] | unclear |
| `cold-scan-producer` | +1.50% | [−3.50%, +3.13%] | unclear |
| `warm-revalidate` | +0.03% | [−3.83%, +2.87%] | unclear |

**Breadth-first cost no measurable wall time, and — as first built — a little memory.**
Every wall-time interval straddles zero, so the honest statement is "not measurably
different", not "free". The resources did move, with intervals clear of zero:

| job | peak RSS | 95% interval |
| --- | ---: | --- |
| `cold-scan-index` | +1.51% | [+0.85%, +2.88%] |
| `cold-scan-producer` | +3.66% | [+2.47%, +4.72%] |
| `warm-revalidate` | +1.17% | [+0.36%, +3.77%] |

On the primary job that was about 34.7 MB to 35.4 MB; producer CPU rose +2.50%
[+1.48%, +4.04%]. The engine digest is identical either way.

**Those costs are historical, not current.** They measured a single global FIFO, since
replaced by region scheduling (exp-013). Measured on the shipped code against
depth-first (exp-014, twenty interleaved paired trials, same binary both arms):

| job | scheduler | wall | 95% interval |
| --- | --- | ---: | --- |
| `cold-scan-producer` | region | −3.04% | [−5.99%, −0.96%] |
| `cold-scan-index` | region | +0.50% | [−1.39%, +1.98%] |
| `warm-revalidate` | serial FIFO | **+2.70%** | [+1.55%, +3.37%] |

Peak RSS on `cold-scan-index` is −1.76% [−2.63%, −0.74%]. So where region scheduling
reaches, breadth-first is now *cheaper* than depth-first on memory and on producer
throughput, having cost memory in exp-012.

**Where it does not reach, it still costs.** `reconcile` walks with the serial
`take_next` rather than the shared queue, so the warm sweep runs the same front-popping
FIFO exp-013 replaced elsewhere and pays +2.70% for it — while a one-shot CLI reads none
of the orientation benefit, because it prints only after reconciliation completes. That
asymmetry is tracked (`fdu-v71x`); the choice is between extending region scheduling to
the sweep and letting the sweep default to depth-first, which is closer to this
document's own position that traversal order is a consumer contract.

This corrects two earlier readings of the same change, in opposite directions.
A six-sample median comparison suggested breadth-first cost about 8% of wall time, and
that figure was written into the plan before it went through the accept rule; sixteen
paired trials say the wall-time difference is not measurable.
Then the corrected write-up overshot, calling the change "free" and "unchanged in
memory" — because the harness rendered every metric that failed the one-sided accept
rule as "n.s.", which made these RSS regressions read as statistical silence.
Both episodes are the same lesson from different sides: a number without its interval,
and an interval without its direction, are each how a project talks itself into a claim.

**The ordering benefit initially failed to survive parallelism, and the scheduler was
rebuilt so that it does.** Measured on the first implementation, breadth-first started 7
of twelve top-level subtrees by the halfway mark against depth-first's 6 with one
worker — and under the default worker count both landed at 7–8, the advantage gone. The
cause was that a global FIFO orders the *queue* while claims stay unordered: several
workers would grind through the same subtree while others sat untouched.

Breadth-first is now **region-scheduled** (exp-013). Work is bucketed by top-level
subtree, each free worker is handed a *different* bucket round-robin, and within a
bucket the order is LIFO so locality and spine-bounded memory come back. No barrier
exists anywhere: if only one region has work, every worker takes it.

On twelve branching subtrees, the *least advanced* top-level subtree a quarter of the
way through the walk holds **42 files at one worker and 33–37 at six** — against
depth-first's **0 and 6**, where perfectly even would be ~46. A deep portion of the tree
no longer delays the horizontal ones, at any worker count. Peak RSS fell −3.77%
[−5.18%, −2.99%] in the process, more than reversing what exp-012 paid, and wall time
did not move.

Worker affinity is the part worth remembering. Keeping a worker inside its region for
locality looks obviously right and is actively harmful: it pins each worker to one
subtree, so with twelve subtrees and six workers only six ever advanced, and
depth-first — whose four-directory claims happen to fan across the root's children —
spread *wider* than breadth-first did. Locality has to come from the size of a claim,
not from a worker refusing to leave.

Two further caveats so nobody over-reads even the corrected result.
This is one warm tree of 60k entries: the frontier width that could make breadth-first
expensive in memory only appears on a tree with a very wide level, and a home folder
with a million directories has not been measured for peak queue size.
And on a cold tree, ordering interacts with I/O locality differently than on a warm one,
where the metadata cache absorbs the difference.
Both belong in the loop before this is quoted as general.

## Progressive results: why traversal order is the whole trick

fdu is already closer to progressive rendering than it looks.
The parallel producer emits `Observation` batches to a sink as it walks; the index
applies them incrementally and `merge_upward` keeps every ancestor’s roll-up current.
So **`index.rollup(path)` is already a valid answer mid-scan** — a monotonically growing
lower bound. What is missing is not streaming; it is *order*, and an API to read while
writing.

Traversal order decides whether partial results are useful or actively misleading:

- **Depth-first** (`pending.pop()` on a LIFO stack, fdu's original and only behaviour
  before this branch) finishes a few subtrees completely and leaves the rest at zero.
  Mid-scan, `wrk/` reads a complete 77 GiB while `Library/` reads 0 GiB. A user sorting
  by size sees a confident, wrong ranking.
- **Breadth-first** grows top-level directories together.
  Numbers stay lower bounds, bars only fill, and relative ordering becomes meaningful
  earlier — which is exactly why metabrowser’s Python walker already queues
  breadth-first to a first render depth.

Note what supplies which property. **Monotonicity comes from the walk being additive,
not from the order** — a depth-first scan's totals only grow too. What the order changes
is *which* subtrees get to grow early, and therefore whether a mid-scan ranking compares
partial values against each other or against zeros. Conflating the two overstates what
choosing an order buys.

The first change was small: `DirectoryQueue` already existed and was shared, so taking
from the front rather than the back turned the walk breadth-first.
What that did **not** buy was any guarantee under the default worker count — the claims
are unordered even when the queue is not — and it made the pending set hold an entire
level of the tree.
Region scheduling (exp-013) fixed both: work is bucketed per top-level subtree and each
free worker takes a different bucket, so the frontier is bounded by the number of
regions plus a run of directories rather than by the widest level, and workers are
spread across subtrees by construction.

Depth-first stays available: it has better locality and lower memory. It is *not* the
right default for the one-shot CLI, as an earlier draft argued — that argument rested on
an 8% wall-time saving that did not survive the accept rule.
This is a scan *policy*, chosen by the caller’s contract, in the same way the cache
policy is.

## What this changes about the plan

Priority order for the browser use case, highest first.
Only the first item is new work; the rest are existing backlog items whose value this
use case sharply raises.

1. **A streaming session API with breadth-first order** (new).
   Start a scan, return immediately, let the caller read roll-ups and per-path
   completeness while it runs.
   `IndexHandle` already provides safe shared reads during writes and `Freshness`
   already distinguishes partial from fresh, so this is mostly surface, not machinery.
   Without it, metabrowser cannot use fdu at all for a cold open.
2. **Adaptive in-flight depth** (`fdu-tt2j`). Roughly 2× on cold large trees.
   Helps the first open, which no cache and no journal can help, and helps every
   platform.
3. **Persisted roll-ups and lazy open** (H33/H34/H16/H35, `fdu-1vd0`). Turns the second
   open from an 11-second load into a first paint.
   This is the single biggest lever for “opens instantly the second time”.
4. **FSEvents scoped revalidation** (the existing plan).
   Makes the background convergence affordable, which is what lets the browser trust
   what it painted.
5. **Lift the 500k/depth-20 caps** in metabrowser once the engine underneath is not the
   limiting factor.

## Answering the framing question directly

*Does FSEvents help here, or should we focus on caching and fast walking?*

Both, and the order is: **caching first, walking second, FSEvents third — but FSEvents
is worth more here than anywhere else in the product.**

- Caching (with a lazy format) is what makes the second open feel instant.
  Nothing else can, because sizes are recursive and only a cache remembers them.
- Fast walking is what makes the first open bearable, and it is the only thing that
  helps a first open at all.
- FSEvents is what makes the painted numbers *true* without paying for a sweep, and a
  browser’s tolerance for labeled staleness is exactly the property that lets it use the
  journal off the critical path.

The mistake would be treating them as alternatives.
They serve three different deadlines, and a browser over a home folder has all three.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
