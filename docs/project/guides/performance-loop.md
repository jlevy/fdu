# The fdu Performance Loop

How we make fdu faster without fooling ourselves.

This is a development workflow, not a CI gate.
Nothing here runs in `make check`, and nothing here blocks a merge.
It exists so that any contributor — human or agent — can pick the loop up months
later, re-run it, and get numbers comparable to the ones already recorded.

The companion documents are the
[experiment ledger](../reports/report-2026-08-10-fdu-performance-experiments.md), which
records every experiment and its verdict, and the
[end-to-end performance plan](../specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md),
which owns the generated-corpus evidence harness this loop borrows from.

## Why a loop and not a list of optimizations

Two failure modes make performance work worthless, and both are easy to fall into.

The first is optimizing what is not slow.
Intuition about where a filesystem walker spends its time is reliably wrong; the
baseline profile for this project found 20% of the time in `open`, which nobody
predicted.
So the loop never begins with a change. It begins with a profile.

The second is believing an improvement that is not there.
A laptop's timings drift by several percent over the course of a few minutes as
thermal state, background processes, and page-cache warmth move around.
Measuring all of the old build and then all of the new one measures that drift as
much as it measures the code.
So the loop interleaves variants trial by trial and decides on *paired* differences,
and it treats anything under 3% as noise regardless of how good the story is.

## What we measure

A single number cannot tell you whether a change is good.
A change that halves wall time by burning four cores is a different thing from one
that halves wall time by doing less work, and only one of them is still a win on a
busy machine.
Every run therefore records all of these, per trial:

| Dimension | Field | What it tells you |
| --- | --- | --- |
| Wall time | `wall_ns` | What the user waits for, including process spawn |
| Component time | `component_ns` | The engine's own timer around just the measured work |
| Total CPU | `cpu_ns` | user + system; the real cost of the work |
| User CPU | `user_cpu_ns` | Time in our code: allocation, hashing, tree building |
| System CPU | `system_cpu_ns` | Time in the kernel: `getdirentries`, `stat`, `open` |
| Blocked | `blocked_ns` | `wall - cpu`; I/O waits and scheduler delay |
| Peak RSS | `peak_rss_bytes` | Whether a speedup was bought with memory |
| Page faults | `minor_faults`, `major_faults` | Allocation churn; major faults mean real disk |
| Context switches | `voluntary_*`, `involuntary_*` | Contention, and whether threads are thrashing |

`cpu_ns / wall_ns` is the parallelism actually achieved, and it is the number that
explains most surprises.

Two axes cross these dimensions:

- **Cold start** — no fdu snapshot exists.
  Everything must come from the filesystem.
  This is `cold-scan-index` (walk plus index build) and `cold-scan-producer` (walk
  only, which isolates the syscall layer from the index).
- **Warm start** — a compatible snapshot exists and is revalidated against the tree.
  This is `warm-revalidate` (load plus reconcile) and `warm-snapshot-load` (load
  only).

Note what "cold" does *not* claim.
The operating system page cache is warm in both cases.
Dropping it on macOS requires root, so a run that does not pass `--purge` records
`os_cache: warm-steady` and means exactly that.
Cold-page-cache numbers measure the SSD, which is not the thing we are trying to
improve.

## The reference tree

Timings against a generated corpus answer a different question than timings against a
real one.
Generated corpora are uniform; real trees have a `node_modules` with 6,800 tiny
JavaScript files at depth 12 next to a `.git` with a handful of large packs, and the
distribution is what stresses a walker.

The loop therefore runs against a real directory the operator nominates, and treats
it as immutable and confidential:

- It is never written to.
  Snapshots go to a scratch directory outside it.
- No path from it is ever recorded.
  The tree is identified by `root_id`, the SHA-256 of its absolute path, plus counts,
  byte totals, a depth histogram, and an extension histogram.
- Every trial is checked against an independent oracle.
  A Python walk computes the same `fdu-index-record-v1` digest the engine computes;
  if a trial's digest disagrees, the sample is marked invalid and kept, never
  silently dropped.
  A faster number that came from skipping entries is not a faster number.
- The tree is fingerprinted before and after every run.
  If it changed, the run says so and exits nonzero, because timings taken against
  two different trees are not comparable.

## The loop

```
1. PROFILE   — profiling build, symbols, --repeat, sampling profiler.
               Produces self-time by layer. No timing claim.
2. HYPOTHESIS— write down what you think is slow, why, and what you expect
               to happen to which metric. Before the change.
3. CHANGE    — smallest diff that tests the hypothesis. One idea per experiment.
4. MEASURE   — release build, --repeat 1, interleaved against the control,
               >= 12 paired trials.
5. DECIDE    — the accept rule below. Record the verdict either way.
6. COMMIT    — accepted changes commit alone, with their numbers in the message.
               Rejected changes are reverted and recorded in the ledger.
```

Steps 1 and 4 use different builds on purpose.
Profiling needs symbols and thousands of stacks, which means a `profiling` build
repeating the work in one warm process.
Timing needs the artifact a user actually gets, which means an unmodified `release`
build doing the work exactly once.
Mixing them produces numbers that describe neither.

### The accept rule

A candidate is accepted when, on paired wall-time differences at equal ordinals:

- the median change is at least **3% faster**, and
- the 95% bootstrap interval of that change lies entirely below zero, and
- no sample was invalidated by the oracle, and
- the complexity is worth it.

