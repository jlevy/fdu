# Feature: Progressive Results — Traversal Order, Streaming Sessions, and Instant Warm Opens

**Date:** 2026-08-11

**Author:** fdu project

**Status:** Draft

## Overview

Make fdu usable by a consumer that needs answers *while* the walk runs and *instantly*
on the second open, rather than only after a complete scan.
This is the architectural work an interactive browser over a multi-million-entry tree
requires, and it is independent of the FSEvents journal: everything here lands on every
platform, helps the first scan as much as the second, and is a precondition for the
journal being worth anything rather than a consequence of it.

Four changes, in dependency order:

1. **Traversal order as a policy** with breadth-first by default, so partial results
   mean something. *Landed with this plan.*
2. **A streaming session API** so a caller can start a scan, get control back, and read
   growing roll-ups with per-path completeness while the walk proceeds.
3. **Persisted roll-ups and lazy open** so the second open paints from the cache in
   milliseconds instead of materialising millions of records first.
4. **Provenance on every value**, so a browser can paint slightly stale numbers
   immediately, mark them approximate, and clear the marks as verification converges —
   which is the difference between a UI that shows something in 50 ms and one that shows
   nothing for eleven seconds.

The motivating measurements, all on this host: a home folder of **4,366,510 files and
1,016,449 directories, 224 GiB** walks cold in **791 seconds**, and a warm snapshot of
that size would take roughly **11 seconds** just to load at today’s ~2 µs/record.
Neither number is compatible with an interactive first paint, and no amount of walking
faster fixes either one on its own.

## Goals

- A consumer reading the index mid-scan sees monotonically improving answers, never a
  confidently wrong ranking
- Traversal order is chosen by the caller’s contract and provably cannot change the
  resulting index or invalidate a cache
- Starting a scan returns control immediately; roll-ups and per-path completeness are
  readable throughout
- A warm open paints the top of a multi-million-entry tree without materialising it
- Every value a consumer reads carries its own confidence, so approximate answers can be
  shown immediately and labelled honestly, then converge to verified
- Every mechanism degrades to the current behaviour rather than replacing it

## Non-Goals

- The FSEvents journal, which is
  [its own plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md).
  This plan is what makes that one worth having; it does not depend on it.
- Content-tier reducers (line counts, hashes).
  The derived-data layer’s shape is fixed by the composable-CLI plan; nothing here ships
  an analyzer.
- A TUI or any UI. This is engine and API surface for consumers that have their own.
- Changing what a one-shot CLI prints.
  Its contract — verify, then print — is unchanged.

## Background

Two facts, both measured, set the shape of the problem.

**Sizes are recursive, so only a cache can answer quickly.** Listing a home folder’s
immediate children takes about a millisecond, but knowing one of them is 53 GiB means
having walked all of it.
A first paint therefore either shows skeletons or reads something that already knows —
there is no third option.

**Partial results are only useful if the order is right.** fdu already streams
internally: the parallel producer emits observation batches as it walks and
`merge_upward` keeps every ancestor’s roll-up current, so `index.rollup(path)` is
already a valid lower bound mid-scan.
But every traversal loop in the crate was a LIFO stack, so those lower bounds arrived
depth-first — one child of the root complete at its final size while its siblings read
zero. A consumer ranking by size mid-scan ranked confidently and wrongly.

metabrowser’s Python walker independently derived the fix years of engine work earlier:
it queues breadth-first to a first-render depth “so a request landing ~500 ms into boot
finds the visible part of the tree already populated”.
It also derived post-order finalize by refcounting children — the same design fdu’s
roll-ups use — and a `max_files` cap of 500,000 that exists only because a Python walker
cannot go further. The full analysis of that comparison and of how the four consumers
differ is in the
[interactive browser research](../../research/research-2026-08-11-interactive-browser-use-case.md).

## Design

### The two data-structure principles

Everything below follows from treating two properties as invariants of the data
structures rather than features of any one surface:

- **Delta-friendly** — the existing contract: every structure is modified only through
  typed, clocked deltas, which is what keeps the in-memory index, the snapshot, and the
  change feed from drifting apart.
- **Partial-friendly** — its peer, made explicit by this plan: every structure is *valid
  and useful while incomplete*, provided the boundary of incompleteness is knowable.
  A partially walked tree is a real answer — every roll-up a correct lower bound, every
  unvisited subtree identifiable — not a broken invariant awaiting repair.
  Queries, serialization, sessions, and reducers must all accept a partial structure as
  a first-class input, and anything that cannot must say so in its signature rather than
  misbehave.

