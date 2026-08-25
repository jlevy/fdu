# Feature: Implementing the Interactive-Client Contract — Files, Functions, and How It Is Tested

**Date:** 2026-08-23

**Author:** fdu project

**Status:** In Progress

## Overview

[The interactive-client contract](plan-2026-08-23-fdu-interactive-client-integration.md)
says *what* to build and why, from a measured metabrowser deep-dive.
This spec says *where the code goes* and *how each piece is proven*, at file and
function level, so a bead can be picked up and implemented without re-deriving the map.

Reading the code to write it changed two things and revealed a third.

**Two of the hardest-looking pieces are already built and unused.** The concurrency
defect needs no new engine machinery: `IndexHandle` is already `Arc<RwLock<Index>>` with
short-write semantics, `reconcile_handle` already exists, and the Python binding simply
holds the wrong type.
And the tag/plane work needs no new snapshot field: `ScanScope` already carries
`ignore_rules_fingerprint` and `reducers_fingerprint`, both already serialized into the
snapshot header, the first sitting at `0` behind the comment *“No ignore rules exist
yet.”* The invalidation slots were carved out in advance.

**Streaming is already goldenable, and this repository already does it.**
`tests/golden/cli-watch.tryscript.md` goldens a live watch stream today.
It does not sleep and it does not sample wall time: a Node driver spawns the watcher,
performs one filesystem action, and blocks until the specific record proving *that*
action arrives before performing the next — causal sequencing, with a cursor so a record
from step N cannot satisfy step N+1. The stability vocabulary a progressive trace needs
already exists too, `[CLOCK]` among the named patterns.
So the testing plan below extends a working mechanism rather than inventing one.

**The third thing is the gap that makes the rest testable.** A watch stream can be
sequenced causally because the test causes each change.
A *scan* trace cannot: the tree is static and the walk’s emission order is decided by
worker scheduling.
Its determinism has to come from injection instead — fixing the worker
count and the traversal order — and **both knobs already exist on `ScanConfig` and
neither is reachable from any surface**. `threads: Option<usize>` sits beside
`order: ScanOrder` at `scan.rs:135`, and `cli.rs:537` constructs `ScanConfig` with
`..ScanConfig::default()`, defaulting both away.
Exposing them is already required by the parity rule; it is also the precondition for
every progressive golden in this plan.

## Goals

- Every bead under `fdu-u7vo` names the files, functions, and signatures it changes, and
  the tests that prove it.
- The three areas the parent spec could not test by example — progressive results, watch
  behaviour, and concurrency — get a stated mechanism that makes them ordinary golden
  and unit tests.
- Determinism comes from injection, per the golden-testing rule that anything which can
  differ between runs must be controlled or filtered.
- Nothing in the test architecture is test-only scaffolding bolted to the side: each
  surface it adds is one a consumer asked for, and each is required by the project’s own
  parity rule.

## Non-Goals

- Re-arguing the contract.
  The parent spec owns *what* and *why*; disagreements belong there.
- A new test framework or a property-testing dependency.
  This repository has no `proptest`/`quickcheck` and keeps its dependency list
  deliberately short; generative tests here are seeded loops in the existing in-file
  `mod tests`, in the style of the exhaustive-over-declared-keys test PR #38 already
  landed.
- Timing gates in CI. A performance assertion on a shared runner measures the runner;
  cost assertions here are *relational* (counter equalities), never wall-clock.

## The Testing Architecture

### What is actually hard, and why

Five capability areas, and they are not equally difficult:

| Area | Why it is hard to test | Mechanism |
| --- | --- | --- |
| Planes and tags | Not hard — more report content | Existing goldens, plus partition-sum loops |
| Runtime registry | Not hard — a pure function of rules | Differential against the compiled default |
| Concurrent reads | Nondeterministic by nature | Deterministic stress with a settled oracle |
| Watch additions | Racy and OS-dependent | Causal sequencing (exists), plus scripted events |
| Progressive session | No surface at all to observe | Trace surface + injected determinism |

Only the last three need new mechanism, and only the last needs a new surface.

### Mechanism 1: the progressive trace is watch mode pointed at the walk

The engine already treats a cold scan and a watch as the same thing: `scan.rs` and
`watch.rs` are both delta producers, and `scan` already takes
`sink: &mut dyn FnMut(Observation)` while `reconcile` takes
`sink: &mut dyn FnMut(&AppliedDelta)`. The command line already knows how to render a
live feed: `Cli::run_watch` (`cli.rs:661`) renders an initial report, then repaints
through `Cli::render_live` (`cli.rs:851`) and streams individual records through
`report_format::render_change` (`report_format.rs:1317`) under
`STREAM_SCHEMA = "fdu.stream/1"`.

So the progressive surface is not a new output contract.
It is the existing repaint loop driven by scan progress instead of watch batches:

- `--progress` joins the **Mode** axis beside `--watch`, so it fits the six-axis rule
  rather than becoming a one-off flag.
- `--progress-at <checkpoint>` names a **logical** trigger — `depth` (a completed
  breadth-first level), `entries:N`, or `batch` — never an interval.
  Wall-clock checkpoints are not reproducible and are the reason `--interval` cannot
  serve here.
- Each emitted frame is an existing `Report` in the requested format, so every view is
  traceable for free and no schema is added.
  Individual deltas emit as the `fdu.stream/1` records watch already emits.
- `Cli::run_watch`’s loop body is factored so watch and progress drive the same
  renderer. The scan session and the watch session present the same shape to it, which is
  the session-to-watch symmetry `fdu-4o0m` already requires.

What this buys, beyond a feature metabrowser asked for: **breadth-first ordering becomes
testable for the first time.** Today the only test of order is that both orders produce
identical engine digests — that is, a proof the order does *not* matter to the final
answer, which is the opposite of the property the order exists for.
The consumer-visible property is that under breadth-first the top-level children grow
together, while under depth-first one completes while its siblings read zero.
A progressive golden over a small fixture shows exactly that, in text, in a diff.

**One documented contract has to be amended rather than quietly broken.** `cli.rs:167`
states, in the `--docs` guide: *“The command never prompts, pages, or animates
progress.”* That sentence stays true in the sense that matters — nothing here animates,
rewrites the terminal, or moves a cursor, and the default is still silent until the
answer. But the sentence as written also forbids what this adds, so it is amended in the
same change to distinguish an animated progress indicator (still never) from an explicit
opt-in stream of frames (this).
Changing behaviour a document promises without changing the document is precisely the
silent exception the design principles forbid.

### Mechanism 2: determinism by injection, which needs two flags that already exist

A watch golden sequences causally because the test performs each change.
A scan trace has no such lever, so its determinism is injected:

- `--threads N` — `ScanConfig.threads` exists (`scan.rs:135`) and is defaulted away at
  `cli.rs:537`. At `--threads 1` the walk has one claimer and emission order is decided
  entirely by the queue.
- `--order breadth-first|depth-first` — `ScanConfig.order` likewise.
- The clock in every frame is the logical `Clock(u64)`, never wall time.
  `[CLOCK]` is already an established named pattern in `cli-watch.tryscript.md`.

Both flags belong to **Scope**, both are already engine capability, and neither is
reachable today — the same defect `fdu-4vkz` names for `order` alone, which widens here
to cover `threads`. That bead therefore blocks every progressive golden, which is the
right dependency: without it there is nothing reproducible to record.

### Mechanism 3: scripted events for the watch paths that cannot be provoked

Causal sequencing covers the changes a test can cause.
It cannot cover the cases that matter most and occur least: overflow escalation
(`InvalidateReason::WatchOverflow`), `UnpairedRename`, `WatchSetupRace`,
`VerificationFailed`, `WatchContention` — the whole `InvalidateReason` enum
(`engine_contract.rs:320`) exists for conditions a test cannot reliably produce on a
real filesystem.

So the watch layer gains a scripted event source behind its existing feature gate: a
JSONL file of backend events replaces the `notify` backend, and those events flow
through **the same** coalescing, the same stat verification, and the same delta path.
The seam is the backend, not the observation, so a scripted event is still verified
against the real filesystem before it becomes an `Op` — the rule that a watch sample is
valid at its `stat` point is preserved, and this is a test seam rather than a back door.
One real end-to-end golden continues to cover the backend binding itself; everything
above it becomes deterministic.