The first three are arithmetic and live in
[`benchmarks/realtree/ledger.py`](../../../benchmarks/realtree/ledger.py).
The fourth is a judgment and is written down as one.
A 4% win that adds a lock, a thread pool, and two new failure modes is not
automatically worth taking, and the ledger records the reasoning when we decline it.

## Hypotheses

Kept as a live list. Each is stated so it can be wrong, with the metric that would
show it. Status is updated as experiments resolve them; see the ledger for results.

### Traversal and syscalls

| # | Hypothesis | Predicted effect | Status |
| --- | --- | --- | --- |
| H1 | The walk is serial, so it uses one core while the machine has ten. A bounded parallel producer feeding a single index consumer will cut wall time several-fold. | `wall_ns` down 3–4×, `cpu_ns` roughly flat or slightly up, `cpu_ns/wall_ns` from 1.0 to 4+ | — |
| H2 | `fs::read_dir` opens each directory by absolute path, so the kernel re-resolves every component from the root. Opening relative to the parent's fd (`openat`) would resolve one component. Baseline profile shows `open` at 20% of self time. | `system_cpu_ns` down, biggest effect on deep trees | — |
| H3 | One `fstatat` per entry (22% of self time) is the floor for a portable walker, but macOS `getattrlistbulk` and Linux `statx` batch metadata per directory. | `system_cpu_ns` down substantially | — |
| H4 | Depth-first traversal order has worse locality than breadth-first for a tree whose directories were written breadth-first. | `system_cpu_ns`, `minor_faults` | — |

### Index and allocation

| # | Hypothesis | Predicted effect | Status |
| --- | --- | --- | --- |
| H5 | `normalize()` allocates a `Vec<OsString>` per path, and the caller allocates the `PathBuf` it parses. Per entry that is several allocations for information the walker already had. Allocator is 17.6% of baseline self time. | `user_cpu_ns` down, `minor_faults` down | — |
| H6 | `merge_upward` walks to the root for every entry, merging a `RollUp` that owns a `BTreeMap<String, ExtTally>`. Extension tallies are merged O(depth) times per file. | `user_cpu_ns` down | — |
| H7 | Children are `BTreeMap<OsString, EntryId>`; a hash map with a cheap hasher would beat comparison-based lookup at node_modules fan-out. Needs no new dependency: FxHash is ~15 lines. | `user_cpu_ns` down | — |
| H8 | The observation batch allocates a `PathBuf` per op and then clones it. Moving instead of cloning removes one allocation per entry. | `user_cpu_ns`, `minor_faults` | — |

### Warm start

| # | Hypothesis | Predicted effect | Status |
| --- | --- | --- | --- |
| H9 | Warm revalidation is currently *slower* than a cold scan (762 ms vs 565 ms measured). Reconciliation does a full walk plus expectation lookups plus a snapshot load, so the cache costs more than it saves. | `warm-revalidate` wall below `cold-scan-index` wall | — |
| H10 | Snapshot load is ~320 ms of the warm start. A format whose on-disk layout can be used without rebuilding the tree would make the warm path open-latency-bound instead of parse-bound. | `warm-snapshot-load` wall down | — |
| H11 | `revalidate` builds a `BTreeSet<OsString>` of seen names per directory, cloning every name. Comparing against the index's existing sorted children directly would remove that. | `user_cpu_ns` down on warm jobs | — |

### Rejected or superseded

Recorded in the ledger with the numbers that killed them.

## Running it

Prerequisites: a release probe, a profiling probe, and a nominated tree.

```shell
# One-time: record what the tree looked like, so later runs can prove it is the same.
uv run --no-project python -m benchmarks.realtree baseline \
  --root /path/to/tree --label mytree

# Profile: where does the time actually go?
cargo build --locked --profile profiling -p fdu --example perf_probe --no-default-features
uv run --no-project python -m benchmarks.realtree profile \
  --root /path/to/tree --job cold-scan-index --label baseline

# Measure: is the candidate faster than the control?
cargo build --locked --release -p fdu --example perf_probe --no-default-features
uv run --no-project python -m benchmarks.realtree measure \
  --root /path/to/tree --label mytree \
  --variant control=/tmp/perf_probe.control \
  --variant candidate=target/release/examples/perf_probe \
  --job cold-scan-index --job warm-revalidate \
  --trials 12 --baseline-fingerprint benchmarks/results/realtree/tree-mytree.json \
  --name exp007-parallel-producer
```

`make perf-baseline`, `make perf-profile`, and `make perf-compare` wrap these with the
project's usual arguments; `PERF_TREE` selects the tree.

Results land in `benchmarks/results/realtree/`, which is gitignored — they are
machine-specific and large.
What gets committed is the ledger entry: the numbers that mattered, the verdict, and
the reasoning.

### Comparing against other tools

`--reference dust=/path/to/dust` measures a third-party tool on the same tree in the
same run.
These numbers live in their own table and never enter the accept rule, because the
tools answer slightly different questions with different guarantees.
What they are for is calibration: they establish what a mature tool achieves on this
hardware, which is the only thing that makes an fdu number mean anything.

`dust` is the most useful comparison because it solves the same problem — walk a tree,
roll up sizes — with a well-tuned parallel walker.
Its source is worth reading; check it out into `attic/` (gitignored) and note that it
is Apache-2.0, so designs may be described and reimplemented but not copied.

---

*Part of the fdu project documentation. See [AGENTS.md](../../../AGENTS.md).*