The two compose: a delta stream applied to a partial structure yields another valid
partial structure. That composition is what a progressive UI *is*, and it is why neither
property can be bolted on at the edges.

### 1. Traversal order as a policy (landed)

```rust
pub enum ScanOrder { BreadthFirst, DepthFirst }   // default: BreadthFirst
```

Both orders visit every entry exactly once and leave an identical index behind, so the
policy sits beside `threads` and `batch_size` as operational, and stays out of
`ScanScope` where it could otherwise invalidate a snapshot.
The serial walk, the revalidation sweep, and subtree reconciliation take from the front
or the back of one `VecDeque` according to the policy.
The parallel worker queue is region-scheduled under `BreadthFirst` (exp-013): work is
bucketed by top-level subtree, each free worker is handed a different bucket
round-robin, and within a bucket the order is LIFO. A global FIFO ordered the queue but
not the claims, so workers clustered in one subtree; regions spread them by
construction. There is still no second walker and no barrier anywhere.

Measured properly (exp-012: 60,067-entry tree, 16 interleaved paired trials per job),
breadth-first costs **no measurable wall time and a little memory**. Wall on
`cold-scan-index` is −0.58% with a 95% interval of [−2.50%, +1.20%]; the walk-only and
warm-revalidation intervals straddle zero too, so the supported claim is “not measurably
different”, not “free”.
Peak RSS did rise in that first implementation, with intervals clear of zero: +1.51%
[+0.85%, +2.88%] on `cold-scan-index` (about 34.7 MB to 35.4 MB), +3.66% on
`cold-scan-producer`, +1.17% on `warm-revalidate`, alongside +2.50% producer CPU.

Those costs no longer apply to the parallel walk.
exp-013 replaced the global FIFO with a region scheduler, and exp-014 measured the
shipped default against `DepthFirst` directly: `cold-scan-producer` wall −3.04%
[−5.99%, −0.96%] and `cold-scan-index` peak RSS −1.76% [−2.63%, −0.74%] — breadth-first
is now the cheaper of the two there.

The warm sweep is the exception and is a known gap: `reconcile` walks with the serial
`take_next`, so region scheduling never reached it and breadth-first costs +2.70%
[+1.55%, +3.37%] on `warm-revalidate`, for an orientation benefit a one-shot CLI never
reads.

That cost is accepted rather than outstanding.
It sits below the project’s own 3% bar for changes worth added complexity, and the queue
ahead of it has proved worth far more.
The adaptive worker pool (`fdu-tt2j`) improves a reproducible 720k cold-index run 5.31%
[−8.37%, −2.70%] while retaining the 120k boundary result.
The macOS bulk-metadata backend then composes with that pool: exp-022 improves the same
720k cold-index job another 30.13% [−32.19%, −25.11%] and producer wall 41.60%, while
the 60k jobs improve 5.22% and 9.25%. Against the original pre-optimization build, the
complete accepted stack is 52.84% faster for cold index and 58.29% faster for
producer-only scans (exp-027). The earlier roughly-2× cold private-tree observation
remains context; exp-027 is now the claim-grade reproduction.
The same audited bulk reader now serves full macOS reconciliation as well.
exp-026 improves warm-open wall 18.97% on the 60k subject and 34.39% on the 720k
subject, with large-tree system CPU down 53.97% and RSS neutral.
Cumulatively, warm-open wall is now 34.78% below the original build (exp-027).
Reconciliation stays serial, so this does not change the breadth-first partial-result
contract; it removes filesystem work from the sound cache fallback and composes with
future FSEvents scoping.
Persisted roll-ups with lazy open (`fdu-1vd0`) turn an 11-second warm load into a first
paint. Tracked at low priority as `fdu-v71x` so the decision stays visible.

An earlier six-sample median comparison suggested ~8%, and that figure was quoted here
before it had been through the accept rule.
It did not survive it — and the correction then overshot in the other direction, calling
the change free, because the harness printed metrics failing the one-sided accept rule
as “n.s.” regardless of which way they pointed.
The correction matters beyond the number: it removes the only argument for giving the
one-shot CLI a different default, so breadth-first is simply the default everywhere and
`DepthFirst` exists for callers with a specific reason — a memory-constrained walk of a
very wide tree — rather than as a performance escape hatch.

Three tests pin the contract: identical engine digests across both orders and several
worker counts, non-decreasing directory depth in emission order under breadth-first, and
scope equality between the two orders.

### 2. Streaming session API

The engine already produces what a browser needs; what is missing is a way to hold it
while it is being produced.