This is the golden-testing guideline’s *“provide a mock mode for all nondeterminism”*
applied at the one seam where this engine has nondeterminism it does not own.

### Mechanism 4: counters as a cost oracle

`counters.rs` already records twenty counters behind `FDU_COUNTERS=1`, and
`FDU_SCAN_DIAGNOSTICS` already demonstrates the pattern for emitting structured run
telemetry: a versioned payload on stderr behind the `__FDU_SCAN_DIAGNOSTICS__=`
sentinel, outside the report envelope, tested in `crates/fdu/tests/cli_exit.rs`.

Counters give an axis no golden covers today: not *what* a run answered but *what work
it did*. Absolute counts are platform-dependent and unstable; **relations between them
are neither**:

- `stats == entries` on a cold walk
- `stats == 0` under `--cache only`
- an idle watch does zero filesystem work — a property the design principles already say
  is *“asserted by test rather than described”*
- a read concurrent with a write triggers no rescan

These are stable, meaningful, and exactly the assertions that catch a regression where
output is unchanged but cost exploded.

### What proves what

| Layer | Lives in | Proves |
| --- | --- | --- |
| Unit / in-file `mod tests` | each module’s `mod tests` | signatures, edge cases, rejections |
| Seeded generative loops | same | partition sums, plane-equals-all, monotonicity |
| Differential | `classify.rs` tests | runtime registry ≡ compiled default |
| Concurrency stress | `fdu-py` `mod tests` + `run_concurrency.py` | no raise, no torn read |
| Golden session | `tests/golden/*.tryscript.md` | the whole invocation a user sees |
| Parity replay | `scripts/run-parity.mjs` | the surfaces cannot disagree |
| Cost relations | golden + `cli_exit.rs` | work done, not just output |

The parity harness is the quiet multiplier: every golden added below is replayed against
the Python surface automatically, so a capability that reaches the command line and not
the package fails the build without anyone writing a second test.
Where a surface legitimately cannot match — the Python package emits no walk telemetry —
the deviation is declared as a class in `scripts/parity-classes.mjs` rather than
silently excluded.

## Implementation Map

Each bead below names files, functions, and the tests that close it.
Line numbers are from the revision this spec was written against and will drift; the
function names will not.

### `fdu-gav9` — shared reads during a write (P0)

**The defect.** `PyIndex` (`fdu-py/src/lib.rs:130`) holds `inner: fdu_core::Index`, an
owned value, so `refresh` (`lib.rs:440`) must take `&mut self`. PyO3 then holds an
exclusive borrow of the whole pyclass across the entire detached reconciliation, and any
concurrent call on that object is rejected.
Every other method already takes `&self`.

**The fix.** Hold the handle the engine already provides:

- `PyIndex.inner: fdu_core::Index` → `IndexHandle` (`index.rs:384`,
  `Arc<RwLock<Index>>`).
- `refresh(&mut self)` → `refresh(&self)`, calling `scan::reconcile_handle`
  (`scan.rs:2861`) instead of `scan::reconcile` (`scan.rs:2839`). That function already
  exists and already takes short write locks per wave.
- Read methods switch from `self.inner.method()` to the handle’s read-locked equivalents
  (`rollup`, `children`, `total`, `since`, `clock`, `freshness`, `len`, `root_path` all
  exist on `IndexHandle`).
- `build_report` (`lib.rs:521`) takes `handle.snapshot()` (`index.rs:503`) and reports
  from the owned clone, which is what `Session::report` (`watch_session.rs:106`) already
  does.
- `watch()` (`lib.rs:362`) simplifies from `IndexHandle::new(self.inner.clone())` —
  which clones an entire index — to an `Arc` clone.

**Two gaps to close first**, both small and both in `fdu-core`:

- `IndexHandle` has no `provenance`; add
  `pub fn provenance(&self, path: &Path) -> Result<Option<Provenance>>` beside the other
  read-locked accessors (`index.rs:450`–`480`).
- `ChildSnapshot` carries `id`, `name`, `kind`, `attrs`, `rollup` but not provenance,
  while `PyIndex::children` (`lib.rs:389`) reports it per child.
  Extend `ChildSnapshot`, so one read lock still serves the whole listing.
- The analysis phase inside `refresh` calls
  `content::analyze_index(&mut self.inner, ..)` and needs a handle-based path or an
  explicit short write.

**Tests.** Extend the existing pattern in `fdu-py/src/lib.rs:1599` — which already has
`same_python_index_uses_runtime_borrow_exclusion` asserting the *current* exclusion — so
it asserts the new contract instead: readers during a write neither raise nor tear.
The Python-level driver is `crates/fdu-py/tests/run_concurrency.py`. The oracle for “no
torn read” is that every concurrent read equals the pre-write or the post-write value
and never a mixture; a settled tree makes those two values known.
Add the counter relation that a read during a write triggers no rescan.

### `fdu-4vkz` — the determinism knobs, widened from `order` to `order` and `threads`

**Files.** `crates/fdu/src/cli.rs` (the `Cli` struct at `:313`, the `ScanConfig`
construction at `:537`), `crates/fdu-py/python/fdu/_models.py` (`ScanOptions`),
`crates/fdu-py/src/lib.rs` (the `open`/`scan` signatures), `tests/golden/`.

- Two `#[arg(..., help_heading = "SCOPE")]` fields joining `scan_depth` and
  `one_filesystem`, parsed by a `parse_order` beside `parse_sort` (`cli.rs:1323`) and a
  plain `usize` for threads.
- `ScanConfig { max_depth, one_filesystem, .. }` at `cli.rs:537` stops discarding them.
- `ScanOptions` gains `order` and `threads`, and `_api.open/scan` forward them.
- Goldens: one session per order over a fixture with several top-level subtrees.
  The parity harness replays them; the surface-vocabulary class in `parity-classes.mjs`
  already covers `--scan-depth` against `max_depth`, so `--threads`/`threads` needs no
  new class.

This bead blocks every progressive golden.

### `fdu-mvt3` / `fdu-7rwf` — planes

> **Superseded 2026-08-24** by *The tag model, made generic* below: tags decoupled from
> planes, the work re-split across
> `fdu-mvt3`/`fdu-brt0`/`fdu-pxfz`/`fdu-xyvu`/`fdu-7rwf`. The file-level observations
> here (where the state goes, the `Entry` padding, the non-invertibility of
> `newest_mtime_ns`) remain correct and are inherited by the new beads; the bead
> boundaries are not.

**Where the state goes.** `index.rs` holds two roll-up types: the public `RollUp`
(`:111`) with `by_ext: BTreeMap<String, ExtTally>`, and the hot-path `InternedRollUp`
(`:133`) keyed by `ExtId`. Planes add a parallel set of totals to both, maintained by
the same functions:

- `Entry` (`:214`) gains tag bits beside `source: Source`, which is already a one-byte
  discriminant — the padding the provenance design already identified.
- `InternedRollUp::merge` / `::unmerge` extend to the new fields, and `merge_upward`
  (`:1499`) / `unmerge_upward` (`:1511`) need no structural change because they
  delegate. Plane sums are invertible; `newest_mtime_ns` is not, and already has its
  repair path in `recompute_newest_upward` (`:1525`), which extends per plane.
- `contribution` (`:1466`) decides what an entry contributes to each plane.

**Invalidation is already wired.** `ScanScope` (`engine_contract.rs:137`) already
carries `ignore_rules_fingerprint`, populated from `IGNORE_RULES_FINGERPRINT = 0` at
`scan.rs:62` under the comment *“No ignore rules exist yet”*, and already serialized by
`put_scope` (`snapshot.rs:657`) and read by `read_scope` (`:676`). Tag rules populate
that constant; plane state bumps `REDUCERS_FINGERPRINT` (`scan.rs:65`). No new field, no
format bump, and a rule change invalidates precisely the snapshots recorded under
different rules rather than every snapshot everywhere — which is what adding a `mix()`
to `engine_fingerprint` (`snapshot.rs:171`) would have done.

