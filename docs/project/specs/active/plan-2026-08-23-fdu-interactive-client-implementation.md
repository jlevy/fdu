# Feature: Implementing the Interactive-Client Contract — Files, Functions, and How It Is Tested

**Date:** 2026-08-23

**Author:** fdu project

**Status:** Draft

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

## Implementation Plan

One phase, because the sequencing is already carried by bead dependencies and the phases
in the parent spec. The work here is to attach this map to those beads and add the
testing beads it introduces.

- [ ] Record the file/function map above on each existing bead’s notes so an implementer
  starts from it (`fdu-u7vo` children)
- [ ] `--progress`/`--progress-at` on the Mode axis, the refactor that lets watch and
  progress share one repaint loop, and the `--docs` amendment (`fdu-m893`, blocked by
  `fdu-4vkz` and `fdu-4o0m`)
- [ ] Scripted watch events behind the watch feature gate, with goldens for the
  `InvalidateReason` cases a real filesystem cannot be made to produce (`fdu-0jyz`)
- [ ] Counter relations as a golden-visible cost oracle, following the
  `FDU_SCAN_DIAGNOSTICS` precedent (`fdu-524n`)
- [ ] Progressive goldens for both traversal orders, and the tagged fixture the plane
  goldens need (`fdu-ey9q`, blocked by the two above and by `fdu-4vkz`)

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

- Should `--progress-at depth` be the only checkpoint that ships?
  `entries:N` is useful to a consumer and awkward in a golden, because the frame count
  then depends on tree size.
- Does the scripted-event source belong behind the `watch` feature gate or behind a
  separate test-only feature?
  The gate keeps the layer deletable, which argues for the former; a test-only feature
  keeps a scripted-event path out of released binaries entirely, which argues for the
  latter.
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