```rust
let session = fdu::Session::start(root, config)?;   // returns immediately
loop {
    let view = session.report(&query)?;             // roll-ups + freshness, any time
    render(view);
    if session.is_complete() { break; }
}
```

- **Reads during writes** already exist: `IndexHandle` serves readers while a producer
  applies short writes, and that is exactly the shape a session needs.
- **Completeness is already modelled**: `Freshness` distinguishes `Fresh` from
  `Partial`, and `freshness_marks` is keyed by path, so a consumer can ask whether a
  directory’s total is final or still growing and render “calculating…” accordingly.
- **The contract is monotonicity**: under `BreadthFirst`, every roll-up a consumer
  observes is a lower bound that only increases until the subtree completes.
  That is the property the API documents and tests, and it is what makes a progress UI
  honest.
- **Python mirrors it**, since metabrowser is the driving consumer and a subprocess
  boundary would defeat the point.

Cancellation and bounded memory matter here in a way they do not for a one-shot: a
session must stop promptly when a user navigates away, and a browser should be able to
cap resident entries.
Both are session-level concerns, not engine ones.

### 3. Persisted roll-ups and lazy open

The second open is where a browser should feel instant, and today it cannot: loading a
5.4M-entry snapshot costs ~11 seconds because every record is replayed through apply and
every reducer is recomputed.

This is the existing backlog — H33 (persist per-directory reducer state), H34 (bulk
arena load), H16 (answer from roll-ups without materialising the index), H35 (block
format with a tail index and lazy decompression, bead `fdu-1vd0`) — whose priority this
use case sharply raises.
The browser needs exactly what they provide: read the header and the top-level directory
records, paint, and load deeper blocks only as the user navigates.
A browser displays a few hundred entries at a time and never needs millions resident.

### 4. Provenance: every value says where it came from

The browser’s strongest requirement is one the CLI never had: **a slightly stale number
is far more useful than no number.** Reopening a large folder should paint immediately
from the cache, mark every value as approximate, and clear those marks as verification
confirms them — converging visibly rather than blocking.

This looks like it collides with the project’s hardest rule, “the cache may never
silently lie”. It does not, and the distinction is the word *silently*. Serving a cached
number **labelled with where it came from and when** is the honest version of exactly
this; serving it as though it were freshly observed is the thing the rule forbids.
The original research already staked this out — fast-but-wrong is a non-goal,
fast-and-labelled is a feature — and this generalises it from a one-off mode into a
property every value carries.

#### The gap in what fdu models today

`Freshness` is per-path and already distinguishes `Fresh`, `Reconciling`, `Stale` and
`Partial`. But an index loaded from a snapshot reports **`Fresh`**, because the snapshot
was complete when it was *written*. For a CLI that revalidates before printing, that is
harmless. For a browser that paints on load it is precisely backwards: nothing has been
checked since the file was read, and the one signal the UI needs is missing.
`Freshness` also answers for the *run*, not for the value, so it cannot say that this
directory is confirmed while that one is not.

#### Three orthogonal facts, not one enum

Provenance answers “where did this come from, when, and is it finished” — three
independent questions that a single enum would tangle:

```rust
pub struct Provenance {
    /// Where the value came from.
    pub source: Source,
    /// When the underlying filesystem observation was made. For `Cached`, this is
    /// when the snapshot captured it — the "as of" a UI shows.
    pub observed_at: SystemTime,
    /// How settled the value is. An enum rather than a boolean, because "not
    /// complete" already means at least two different things a consumer may need to
    /// distinguish, and more are foreseeable.
    pub status: Status,
}

#[non_exhaustive]
pub enum Status {
    /// The value covers everything beneath this path.
    Complete,
    /// A walk beneath this path is still running; the value is a lower bound that
    /// only grows.
    Partial,
}

pub enum Source {
    /// Observed from the filesystem by this process.
    Scanned,
    /// Loaded from the snapshot and re-verified by stat this session.
    Revalidated,
    /// Loaded from the snapshot; the change journal reported nothing touching this
    /// subtree since the cursor. Deliberately weaker than `Revalidated`: the Phase 0
    /// spike found FSEvents can omit history without raising a flag, which is what the
    /// periodic full sweep bounds.
    JournalScoped,
    /// Loaded from the snapshot and not re-checked.
    Cached,
}
```

Splitting `complete` out of the source is what keeps two different UI affordances
distinct. An incomplete value is monotone and reads as “≥ 3.2 GB, counting” — a bar that
only fills. A complete `Cached` value is a point estimate that may move in either
direction and reads as “~3.2 GB, as of 2 minutes ago”.
Collapsed into one “not sure yet” state, a shrinking number would look like a bug.