**The subtle part: `plane` must not fall to the slow tier.** `Selection::is_unfiltered`
(`query_selection.rs:162`) is the gate between reading precomputed roll-ups and the
re-aggregating `walk` (`query_report.rs:812`). If `plane` is treated as an ordinary
filter it makes every plane query filtered, which is the 122 ms path the parent spec
measured and rejected.
A plane selects *which precomputed roll-up to read*, so `is_unfiltered` must stay true
for a plane-only query and the section builders (`build_section`, `:916`) must route to
the plane’s roll-up.
Combining `plane` with a real filter falls to tier two as any filter does.

**Dependency note.** The tag matcher wants the `ignore` crate, and
`query_glob.rs:7`–`14` already records the decision and the escape hatch: the module
exists to avoid `globset`’s transitive weight for query-time patterns, and states that
*“if the pattern language grows toward regexes or real gitignore semantics … `globset`/
`ignore` goes through the dependency cool-off and this module is deleted.”* Real
gitignore semantics is exactly this bead, so the escape hatch applies as written,
subject to the 14-day cool-off.

**Tests.** Partition sums as seeded loops in `index.rs`’s `mod tests`: for every enabled
tag, plane plus complement equals the untagged totals, across scan, refresh, and watch
mutation. Plane-equals-all when nothing is tagged.
Fingerprint invalidation as a snapshot round-trip under changed rules.
A tagged fixture under `tests/golden/fixtures/` with a `.gitignore` including a
negation, and goldens in every format.

### `fdu-7rwf` — what the surfaces found underneath them

Shipped, and the interesting part is not the surfaces.
`--promote` on Scope, `--plane` on Selection, `ScanOptions(promote=)`,
`Selection(plane=)`, `Child.plane` beside `Child.totals`, and `plane=` on `total()`,
`rollup()` and `children()` were mechanical.
Exposing them found three defects in the maintained state beneath, each of which had
been passing every test that existed.

They share one shape, and it is worth stating because it will recur for any maintained
projection. A plane read is fast precisely because it reads state maintained somewhere
else, on a different code path, at a different time — so a wrong plane is not
distinguishable from a right one by looking at it.
All three produced well-formed numbers of the right magnitude under the right heading.

- **`ensure_dir_chain` built its placeholder’s contribution by hand.** It wrote
  `InternedRollUp { dirs: 1, .. }` with no planes at all, and on a real walk nearly
  every directory is materialised as an ancestor before it is observed — so a plane’s
  directory count was near zero while its files and bytes were exactly right.
  The fix is one shared `count_dir_into_planes`, because two hand-written copies of the
  rule is how the two paths diverged in the first place.
- **A rebind re-tagged every entry and left the planes derived from the old bits.**
  `retag` is the *only* way a Path-tier rule’s bits are ever correct — the control files
  are not known until the walk ends — so this made `gitignore`, the rule planes exist
  for, report a plane equal to the whole tree.
  `rebuild_planes` now runs when any bit moved.
- **An unfiltered `--view summary` was answered by the tier that retains no index.**
  `Selection::plane` is deliberately outside `is_unfiltered`, which is what keeps a
  plane query on the roll-up tier; the planner reads that same predicate to decide
  whether aggregate tallies alone will do, and aggregate tallies hold no plane.
  It did not fail when asked for one.
  It returned the whole tree.

None of the three is visible from one tier.
What found them is `crates/fdu-core/tests/plane_equivalence.rs`: scan a real tree, then
require the maintained plane and a walk over the same restriction to agree, at every
directory, for both a Name-tier and a Path-tier rule, with and without a rebind.
The walking tier is the only independent computation of the same answer that exists,
which makes it the oracle — and the two-tier design that made planes worth building is
what supplies it.

One test-harness gap surfaced alongside them.
`public_smoke.py` calls each check by name from `main`, which is readable and has a
silent failure mode: a check that is written and never listed passes forever and looks
exactly like one that passes.
A new check went unrun for three mutation attempts before that was noticed, so the file
now parses its own source and asserts that every `check_` it defines is called.

### `fdu-xyvu` — admission, and the record a pruned tree has to keep

Shipped as `crates/fdu-core/src/admission.rs`. `HiddenPolicy` is asked once, by name,
before the `stat` — an entry outside scope costs one `bool` and nothing else, which is
the point of pruning at admission rather than filtering afterwards.
It is called from every listing loop the engine has: the serial walk, the parallel walk,
and both reconciliation paths.
That is four call sites and one predicate, because a rule spelled out at each site is a
rule that diverges at one of them, and a scan and a refresh disagreeing about which
entries exist reads as corruption rather than as a rule.

**The snapshot has to record where the pruned control files were, and the test to prove
it has to use `CachePolicy::Only`.** `Index::control_file_directories` finds
`.gitignore` files by reading the index, which is exactly the wrong instrument once
pruning has removed them; the walk therefore reports what it saw into
`ScanReport::control_dirs` and the index adopts it.
Persisting that costs a `FORMAT_VERSION` bump, which `ScanScope`’s new field required
anyway.
The trap is that under `Auto` a revalidation re-walks and re-records them, so the
whole section could be deleted with every assertion still passing — the warm-start test
only says what it means under `Only`, which is contractually forbidden to touch the
tree.

**Two copies of one rule is two messages.** The command line validated `--hidden` itself
and the Python dataclass validated `hidden` again, which produced `--hidden` against
`hidden` and double quotes against single for the same mistake; the parity harness
recorded the pair as a difference between the surfaces, and it was a difference between
two copies of a rule.
`admission::parse_policy` is now the only judge and both surfaces print its sentence.
The one check that stays in Python is `isinstance(hidden_allow, str)`, because a bare
string where a tuple belongs is a Python shape mistake with no command-line spelling to
disagree with.

### `fdu-bjhy` — the second admission axis, asked after the `stat` rather than before

`exclude_special` is `fdu-xyvu`’s rule one axis over, and the difference decides where
it can live. A name says whether it is hidden; nothing about a name says whether it
belongs to a socket.
So `scan::retains` is asked wherever a kind first *becomes known* — after the metadata
read — and that is seven sites rather than four: both walkers, both reconcilers, the
single-path refresh, and the watcher’s apply funnel, which is the third producer of rows
and the only one that learns a kind from an event rather than from a listing it
controls.

**The guard went into the wrong reconcile loop first.** `revalidate` has a listing loop
that looks exactly like the reconcilers’ and is not on their path; with the rule there,
a scan excluded a socket and the first refresh put it back.
The tests were written before the wiring was believed, which is the only reason that was
found here rather than in a consumer’s cache.

**Excluding is removing, not skipping.** A file replaced in place by a socket is one
event on a path that never goes absent, and every listing loop takes the name out of its
missing-set before the kind is known — so a `continue` leaves the old row standing over
the socket for as long as the index lives, because nothing will look at that path again.
Each site emits `Op::Remove` instead, and the watcher’s substitution carries the
producer’s expectation across: rebatching through `Observation::new` would flatten every
arbitration precondition to `Any`, quietly widening what an excluded kind may overwrite.

**Admission runs before the budget claims a slot.** `fdu-97dd`’s cap is strict, so an
entry the scope does not hold must not spend a slot a retained one could have used.

**No format bump.** The scope flags byte had a spare bit and a clear bit means “kept”,
which is what every snapshot written before this change meant.

**What the watcher’s test can and cannot prove.** On Linux a rename onto a watched path
escalates to a root invalidation, so reconciliation sweeps the stale row away and a
watcher that merely *ignored* the event looks correct from outside.
The integration tests prove the rule holds end to end; only a unit test on the admission
function separates excluding the object from dropping its event, which is the difference
that matters on a backend reporting the file without invalidating its parent.

### `fdu-91ru` — a continuation that costs a page

The first version of the flat page was bounded and lossless and quadratic, which is a
combination worth naming because two of the three look like success.
Every call began at the root, filtered the whole subtree, recomputed the selection-wide
denominator, and counted every match at or before the cursor — so assembling P pages
cost P passes over the index.
The lossless-assembly tests all passed, because losslessness is not the property that
was broken.

Two costs, and they need different answers:

- **The seek.** A bare path cursor identifies where to resume and gives no way to get
  there except forward from the top.
  `seek_after` descends the cursor path instead: at each level it pushes the siblings
  *after* the component it came through — a range over an ordered map, not a scan that
  discards what it passes — then the cursor’s own children on top, which is exactly the
  stack the walk would have left.
  No entry before the cursor is looked at.