#### Where provenance is stored, and why not per entry

The obvious implementation — a `Provenance` struct on every `Entry` — is the wrong one,
and the reason is the memory budget.
Entries already cost ~493 B each (measured: 2.65 GB for the 5.4M-entry home folder), the
frontier research wants that near 50 B, and one of its standing items is *removing* the
64-byte `RollUp` from the ~88% of entries that are files.
Adding 24 bytes per entry to every one of 5.4M would push in the wrong direction for a
value that is nearly always the same across millions of neighbours.

So provenance is stored where it varies, and derived where it does not:

- **Per entry: one byte.** A `Source` discriminant, which fits in existing padding
  beside `kind` and `ext_id`. That is the only fact that genuinely differs entry to
  entry — this file was re-stat’d, that one came from the snapshot and was not.
- **Per index: the timestamps.** `observed_at` for a scanned entry is when this
  session’s scan ran; for a cached entry it is when the snapshot was captured.
  Both are properties of the *index*, held once, not of each of five million entries.
  An entry’s `observed_at` is looked up from its source, not stored beside it.
- **Per directory: the composed value.** `RollUp` gains the worst-source, oldest-
  observation and worst-status of its subtree — three small fields on a struct that
  exists once per directory, which is where the interesting composition lives anyway.
  This is also why the file-side `RollUp` removal and this feature must be designed
  together rather than in either order.

`Provenance` is therefore a *view type*: constructed on demand by
`Index::provenance(path)` and by the query layer, never a field.
Consumers get the whole struct; the index stores a byte and two clocks.

#### Provenance rolls up, and that is nearly free

A directory’s total is only as trustworthy as its least trustworthy descendant, so all
three facts compose upward by monotone operations: `source` takes the weakest,
`observed_at` the oldest, `status` the worst in its ordering.
That is the same shape as every other roll-up fdu maintains, so it reuses `merge_upward`
rather than adding machinery.
A directory whose whole subtree is verified reports `Revalidated` and a UI drops the
indicator for that row; one unchecked file deep inside keeps its ancestors honest all
the way to the root, with the oldest `observed_at` explaining how stale the worst of it
is.

#### Convergence has to be observable, not polled

Clearing an indicator requires knowing *when* a value became trustworthy, so the session
emits provenance transitions per path alongside the value changes it already produces.
Two outcomes matter and both must be reported: verification that **confirms** a cached
value (clear the mark, no visual jump) and verification that **corrects** it (update and
clear, and the UI may want to draw attention).
A consumer that only learns about corrections cannot tell “still checking” from “checked
and fine”.

#### Verification should follow the user’s attention

The browser knows which directories are on screen; fdu does not.
So the session takes a hint:

```rust
session.prioritize(&path);   // the user just opened this — verify it next
```

Verification is otherwise breadth-first like the walk, but a prioritised subtree jumps
the queue. This is what makes convergence feel immediate rather than merely fast: the
handful of rows a user is actually looking at confirm in milliseconds even while
millions of entries behind them are still unverified.
Without it, a uniform sweep spends most of its effort on rows nobody is reading.

#### It is a library property first, and a CLI feature because of that

Provenance belongs on the value, not in a rendering layer, so every consumer gets it
from the same place: `Report` rows carry it, all four output formats serialise it, and
Python exposes the same struct.
The CLI then displays it rather than inventing it, which is the project’s standing rule
that the CLI invents nothing.

For a one-shot command the common case stays quiet: it verifies before printing, so
every row is `Scanned` or `Revalidated` and there is nothing to annotate.
Provenance becomes visible exactly when it should — under `--cache only`, where every
row is `Cached` and the header says as of when; under `--allow-partial`, where
incomplete subtrees are marked; and in any future progress mode.
The same data that lets a browser draw a small “approximate” glyph lets the CLI print an
honest “as of” line, and lets an agent consuming JSON decide whether a number is good
enough for what it is about to do.

#### What this makes the journal worth

This reframes FSEvents more sharply than the earlier analysis did.
Without a journal, every cached row is equally suspect on open, and clearing the
indicators means verifying all of them — minutes at home-folder scale.
With one, a ~200 ms replay names the few directories that could have changed, so
**almost every row can move from `Cached` to `JournalScoped` at once** and only a
handful keep their marks.
The UI goes from entirely-approximate to almost-entirely-confirmed in a fraction of a
second, and stat verification is scoped to what the journal named plus whatever the user
is looking at.