- **The denominator.** An arbitrary predicate over a tree has no ordered index to count
  through, so the first page has to walk the selection to learn its size.
  That one is unavoidable; recomputing it per page is not.
  `EntryCursor` carries the total, the aggregates and the delivered count forward.

Measured on a 660-entry fixture at limit 5: 666 entries visited for the first page, then
a flat 14 for every page after it, page 2 and page 25 alike.
The test asserts flatness rather than a ratio, because flatness *is* the property — a
continuation that crossed the prefix would cost more with every page, and a ratio
against the first page would pass for one that crossed half of it.

**The cursor is version-bound, and that is not belt-and-braces beside `expected`.** Its
counts were established against one image, so replaying it against another would report
a denominator for a tree that is no longer there — a wrong answer with nothing in the
page saying so, which is worse than the stale-pin case a caller can already detect.

**What a consuming wire format has to do with it.** A catalog query whose cursor is a
string should carry this value *encoded*, not reduce it to a path: the path is the half
that makes every page pay for the whole selection again.
That is a mapping, not adapter state — no mirror, no retained result set, no second
cursor.

**What still does not page: sorted queries.** A resumable cursor has to seek in the
order it emits. Path order is a total order the tree already holds; mtime order is not,
so “recently changed” remains a bounded *slice* rather than a page.
`fdu-t5h2` records the two shapes that could fix it and why neither should be built
without a consumer asking.

### `fdu-7sou` / `fdu-97dd` — where each scope axis can be kept

Watching used to refuse every narrowed scope, on one rule over three axes.
Splitting the rule by *what each axis is a property of* is what made a bounded consumer
handle possible, and the split is the whole design:

- **The hidden-path rule and `max_depth` are properties of the path itself**, so
  `within_scope` redraws both around a single event with no I/O at all.
  The hidden rule asks every *component*, not the leaf: the walk never descends into a
  pruned directory, so nothing beneath one is in the index, and a backend reports
  `.git/HEAD` as its own path with nothing in that name saying its parent was pruned.
- **`one_filesystem` needs a `stat`, and of the entry’s parent.** `should_descend` gates
  descent, not retention, so a mountpoint is listed by its parent and *recorded* while
  nothing under it is ever read.
  The rule is therefore “did the walk descend into this entry’s parent” — asking the
  entry’s own device rejects the very row the scan keeps, so a live event on a
  mountpoint would delete it and the next rescan put it back.
  `scan::within_scope` is that predicate, beside `admits` (by name, mid-listing) and
  `retains` (by kind, after the `stat`). Depth counts components, which agrees with
  `should_descend` by construction: it admits a child at `parent_depth + 1 < max`, so
  the deepest entry a walk records has exactly `max` components.
- **`max_files` is a property of the whole inventory**, so no per-event predicate can
  decide it — whether *this* file is inside the cap depends on every other file,
  including the ones the capped walk never read.
  The index keeps it instead, in `upsert_beneath`, which is the one place a new row is
  allocated and the one place the previous state of the path is already in hand.

That second half also closed a gap nobody had connected to watching: reconciliation
walks from the index and never consulted `scan::Budget`, so **one refresh turned a
bounded inventory into an unbounded one** while the scan identity went on claiming a
cap.
`Budget` keeps its own job — stopping *discovery*, which is what makes a capped scan
cheap.

**An out-of-scope upsert becomes a removal, not a dropped op.** A directory moved
deeper, a filesystem mounted over one, a file replaced by a socket: each is one event on
a path that never goes absent, so anything short of a removal leaves the old row
standing over the new object forever.
An out-of-scope *invalidation* is dropped instead — it asks for a subtree to be
reconciled and there is no subtree.

**Directories are not counted against the cap.** A directory carries no bytes of its
own, and admitting one keeps the tree navigable to what is already there; counting them
would lose the shape of the tree as well as its contents.

**The refusal and the coverage loss are one commit.** `AppliedDelta::of_both` exists for
this: at a cursor between them the index would have dropped an entry and still claimed
to cover everything, which is a moment that never happened.

**What no rule can give.** Which files a long-lived capped index holds depends on the
order events arrived, as which files a capped *walk* holds depends on the order it
reached them. No rule bounds the retained set and is history-independent at once.
Coverage says the set is short, which is the fact a consumer can act on — and the
alternative, letting events through, makes the cap a hint that any long session walks
past.

Worth carrying into the cross-engine fixture: the consuming contract’s own reference
walker gives each subtree rewalk a fresh `max_files` budget, so its retained set is
bounded per walk rather than in total.
One side has to move, and this is the side that keeps the bound a bound.

### `fdu-b2vy` / `fdu-ctp5` / `fdu-e2p7` — the taxonomy

**Groups.** `ContentFamily` (`classify.rs:19`) is a closed five-value enum answering an
analysis question and must keep doing so.
The browsing taxonomy is a second axis: a `group` field on the rule dialect
(`GeneratedRule`, `:158`), a `groups` view beside `families`, and group totals on the
roll-up types alongside planes — the same reducer path, which is why `fdu-n4gn` measures
them together.

**Runtime registry.** `build.rs:217` renders `TYPE_RULE_FINGERPRINT` and the rule table
into `OUT_DIR`, included at `classify.rs:167`; `RULES_BY_FILENAME`/`RULES_BY_EXTENSION`
(`:181`–`184`) index it into `LazyLock` statics over `&'static` data.
The registry becomes a value:

- `GeneratedRule`’s `&'static str` fields become owned, and the two statics become
  fields on a `TypeRegistry` built by the same `index_rules` (`:186`) — same algorithm,
  same tie-break, different lifetime.
- `classify_path_with_prefix` (`:268`) takes `&TypeRegistry`; a default-registry wrapper
  keeps every current call site working.
- `type_rule_fingerprint()` (`:204`) stops being `const fn` over a compiled constant and
  reads the active registry.
  Its three consumers — `scan.rs:203`, `content_model.rs:245`, and the freshness
  comparison at `content_model.rs:263` — already compare it, so invalidation needs no
  new plumbing.
- The compiled registry stays the default and the fast path: no file to find, no parse
  at startup, CLI behaviour unchanged.

**Tests.** The differential is the important one and it generalizes a test PR #38
already wrote: `indexed_rule_tiers_agree_with_the_scan_they_replaced` pins
`max_by_key`’s last-wins tie-break over every key the table declares.
That becomes a property over *any* registry, plus a migration assertion that the
runtime-parsed default classifies byte-identically to the compiled one over the same key
set. Validation is tested for what it rejects: duplicate ids, unknown group, tie-break
ambiguity, a fingerprint colliding with the default.

### `fdu-mz1a` / `fdu-fh0k` / `fdu-rhu3` / `fdu-97pb` — the watch contract

- **Dirty sets** (`fdu-mz1a`): `merge_upward` (`index.rs:1499`) already walks exactly
  the ancestors whose roll-ups changed.
  `Batch` (`watch_session.rs:61`) carries the set; `PyWatch::__next__` (`lib.rs:1023`)
  surfaces it. Tested against an independently computed ancestor set, and goldened
  through a `--watch` driver as the existing capture scripts do.
- **Scoped refresh** (`fdu-fh0k`): `scan::reconcile_subtree` (`scan.rs:2851`) already
  exists; the work is the Python signature and the equivalence test against a full
  refresh over the touched subtree.
- **Poll backend** (`fdu-rhu3`): a `WatchConfig` backend choice with a stated interval,
  beside the existing `settle`/`max_hold` durations (`watch.rs:83`).
- **Asyncio adapter** (`fdu-97pb`): a worker thread draining `Watch.__next__` into an
  `asyncio.Queue`, shipped in the package rather than left to each consumer, plus the
  SSE-resume example mapping `since`/`truncated` (`index.rs:251`, journal capacity
  `:55`) to `Last-Event-ID` and resync.
  Blocked by `fdu-gav9`: an event-loop adapter over a surface that raises under
  concurrent access relocates the defect rather than fixing it.
- **The terminal state** (`fdu-vfx7`): `Index::engine_state()` assembles trust,
  coverage, lifecycle and run facts in one place, and `Index::since` captures it inside
  the guard that already produced the journal slice and the cursor.
  A batch therefore says what moved, where to resume, *and* how far to trust what the
  consumer now holds, all from one instant.
  The field the transitions list used to occupy is now `transitions`, because the two
  answer different questions and sharing a name invited the mistake this fixes: a
  consumer folding interval events into its own copy of the state is the mirror the
  boundary exists to forbid.
  A follow-up read is not an equivalent substitute and cannot be made one — the next
  commit can land between the two calls, and the index retains only its current image,
  so there is nothing to ask for the state as of a position already passed.
  Both acceptance tests force that interleave rather than racing it.

### `fdu-4o0m` — the session, and the trace surface

The session’s Python and CLI shapes land together, because the CLI shape is what makes
the Python shape testable.
`Cli::run_watch` (`cli.rs:661`) is refactored so its repaint loop takes either producer;
`--progress`/`--progress-at` join the Mode axis; `prepare_report` (`execution.rs:188`)
grows a progressive sibling that retains the index and yields frames.
Progress is readable mid-walk — entries applied, clock, completeness — because a
crawl-status UI renders exactly that.

**Tests.** The progressive goldens are the headline: at `--threads 1` with each order, a
fixture with several top-level subtrees, `--progress-at depth`, showing breadth-first
growing the top level together and depth-first completing one subtree while its siblings
read zero. Monotonicity is asserted as a relation across frames rather than by eye.
Frame count stays bounded by using `depth` rather than `entries:1`, keeping each golden
inside the size budget the guidelines set.

### `fdu-16l7` / `fdu-tib6` / `fdu-knyw` / `fdu-vfyw` — adoption

- Classification identity in listings extends `ChildSnapshot` — the same struct
  `fdu-gav9` already touches, so the two should land in that order.
- Walk telemetry as typed values mirrors `PerformanceSummary` (`execution.rs:59`),
  delivered beside the report exactly as `performance_footer` (`cli.rs:1099`) does for
  text, never inside the envelope.
- `TreeNode` remainders replace a bare `truncated: bool` with the dropped aggregate.
- The reference embedder and the cross-engine fixture are the acceptance test `fdu-p02b`
  asked for.

## What Landed

The map above was written before the code was.
This section records what was actually built, at the same file-and-function level, so
the two can be compared — and so the places where implementation contradicted the plan
are on the record rather than in a commit message nobody re-reads.

The tables below are the shipped rows, and they are the claim: each names a bead and
where its work landed.
A count is deliberately not stated here.
`fdu-u7vo` expands through implementation and review — the last count written into this
file was stale as it landed — so `tbd list --parent fdu-u7vo --all` is the live map and
this document is the record of what was built.
Every row cleared `make check`, which replays the golden corpus against the command line
and against the Python package and fails on any unclassified difference.

### Engine: the read path

| Bead | What shipped | Where |
| --- | --- | --- |
| `fdu-gav9` | `PyIndex` holds an `IndexHandle`, so a read is served during a write | `fdu-py/src/lib.rs`, `index.rs:IndexHandle::with_index` |
| `fdu-2ivi` | `IndexHandle::read(&ReadRequest) -> ReadBundle`: children, roll-ups and totals under one guard | `index.rs:IndexHandle::read` |
| `fdu-plwq` | `IndexHandle::children_page(&ChildPageRequest) -> Option<ChildPage>` | `index.rs:child_page` |
| `fdu-qgl9` | `ReadBundle::work`, a per-read `Work` record | `index.rs:Work`, `Index::lookup_visiting` |
| `fdu-5hip` | `RollUp::others`, `RollUp::is_empty`, `ChildSnapshot::is_empty_subtree` | `index.rs:contribution` |
| `fdu-e2p7` | `Bound` on every roll-up’s extension rows, with `ExtRemainder` | `index.rs:named_rollup_bounded` |
| `fdu-knyw` | `TreeNode::remainder` replaces a bare `truncated: bool` | `query/query_report.rs:withheld_children` |
| `fdu-samw` | `ReadRequest::report` and `ReadBundle::report`: the whole query algebra under the bundle’s guard, with `ProjectionWork` saying what each part cost | `index.rs:IndexHandle::read`, `index.rs:ProjectionWork` |
| `fdu-or38` | `others` on `SummaryRow` and `TreeNode`, so the report views can tell a directory of symlinks from an empty one | `query/query_report.rs`, `report_format.rs:others_suffix` |

Four things about this group were not obvious from the outside.

**`snapshot()` is not a read.** The first fix for `fdu-gav9` routed `build_report`
through `IndexHandle::snapshot()`, which deep-clones every entry.
Every test passed and a report went from milliseconds to seconds — a 1,900× regression
that only re-running the measurement caught.
`with_index` holds the read guard and hands out `&Index` instead, which is what a pure
reader needs and what `snapshot` was never for.

**A listing row must not carry a map.** `ChildSnapshot` built an owned `RollUp` per
directory child, so a thousand children cloned a thousand `BTreeMap`s to render a
thousand size columns.
`ChildSnapshot::totals` is now `Option<RollUpScalars>`, which is `Copy` — the type is
the assertion, since a row physically cannot hold a map.
The breakdown stays available as its own projection for the one directory being
inspected.

**A page’s remainder is the complement of that page, not of the delivery.** Stating it
against the whole directory keeps it exact on every page with no cursor carrying a
running total, and “showing 50 of 812” is the sentence a listing wants.
It also means `remainder` stays present on the last page, so `next` — and only `next` —
says whether paging continues.
A consumer looping on `truncated` would never stop, which is why there is a test named
for exactly that.

**A bundle had to grow past three projections, and the provenance could not come with
it.** The contract asks that every result carry version, cursor, lifecycle, coverage and
work facts describing *the same observation boundary*, over nine query kinds.
The bundle carried three; everything else was reachable only through `report()`, which
takes its own guard — so a listing beside a “recently changed” panel was two calls with
a write able to land between them, and the halves described different moments while each
stayed individually true.
`ReadRequest` now carries a `Query`, and the report is built inside the same guard.
What could *not* move inside is the provenance: scan start, cache tier, completeness and
errors are facts about the **run**, and a read guard has no opinion about them.
They ride in with the query rather than being sampled where they were not measured.

**A total that hides which part was slow is a counter that stopped working.**
`ProjectionWork` charges children, totals, roll-ups and the report separately, and
`ReadBundle::work` is their sum term by term.
The one figure that stays shared is `lock_wait_ns`: the projections waited together, so
splitting it would be inventing a number — and that is exactly the reason the original
single `Work` gave for not splitting anything.
Wall time *is* separable, because the parts run in sequence.
The report projection deliberately reports no `entries_visited`: a report either serves
from maintained roll-up state or re-aggregates by walking, and claiming a walk it did
not do — or a zero for one it did — is worse than claiming neither.
`Selection::is_unfiltered` is what decides which happened, and the caller already holds
it.

**Symlinks weigh nothing and are not nothing.** `contribution()` gave symlinks and
devices a default roll-up, so a directory of a hundred symlinks was zero files, zero
directories and zero bytes: the same arithmetic as empty.
`others` counts them, and `ChildSnapshot::is_empty_subtree` returns `Option<bool>` so a
`Status::Partial` subtree declines to answer rather than claiming emptiness it has not
established.

The report views could not read that count, so the same bug survived at the surface the
surfaces are supposed to agree on: `--view tree` went on rendering a hundred symlinks
and nothing at all identically.
`fdu-or38` carries it into `SummaryRow` and `TreeNode`, and it was left open
deliberately because what remained was a *display* decision rather than an engine one.
The decision is a suffix, not a column, and absent rather than zero: a column spends
width on every row of every tree for a number that is zero almost everywhere, and a
printed `0 others` does the same to the eye.
Machine formats carry the field unconditionally, because a consumer branching on a key’s
presence is a consumer with two code paths for one question -- the same argument the tag
work made for always emitting `"tags": []`. The text goldens did not move at all, which
is that decision working.

### Engine: classification