That is the journal’s real product value for this use case: not that it makes
verification cheaper, but that it makes *most of the display trustworthy immediately*
while honestly flagging the rest.

### How these compose

|  | first open (cold) | second open (warm) |
| --- | --- | --- |
| first paint | skeletons immediately; breadth-first fills top-level totals within the first seconds | persisted roll-ups read from the snapshot header — milliseconds |
| convergence | the walk continues; totals grow monotonically | verify what changed (journal where available, else a sweep or rescan by the adaptive policy) |
| navigating deeper | already walked, or walking | lazy block load per directory |
| what the UI shows | lower bounds that only grow, marked “counting” | cached values marked “approximate”, marks clearing as confirmation arrives |

The journal appears in one cell of one row, which is the point of separating these
plans: everything else here works without it, on every platform.

## Implementation Plan

### Phase 1: Order and session

- [x] `ScanOrder` policy, breadth-first default, all four traversal loops, tests for
  index equality, depth monotonicity, and scope stability
- [x] `--order` on the probe.
  The one-shot CLI does **not** default to `DepthFirst`: the 8% that would have
  justified it did not survive the accept rule (exp-012), so breadth-first is the
  default everywhere and `DepthFirst` is for callers with a specific memory or locality
  reason
- [x] Adaptive cold-scan workers: begin at the warm-small knee, calibrate the first 16k
  entries from existing chunk attribution, and spawn bounded reserves only for slow
  filesystem service (exp-015–021, `fdu-tt2j`)
- [x] macOS bulk metadata: replace directory enumeration plus one metadata syscall per
  entry with fail-closed `getattrlistbulk`, retaining the portable backend elsewhere and
  at mount/firmlink boundaries (exp-022)
- [ ] `Session`: start/read/complete/cancel over `IndexHandle`, with documented
  monotonicity and per-path freshness; bounded-memory option
- [ ] Python `Session` mirroring the Rust surface
- [ ] Loop experiment: time-to-useful-top-level-ranking, breadth-first against
  depth-first, on a home-folder-scale tree — the metric this plan exists to move

### Phase 2: Provenance and convergence

- [ ] `Provenance` per value, composed through the existing reducer path by weakest
  source / oldest observation / worst status - note these aggregates are **not
  invertible** under deletion or revalidation, so the design must specify the recompute
  path (`fdu-fka6`); a snapshot-loaded index reports `Cached`, not `Fresh`
- [ ] Provenance transitions on the session’s change stream, reporting confirmations as
  well as corrections
- [ ] `session.prioritize(path)` so verification follows the user’s attention
- [ ] Surface confidence in `Report` rows and in every output format, per the
  composable-CLI plan’s rule that no policy may silently lie

### Phase 3: Instant warm open

- [ ] Persist per-directory reducer state in the snapshot (H33)
- [ ] Bulk arena load, no per-record observation replay (H34)
- [ ] Answer a query from persisted roll-ups without materialising the index (H16)
- [ ] Block format with tail index and lazy decompression (H35, `fdu-1vd0`)

## Testing Strategy

Order equivalence is the load-bearing correctness property and is already tested:
identical engine digests across orders and worker counts.
Monotonicity gets a property test — sample roll-ups repeatedly during a scan of a
fixture tree and assert no observed total ever decreases.
Session tests cover start/cancel determinism and that a partial read is labelled
partial. The real-tree harness gains a time-to-useful-ranking job, since a plan about
*when* answers arrive cannot be validated by a benchmark that only measures when they
finish.

## Open Questions

- Peak queue width for breadth-first on a tree with a very wide level (a home folder has
  ~1M directories); measured at 60k, unmeasured at scale, and the answer decides whether
  a hybrid — breadth-first to a first-render depth, then depth-first below, as
  metabrowser does — is worth the complexity.
- Whether the one-shot CLI should really default to `DepthFirst`, given that a future
  `--progress` would want the opposite.
- How a session should bound memory: entry cap, depth cap, or eviction.

## References

- [Interactive browser research](../../research/research-2026-08-11-interactive-browser-use-case.md)
  — the four-consumer comparison and the measurements behind this plan
- [Performance frontier research](../../research/research-2026-08-10-performance-frontier.md)
  — H16/H33/H34/H35, the verification tiers, and the cost model
- [Composable CLI and query surface plan](plan-2026-08-10-fdu-composable-cli-surface.md)
  — `Query`/`Report`, which a session returns
- [FSEvents-scoped revalidation plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md)
  — the convergence half, deliberately separate

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