| Bead | What shipped | Where |
| --- | --- | --- |
| `fdu-ctp5` | `TypeRegistry` as a runtime value; the compiled default is one instance of it | `classify.rs`, `classify/type_rule_manifest.rs` |
| `fdu-b2vy` | `GroupId`, `RollUp::by_group`, `--view groups` | `classify.rs:TypeGroup`, `index.rs:InternedRollUp::by_group` |
| `fdu-16l7` | `Classification` and logical extension on listing and files rows | `index.rs:child_snapshot`, `query_report.rs:file_rows` |
| `fdu-5q6e` | Two extension levels: `logical_ext` and `TypeRegistry::canonical_ext` | `classify.rs` |

The manifest parser is `include!`d by `build.rs`, so the dialect that parses a
user-supplied rule file is the same code that generates the compiled default — one
parser, not two that agree until they do not.
Its header comment uses `//` rather than `//!`, which is not style: an `include!`d file
is spliced mid-module and an inner doc comment there is a compile error.

Group tallies are **maintained**, not derived from `by_ext`. Deriving is wrong twice
over: an exact-filename rule (`Makefile`, `Dockerfile`) has no extension bucket to
derive from, and a registry may map two extensions of one group to different types.

The two extension levels are one change with two regressions hiding in it.
`classify_with` matches rules by exact key with no suffix fallback, so returning the raw
`.v2.zip` alone makes the archive `unknown:.v2.zip`, and `ext_bucket` wraps the same
lookup, so the `.zip` roll-up bucket splits at the same moment — in exactly the names
the change is for. The canonical level exists to make the logical level safe.
The property the bead asked to pin was verified by running `fdu` against its own fixture
before and after: `--view types` and `--view extensions` are byte-identical.

### Surfaces

| Bead | What shipped | Where |
| --- | --- | --- |
| `fdu-4vkz` | `--order` and `--threads` on the Scope axis; `ScanOptions.order`/`.threads` | `fdu/src/cli.rs`, `fdu-py/python/fdu/_models.py` |
| `fdu-ctp5` | `--type-rules` and `ScanOptions.type_rules`; `fdu.TypeRegistry` | `cli.rs:load_type_rules`, `fdu-py/src/lib.rs:PyTypeRegistry` |
| `fdu-tib6` | `Index.telemetry` as a typed `WalkTelemetry` | `fdu-py/python/fdu/_models.py` |
| `fdu-mz1a` | `Watch.dirty_rollups`: the roll-ups each batch invalidated | `watch.rs`, `fdu-py/src/lib.rs` |
| `fdu-rhu3` | `WatchBackend::Poll { interval }`, reachable as `poll_interval` | `watch.rs:WatchBackend` |
| `fdu-97pb` | `fdu.aio.watch_batches()` and an SSE-resume example | `fdu-py/python/fdu/aio.py`, `examples/sse_resume.py` |
| `fdu-vfx7` | `EngineState`, carried as `Batch::state` and `Since::state`; `Batch::transitions` renamed from `state` so the two cannot be confused | `index.rs:EngineState`, `watch_session.rs:Batch`, `fdu-py/src/lib.rs:engine_state_dict` |
| `fdu-97dd` | `ScanConfig::max_files` and `ScanScope::max_files`; the walk stops descending at the cap, coverage becomes `Partial(Budget)`, and a typed `ResourceStop` issue says so | `scan.rs:Budget`, `snapshot.rs` (format 4), `cli.rs:--max-files`, `fdu-py/python/fdu/_models.py:ScanOptions` |
| `fdu-nlhl` | `scan::reconcile_paths` and a `RefreshReceipt`: many hint paths as one operation, overlapping ones folded into one walk, every subtree announced before any is read, and typed per-path refusals | `scan.rs:reconcile_paths_target`, `scan.rs:covering_roots`, `fdu-py/python/fdu/_api.py:refresh_paths` |
| `fdu-91ru` | `EntryPageRequest` / `EntryPage` on the bundled read: a bounded, resumable page in path order, with an exact remainder paired with its continuation and totals over the whole selection | `index.rs:entry_page`, `index.rs:push_children`, `fdu-py/src/lib.rs:entry_page_dict` |
| `fdu-vfx7` | `Interest::{Rows, Invalidations}` and `Session::with_interest`; the batch’s cost measured across the binding, with the phases exact-or-absent rather than defaulting to zero | `watch_session.rs:Interest`, `fdu-py/src/lib.rs:PyWatch::__next__`, `fdu-py/python/fdu/_api.py:Watch.__next__` |
| `fdu-bjhy` | `ScanConfig::exclude_special` and `ScanScope::exclude_special`: sockets, FIFOs and device nodes pruned at admission rather than filtered afterwards, so the index holds three kinds and the roll-ups count exactly the rows a listing shows | `scan.rs:retains` (both walkers, both reconcilers, the single-path refresh), `watch.rs:retained`, `snapshot.rs:SCOPE_EXCLUDE_SPECIAL`, `cli.rs:--special`, `fdu-py/python/fdu/_models.py:ScanOptions.special` |
| `fdu-7sou` / `fdu-97dd` | A bounded scope is watchable: `max_depth` and `one_filesystem` as per-event predicates, `max_files` as an index-owned inventory bound the walk, the refresh and the watch all keep | `scan.rs:within_scope`, `index.rs:upsert_beneath`, `engine_contract.rs:AppliedDelta::of_both` |
| `fdu-vfyw` | The reference embedder produces the consuming contract’s own scope-digest bytes, from a fixture recorded by running its function rather than re-reading its spec | `fdu-py/examples/browser_provider.py:scope_fingerprint`, `fdu-py/tests/fixtures/scope-fingerprint.json` |

`WatchConfig` lost `Copy` when the poll interval arrived, which threaded `&WatchConfig`
through `validate`, `apply_intent`, `verify_intent` and `run_worker`. That is the kind
of change worth naming because it looks like churn and is not: a config that can no
longer be copied is a config that can no longer be silently diverged.

The asyncio adapter owns the thread-affinity rule rather than documenting it.
`PyWatch` is `#[pyclass(unsendable)]`, so the first version — create the watch, hand it
to a worker — panicked at runtime.
The adapter opens, drains and closes the watch **on** the worker thread, and the event
loop only ever sees a queue.
Backpressure is `asyncio.run_coroutine_threadsafe(...).result()`, and the `finally`
block drains the queue so a cancelled consumer cannot leave a producer blocked on a full
one.

### Engine: the tag model

| Bead | What shipped | Where |
| --- | --- | --- |
| `fdu-5yqb` | `Status::Partial(CoverageReason)` — partial coverage says why, not only that | `engine_contract.rs:CoverageReason`, `index.rs:FreshnessMark` |
| `fdu-mvt3` | `TagRule`, `TagTier`, `TagRules`, `TagBits`; `Entry.tag_bits`; `Selection::tags`; `--tag-rules`/`--tag`/`--not-tag`; `ScanOptions.tag_rules`, `Selection.tags`/`.not_tags` | `tags.rs`, `index.rs`, `query/query_selection.rs`, `cli.rs`, `fdu-py/src/lib.rs` |

Four things about the tag model were decided against the plan as written, and each was
decided by something the code made visible rather than by argument.

**A tag is not a plane.** The plan had them one and the same, and the coupling had
already forced `hidden` out of the model once: a maintained per-directory aggregate for
hidden entries would have to walk the `.git`, cache and virtualenv trees the tag exists
to identify. They are now separate concepts in the same module, with the split stated at
the top of it: tags are unbounded and nearly free, planes are a small *declared* subset
that rides the ancestor-merge path and costs on every mutation whether or not anyone
reads them. Filtering by an unpromoted tag re-aggregates by walking, which is the
two-tier rule the query surface already applies to every other predicate.

**A rule declares what it may read, and the engine refuses what it cannot afford.**
`TagTier` is `Name`, `Path` or `Content`, and `TagRules::from_names` rejects a
Content-tier rule at enable time.
Without that check, adding a `binary` or `text` tag would silently turn a metadata walk
into a content walk, and the only symptom would be that scans got mysteriously slower.
The refusal is its own error variant rather than folded into “unknown”, because an
unknown name is a typo and this is a real rule that costs more than the caller asked to
spend — opposite responses to what looks like one class of mistake.

**Tag bits are not in the snapshot, and that is the design.** They are derived from
facts the index already holds, so `Index::with_tag_rules` re-tags what is already there.
That is not an optimization: the loader restores entries *before* the caller’s rules are
known, so an index that only tagged at insert would answer “no tags” for every entry
after a warm start while a cold scan of the same tree answered correctly — one tree, two
answers, and it would read as a cache fault rather than a tagging one.
A `--cache only` run that walks zero files now returns the same rows as a cold scan.

**The loader must not be handed back its own optimization.** `TagRules::evaluate` takes
the relative path as a closure rather than a value.
The upsert path already holds a path; the snapshot loader holds a parent id and a
basename, and reconstructing a path per record is exactly the work a callgrind profile
put at about 27% of load in the allocator and which that path was rewritten to avoid.
The first draft of this wiring called `path_of(parent).join(name)` per record — in the
function whose own doc comment says why it must not.

Two smaller decisions are worth recording because they were reversed once.

`ensure_dir_chain` tags its placeholder directories rather than leaving them at zero.
`apply_upsert` on an existing entry of the same kind rewrites attributes and source
only, so a directory that entered as an ancestor placeholder would have stayed untagged
for the life of the index — and every ancestor of a deep first observation enters there.

The command line no longer wraps a tag error in `invalid --tag:`. The parity harness
caught it: the library message already quotes the rejected name and lists what is
available, so the prefix added only the one thing the Python surface could not say
identically. Both surfaces now print the same sentence, and the five new golden sessions
record as exact matches rather than as a declared deviation class.

### Test architecture

| Bead | What shipped | Where |
| --- | --- | --- |
| `fdu-0jyz` | `WatchBackend::Scripted { events }`, a tab-separated event script | `watch/scripted_events.rs` |
| `fdu-524n` | `FDU_COUNTERS` machine payload on stderr, asserted by relation | `counters.rs:Counts::to_json`, `tests/golden/cli-cost.tryscript.md` |

The scripted backend is what makes the `InvalidateReason` paths testable at all: a test
cannot ask a filesystem for an inotify `Q_OVERFLOW`. Writing those tests found two
assertions **backwards** in the plan and in this author’s head: a rescan flag escalates
*the subtree it names*, and an unpaired rename escalates *to the root*, because there is
no safe bound on where the counterpart landed.
The tests now pin the engine’s real contract, which is the opposite of the one that was
assumed.

Counter assertions are relational and never wall-clock, as the strategy above requires.
The Python surface emits no `__FDU_COUNTERS__` line, so the golden driver scripts are
guarded and the difference is a declared `process-instrumentation` class in
`parity-classes.mjs` rather than a harness crash.

### What the corpus learned

Two golden-corpus rules earned their keep and are worth restating with the evidence.

`tryscript run --update` writes *what it saw*, expanding named patterns into literals
and producing a golden that passes only on the machine that recorded it.
The workflow that works is: snapshot `tests/golden`, run the update, then transplant —
keep the named patterns from the before-file and take only genuine behaviour changes
from the after-file.
`make check`’s portability pass refuses the alternative.

A directory’s `bytes` is filesystem-dependent (4096 on ext4, not 0), which needed a new
`DIR_SIZE` named pattern.
And a cache-only session cannot be warmed with `--view summary`: that view is served
from the transient tier and writes no snapshot, so the warming run has to be
`--view tree`.

### Read against the reconciliation

The contract was amended after this map was written, by
[the reconciliation](../research/research-2026-08-23-interactive-contract-reconciliation.md)
and metabrowser’s reply on PR #44. Checking what landed against that amended list rather
than against the original found three things, and two of them were gaps in work already
called done.

**Every “new work” item is built**: the batched multi-projection read under one guard
returning clock, cursor, state and fingerprints; per-result work counters; roll-up leaf
counts. Both flipped readiness verdicts are addressed on the `children()` side.
Nothing here reproduces metabrowser’s HTTP or wire types, and the poll backend is
exactly the “capability negotiation limited to real platform gaps such as native-watch
availability” the amendment asks for.

**The registry packet was supplied and echoed but not checked.** “Supply the immutable
registry packet at open, echo the indexed identity, and fail on disagreement” — the
first two thirds landed and the third did not.
Both fingerprints were readable, so a caller could always have compared them; what it
could not do was be *unable* to skip the comparison, which is the whole point of a
packet identity. `from_manifest` now takes the expected identity and fails the open,
naming both numbers.

**The SSE example was silent about a gap the contract names.** The amendment says the
resume cursor is not ready for a production feed until trust transitions ride the same
clock. The example did not claim otherwise, and did not say so either — and `fdu-jxs0`
records why: an unchanged upsert mutates `entry.source` and returns `false`, and
`finish_reconcile` mutates `verified` directly, so neither advances the clock nor
reaches an `AppliedDelta`. A consumer following the example would have built a feed that
is current on what changed and silent on how far to trust it.
The example now says so.

**One amendment had not been applied to the beads at all**: `fdu-jxs0` was to rise from
P2 and had not. It has now.

### Read against the implemented provider contract

MetaBrowser Phase 1 shipped, and with it the consumer-side contract became concrete
rather than prospective
([`arch-inventory-provider.md`](https://github.com/jlevy/metabrowser/blob/b17fafb/docs/project/architecture/arch-inventory-provider.md),
relayed on PR #44 on 2026-08-24). Reading fdu’s Python surface against the real
`InventoryHandle` rather than against a description of it answers three of five
operations and finds four concrete gaps.

| `InventoryHandle` | fdu today |  |
| --- | --- | --- |
| `open(root, config)` | `fdu.open` / `fdu.scan` | built |
| `read(request)` → version, cursor, state, projections, work | `Index.read` under one guard | built (`fdu-2ivi`, `fdu-plwq`, `fdu-qgl9`) |
| `refresh(request)` | `Index.refresh(path=…)` | built (`fdu-fh0k`) |
| `prioritize(request)` | — | **`fdu-sgp7`** |
| `close()` | on `Watch`, not on `Index` | **`fdu-sgp7`** |

**Coverage says whether, not why.** The contract wants complete, or partial with one of
`building`, `budget`, `cancelled`, `inaccessible`, `failed`. `Status` is a bare
`Complete | Partial`, so a consumer cannot tell a walk still running from a directory it
could not read from a dropped watch queue — three situations with three different
correct responses. Four of the six reasons are already engine state that simply is not
carried; two need the session.
**`fdu-5yqb`**, and the one of these four that is ready now.

**The invalidation vocabulary is one of four signals.** `reset` is built and is exactly
`ChangeSet.truncated`. Dirty *paths* exist as `Watch.dirty_rollups`. `all_dirty` is not
distinguishable — when the watch escalates to a root invalidation the set is unlabelled,
so “the root’s own roll-up moved” and “throw everything away” look alike, and they
demand opposite amounts of work.
Dirty *query kinds* are absent entirely.
A state-only transition produces no signal at all, which is `fdu-jxs0`. **`fdu-fltq`**.

**The bundled read is one level short of the promise.** Its guarantee is that everything
in a result describes *the same observation boundary*, over an algebra of nine kinds.
`ReadRequest` carries three of them — directory, roll-up, totals — and every other kind
is reachable only through `report()`, which takes its own guard.
So a consumer wanting a listing and a recent list at one instant makes two calls, a
write lands between them, and the page is internally inconsistent in precisely the way
the bundled read exists to prevent.
That is the defect `fdu-2ivi` fixed for listing-plus-header, still present one level up.
**`fdu-samw`**.

What the contract confirms rather than changes: fdu’s freshness vocabulary (`fresh`,
`reconciling`, `stale`, `partial`) is already the contract’s, the two extension levels
are the settled rule, and points 1, 3 and 6 — one handle owning the retained index,
bounded scalar and paged projections, a translational adapter holding no semantic state
— are built and were the subject of this branch.

### The tag model, made generic

The owner’s direction (2026-08-24): gitignore is one flag among several — text, binary,
and other facts will follow — so the model must be generic and the code must not fill
with if-gitignore blocks.
The review that preceded this found nothing built beyond the reserved
`ignore_rules_fingerprint` slot at `0`, which made it the cheapest possible moment to
fix the shape.
The approach was then delegated; these are the decisions, each recorded on
the bead that owns it:

| Decision | Bead |
| --- | --- |
| Tags (unbounded entry bits) decoupled from planes (a small *declared* promoted subset) — the 1:1 coupling had already forced `hidden` out of the model once | `fdu-mvt3`, `fdu-pxfz` |
| Rules carry a tier — Name, Path, Content — and Content-tier rules are rejected at enable time in v1, so a future `binary` tag cannot silently turn a metadata walk into a content walk | `fdu-mvt3` |
| Categorical facts (mime type) are not tags: they are interned-key tally maps, the `ext_id`/`group_id` mechanism. Two shapes; neither absorbs the other | spec Phase 1 |
| `ScanScope.ignore_rules_fingerprint` → `tag_rules_fingerprint`; same wire position, empty set still fingerprints to 0, every existing snapshot stays valid | `fdu-mvt3` |
| `ignore` lands behind a default-on `gitignore` feature (notify’s precedent; measured +1.06 MiB, 9 crates, no lean mode). The MSRV trap found by checking went deeper than a version pin: 0.4.30 declares no `rust-version` and still needs 1.88, so the workspace floor moved to 1.88 rather than the crate moving back | `fdu-brt0` |
| `dotfile` ships as the zero-dependency Name-tier reference rule, unpromoted — the model is provable end-to-end before the dependency lands | `fdu-mvt3` |
| Hidden admission is scope (prune + allowlist + fingerprint), its own bead, owning its `FORMAT_VERSION` bump | `fdu-xyvu` |
| `Classification.flags`: `vendored` and `documentation` fold in as Path-tier rules sharing one predicate with the classification; `generated` cannot, because its tier is refused | `fdu-n7mv` |
| Keep the `others` leaf counts — the implemented contract requires them; the measurement (`fdu-2ig2`) runs on any quiet host or rides `fdu-n4gn` | beads |
| `fdu-vrwy` stays its own change, not this PR | bead |

The build order this implies: **Track A** (contract completion, engine-internal) is
`fdu-5yqb` → `fdu-samw`, with `fdu-fltq` behind `fdu-jxs0` and `fdu-sgp7` behind the
session. **Track B** (tags) is `fdu-mvt3` → `fdu-brt0` / `fdu-pxfz` → `fdu-7rwf` →
`fdu-vfyw` / `fdu-n4gn`. **Track C** (independent smalls, any time): `fdu-or38`,
`fdu-xyvu`, `fdu-vrwy`. The session chain stays behind the progressive-results epic.
Ready today: `fdu-5yqb`, `fdu-mvt3`, and Track C.

## What Did Not Land, And Why

Seven beads under the epic are open, and none of them is open because the work was
tedious.

**`fdu-mvt3`, `fdu-7rwf` — partitioned tallies.** Both blockers this paragraph
originally stated dissolved on inspection, and are kept struck rather than deleted:
~~metabrowser confirming the hidden plane~~ (the confirmation arrived — hidden prunes at
scope, now `fdu-xyvu`) and ~~the 14-day cool-off on the `ignore` crate~~ (the cool-off
gates young releases, and `ignore` is mature — the real constraint found by checking is
that its current release needs Rust 1.88, answered by raising the workspace floor to
1.88 rather than holding the crate back).
What remained was a design decision, since taken: see *The tag model, made generic*.

**`fdu-4o0m`, `fdu-m893`, `fdu-ey9q` — the session, progress mode, progressive
goldens.** These sit behind the progressive-results epic (`fdu-wpa0`), which owns the
session type these three present.
Building a progress mode against a session that does not exist yet would mean inventing
the session in the command line, which is the one thing the surface architecture
forbids.

**`fdu-n4gn` — what planes and groups cost.** A loop job needs a quiet host.
Run on a shared runner it measures the runner, which is the same reason no timing gate
is in `make check`.

**`fdu-vfyw` — the reference embedder and cross-engine fixture.** Needs the dual-plane
tallies above and gitignore negations to be meaningful; a fixture that agrees on the
half both engines already implement proves nothing about the half they do not.

Two follow-ups were split out of work that did land, rather than being quietly dropped:

- `fdu-gy3g` — vendoring the File Rollup conformance packet.
  Its cases are matching-only, so they pass against a single extension level and would
  have gone green both before and after `fdu-5q6e`. It needs direct
  basename-to-logical-extension cases from metabrowser first; vendoring it as it stands
  buys a green check that proves nothing.
- `fdu-or38` — the report views still cannot tell a symlink-only directory from an empty
  one, because `SummaryRow` and `TreeNode` carry no leaf count.
  Adding a column to the text table is a command-line display decision and moves every
  golden, so it is worth choosing deliberately rather than inheriting.

## Implementation Plan

One phase, because the sequencing is already carried by bead dependencies and the phases
in the parent spec. The work here is to attach this map to those beads and add the
testing beads it introduces.

Checked items are recorded in **What Landed** above, at the same file-and-function level
as the map they came from.

- [x] Record the file/function map above on each existing bead’s notes so an implementer
  starts from it (`fdu-u7vo` children)
- [ ] `--progress`/`--progress-at` on the Mode axis, the refactor that lets watch and
  progress share one repaint loop, and the `--docs` amendment (`fdu-m893`, blocked by
  `fdu-4o0m`, which is blocked by the progressive-results epic)
- [x] Scripted watch events behind the watch feature gate, with goldens for the
  `InvalidateReason` cases a real filesystem cannot be made to produce (`fdu-0jyz`)
- [x] Counter relations as a golden-visible cost oracle, following the
  `FDU_SCAN_DIAGNOSTICS` precedent (`fdu-524n`)
- [ ] Progressive goldens for both traversal orders, and the tagged fixture the plane
  goldens need (`fdu-ey9q`, blocked by `fdu-m893` and by the tagged planes)

## Testing Strategy

Stated per bead above.
The rules that govern all of it:

- Determinism is injected, never waited for.
  No test sleeps to let a scan progress; it fixes the worker count and the order, or it
  sequences causally on a record that proves the previous step landed.
- Unstable values get a **named pattern**, never a bare `[..]` elision.
  `[CLOCK]`, `[ALLOCATED]`, `[MTIME_NS]`, `[STAMP]`, `[DIR_BYTES]` already exist; new
  ones are declared in the session’s frontmatter.
  `make portability` rejects anything machine-specific that `--update` expanded, which
  is a failure this corpus has had twice.
- Cost assertions are relational, never absolute, and never wall-clock.
- Every new golden is replayed against Python by the parity harness.
  A legitimate difference becomes a declared class in `parity-classes.mjs`; an
  undeclared one fails the build.

## Open Questions

Answered questions are struck through with the answer, rather than deleted: what a
decision replaced is part of the record.

- Should `--progress-at depth` be the only checkpoint that ships?
  `entries:N` is useful to a consumer and awkward in a golden, because the frame count
  then depends on tree size.
- ~~Does the scripted-event source belong behind the `watch` feature gate or behind a
  separate test-only feature?~~ Answered: the `watch` gate.
  A separate feature would mean the scripted backend is not compiled in the
  configuration anyone ships, so the seam it tests would be exercised only in a build no
  user runs — which is how a test-only path drifts from the one it stands in for.
  `WatchBackend` is a plain enum variant beside `Native` and `Poll`, and the gate keeps
  the whole layer deletable exactly as before.
- Owned registry strings cost something against `&'static` ones.
  The measurement is a loop job on PR #38’s own subject, and the answer decides whether
  the compiled default keeps a specialized path.
- Can the progressive frame format double as the reference contract for the embedder’s
  SSE feed? If so the golden corpus and the product artifact are the same thing, which
  would be worth a little awkwardness elsewhere to achieve.

## References

- [The interactive-client contract](plan-2026-08-23-fdu-interactive-client-integration.md)
  — what to build and why; this spec is its implementation map
- [Design principles](../architecture/fdu-design-principles.md) — the axis rules, the
  golden-test contract, and the rule that a document promising behaviour is amended
  rather than quietly broken
- [Surface architecture](../architecture/fdu-surface-architecture.md) — the parity
  harness these tests flow through
- `tests/golden/cli-watch.tryscript.md` and `tests/golden/bin/watch-capture.mjs` — the
  working causal-sequencing pattern this plan extends
- `tbd guidelines golden-testing-guidelines` — stable/unstable classification, mock
  modes, and the hermeticity rule that anything which can vary must be injected or
  filtered

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
