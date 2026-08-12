# The fdu Performance Loop

How we make fdu faster without fooling ourselves.

This is a development workflow, not a CI gate.
Nothing here runs in `make check`, and nothing here blocks a merge.
It exists so that any contributor — human or agent — can pick the loop up months later,
re-run it, and get numbers comparable to the ones already recorded.

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
predicted. So the loop never begins with a change.
It begins with a profile.

The second is believing an improvement that is not there.
A laptop’s timings drift by several percent over the course of a few minutes as thermal
state, background processes, and page-cache warmth move around.
Measuring all of the old build and then all of the new one measures that drift as much
as it measures the code.
So the loop interleaves variants trial by trial and decides on *paired* differences, and
it treats anything under 3% as noise regardless of how good the story is.

## What we measure

A single number cannot tell you whether a change is good.
A change that halves wall time by burning four cores is a different thing from one that
halves wall time by doing less work, and only one of them is still a win on a busy
machine. Every run therefore records all of these, per trial:

| Dimension | Field | What it tells you |
| --- | --- | --- |
| Wall time | `wall_ns` | What the user waits for, including process spawn |
| Component time | `component_ns` | The engine’s own timer around just the measured work |
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
  This is `cold-scan-index` (walk plus index build) and `cold-scan-producer` (walk only,
  which isolates the syscall layer from the index).
- **Warm start** — a compatible snapshot exists and is revalidated against the tree.
  This is `warm-revalidate` (load plus reconcile) and `warm-snapshot-load` (load only).

Note what “cold” does *not* claim.
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

The loop therefore runs against a real directory the operator nominates, and treats it
as immutable and confidential:

- It is never written to.
  Snapshots go to a scratch directory outside it.
- No path from it is ever recorded.
  The tree is identified by `root_id`, the SHA-256 of its absolute path, plus counts,
  byte totals, a depth histogram, and an extension histogram.
- Every trial is checked against an independent oracle.
  A Python walk computes the same `fdu-index-record-v1` digest the engine computes; if a
  trial’s digest disagrees, the sample is marked invalid and kept, never silently
  dropped. A faster number that came from skipping entries is not a faster number.
- The tree is fingerprinted before and after every run.
  If it changed, the run says so and exits nonzero, because timings taken against two
  different trees are not comparable.

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

The accept metric is wall time by default, with one narrow exception: when a hypothesis
*pre-registered* a different signal in the registry before any measurement ran (as the
research backlog does for several), acceptance may be taken on that declared metric,
with the reasoning stated in the record.
A post-hoc metric switch — measuring on wall, missing, and then finding a metric that
passes — is never an accept.
exp-009 is the worked example: its registry row predicted the load *component*, the
component cleared decisively while wall was diluted by probe overhead, and the record
says exactly that.
A 4% win that adds a lock, a thread pool, and two new failure modes is
not automatically worth taking, and the ledger records the reasoning when we decline it.

## Hypotheses

Kept as a live list.
Numbering is shared with the
[performance-frontier research](../research/research-2026-08-10-performance-frontier.md),
whose backlog owns H12–H46; new hypotheses from any source take the next free number
(currently H57) so no id ever means two things.
Each is stated so it can be wrong, with the metric that would show it.
Status is updated as experiments resolve them; see the ledger for results.

### Traversal and syscalls

| # | Hypothesis | Predicted effect | Status |
| --- | --- | --- | --- |
| H1 | The walk is serial, so it uses one core while the machine has ten. A bounded parallel producer feeding a single index consumer will cut wall time several-fold. | `wall_ns` down 3–4×, `cpu_ns` roughly flat or slightly up, `cpu_ns/wall_ns` from 1.0 to 4+ | **Confirmed** (exp-001, −50.0%) |
| H2 | `fs::read_dir` opens each directory by absolute path, so the kernel re-resolves every component from the root. Opening relative to a retained dirfd (`openat`) removes repeated prefix resolution. After H26 landed, `open` is the largest remaining cold cost at **33.86%** of self time. | `system_cpu_ns` down, biggest effect on deep trees | **Refuted for one retained root fd** (exp-024): 720k cold-index wall −0.07% [−4.06%, +1.53%], with indexed and producer system CPU neutral. Parent/ancestor dirfds remain a distinct H24/H29 design problem. |
| H3 | One `fstatat` per entry (20.08% of post-adaptive self time) is the floor for a portable walker, but macOS `getattrlistbulk` batches enumeration and complete stat-tier metadata per directory. | `system_cpu_ns` down substantially | **Confirmed on macOS** (exp-022): 720k cold-index wall −30.13%, producer wall −41.60%, system CPU −46.62%/−61.40%; 60k wall −5.22%/−9.25%. Linux `statx` remains open. |
| H4 | Depth-first traversal order has worse locality than breadth-first for a tree whose directories were written breadth-first. | `system_cpu_ns`, `minor_faults` | — |
| H31 | A latency-bound walk needs more in-flight metadata operations than the six-worker warm-small knee. Calibrating aggregate chunk service time over the first 16k entries should select sixteen workers in the slow state and retain six in the fast state. | slow cold wall down 5–10%; fast wall and resources unchanged | **Confirmed for the portable high-latency path** (exp-015–021): service-time calibration improved 720k cold-index wall 5.31% and producer wall 10.09%. H52 confirms that the same trigger correctly remains at six after bulk metadata removes the per-entry wait. |
| H52 | H26 removed the per-entry metadata wait that made sixteen workers the pre-bulk 720k knee. On the bulk backend, six workers should now match or beat deeper fixed pools while using less CPU and memory, and the H31 service-time trigger should remain inactive. | 6-worker large-tree wall no worse than 8/12/16; CPU, context switches, and RSS lower | **Confirmed** (exp-025): sixteen workers regressed indexed wall 19.19%, producer wall 12.65%, total CPU 107–117%, and RSS about 33%. Eight was neutral in the exploratory curve, and automatic calibration remained at six. |
| H54 | The macOS bulk reader creates and drops one `Vec<Entry>` per directory: 7,350 allocations at 60k and 88,201 at 720k. Retaining the staging vector in each reader and draining it after a successful complete-directory parse should reuse capacity without weakening atomic fallback. | cold/warm `user_cpu_ns` and faults down; at least one primary wall/component down 3%; RSS neutral | **Refuted at 60k** (exp-028): cold-index wall +0.21%, producer +1.32%, warm −0.85%; predicted CPU/fault reductions were absent and producer RSS/faults regressed. Reverted without a 720k run. |
| H55 | A 256 KiB `getattrlistbulk` buffer should reduce repeat calls in wide directories versus H26’s 64 KiB buffer. | cold wall/component and system CPU down at least 3%; warm may compose; account for about 1.1 MiB more capacity across six workers | **Refuted at 60k** (exp-029): cold-index wall -1.80% [−5.95%, +5.45%], producer +1.78%, and warm −0.01%; the syscall/CPU mechanism was absent while cold RSS and faults regressed. Reverted without a 720k run. |

### Index and allocation

| # | Hypothesis | Predicted effect | Status |
| --- | --- | --- | --- |
| H5 | `normalize()` allocates a `Vec<OsString>` per path, and the caller allocates the `PathBuf` it parses. Per entry that is several allocations for information the walker already had. Allocator is 17.6% of baseline self time. | `user_cpu_ns` down, `minor_faults` down | **Confirmed** (exp-004): −9.4% warm revalidate, −17.8% snapshot load. No effect on the cold path, which is syscall-bound. |
| H6 | `merge_upward` walks to the root for every entry, merging a `RollUp` that owns a `BTreeMap<String, ExtTally>`. Extension tallies are merged O(depth) times per file. | `user_cpu_ns` down | **Confirmed via H18** (exp-008): interning the keys to `u32` ids made cold-scan-index 15.7% faster. The per-directory-accumulation half (the registry’s H13) is still open. |
| H7 | Children are `BTreeMap<OsString, EntryId>`; a hash map with a cheap hasher would beat comparison-based lookup at node_modules fan-out. Needs no new dependency: FxHash is ~15 lines. | `user_cpu_ns` down | — |
| H14 | The exclusive reconcile path re-derives child expectations through path joins and root descents that the shared-handle path already reads off entry ids. | warm `user_cpu_ns`, `minor_faults` down | **Confirmed** (exp-007, `92d6212`): first run was underpowered under load-average-17 noise; the quiet 20-trial re-run measured warm-revalidate wall −7.09% [−8.92, −5.76]. |
| H18 | Extension tallies keyed by owned `String` cost ~523k clones and string-keyed descents per 60k scan; interning to `u32` ids makes merges integer work. | cold `user_cpu_ns` down, RSS down | **Confirmed** (exp-008, `bb1529d`): cold-scan-index −15.65% [−32.8, −0.8] even in a noisy run. |
| H32 | The snapshot loader reads the whole image twice — once for CRC, once to parse; folding the digest into the parse removes a full pass. | `warm-snapshot-load` component −15–25% | **Confirmed on its pre-registered signal** (exp-009, `9f4f029`): load component −12.38% [−22.85, −4.71] on the quiet re-run; wall diluted by probe spawn and oracle overhead, per the accept-rule exception above. |
| H17 | Replacing the transient per-directory expectation map with a sorted claim-list (and deferring path joins) removes per-entry allocation on the warm sweep. | warm `user_cpu_ns`, `minor_faults` down | **Refuted at 60k-warm** (exp-010): −0.03% [−1.37, +1.64]. After H14 the map already read straight off entry ids; the residue is noise next to one `fstatat` per entry. |
| H13 | Accumulating consecutive same-parent insert contributions and merging once per run cuts upward merges ~7×. | cold `user_cpu_ns` down | **Refuted after H18** (exp-011): −2.53% [−8.39, +0.23]. Interning had already removed the expensive part of each merge — the two hypotheses competed for the same cost, and interning captured it alone. Re-test if content-tier reducers make roll-ups heavy again. |
| H8 | The observation batch allocates a `PathBuf` per op and then clones it. Moving instead of cloning removes one allocation per entry. | `user_cpu_ns`, `minor_faults` | **Refuted** (exp-003): removing ~120,000 clones per scan changed nothing measurable. The allocator cost is in the producer, not in apply. |
| H51 | The portable producer clones every relative `PathBuf` into its observation even though non-directory entries can transfer ownership; retaining a second path is necessary only for directories added to the frontier. | cold producer `user_cpu_ns`, `minor_faults`, then wall | **Refuted** (exp-016): cold-index wall −0.44% [−5.30%, +1.52%], with CPU also unclear; peak RSS and minor faults instead regressed about 4%. The allocator can reuse the short-lived original buffer, so moving it changes which allocation stays live without reducing measured work. |

### Warm start

| # | Hypothesis | Predicted effect | Status |
| --- | --- | --- | --- |
| H9 | Warm revalidation is currently *slower* than a cold scan. Reconciliation does a full walk plus expectation lookups plus a snapshot load, so the cache costs more than it saves. | `warm-revalidate` wall below `cold-scan-index` wall | **Open, narrowed to snapshot load.** H12/exp-030 brings the verified 60k warm open from about 508 ms to 351 ms, versus about 296 ms cold; reconciliation itself is now about 151 ms, so persisted roll-ups/bulk load own most of the remaining gap. |
| H12 | After H14 elided exclusive no-op applies and H26/H53 batched metadata, workers can compare bounded directory waves against one immutable baseline and send only changes through the delta contract. This revisits exp-002 without its single-consumer funnel. | 60k warm wall down at least 15% with its interval below zero; reconciliation component down at least 25%; exact parity and RSS increase no greater than 10% | **Confirmed** (exp-030): four-worker waves improve warm wall 30.25% at 60k and 59.53% at 720k; reconciliation component falls 50.31%/72.55%, 60k RSS rises 3.29%, and large RSS improves 0.99%. Shared/scoped/one-worker paths retain serial arbitration. |
| H56 | exp-030’s post-profile attributes about 13% of 60k warm samples to scoped thread startup/waiting. Quadrupling the directory wave should amortize that residue while keeping both deferred changes and progressive publication bounded. | 60k warm wall or component down at least 3% with its interval below zero; RSS increase no greater than 5%; exact parity | **Ready**: compare 4,096 with 1,024 directories per wave at the accepted four-worker depth; confirm at 720k only if the 60k gate passes. |
| H10 | Snapshot load is ~320 ms of the warm start. A format whose on-disk layout can be used without rebuilding the tree would make the warm path open-latency-bound instead of parse-bound. | `warm-snapshot-load` wall down | **Partly addressed** (exp-005): the loader was resolving each entry’s path from the root three times over although the parent id was in hand. The format change itself remains open. |
| H11 | `revalidate` builds a `BTreeSet<OsString>` of seen names per directory, cloning every name. Comparing against the index’s existing sorted children directly would remove that. | `user_cpu_ns` down on warm jobs | — |
| H53 | Full reconciliation still uses portable enumeration plus one `fstatat` per entry even though H26’s audited macOS reader returns the same complete stat-tier contract in bulk. Reusing it per directory should remove the warm profile’s 29.25% `fstatat` and 6.76% `getdirentries64` costs while preserving complete-directory fallback. | `warm-revalidate` wall and component down at least 3%; `system_cpu_ns` down; oracle parity at 60k and, if scale-sensitive, 720k | **Confirmed on macOS** (exp-026): warm wall −18.97% at 60k and −34.39% at 720k; large component −39.05%, CPU −44.06%, system CPU −53.97%, RSS neutral. Direct, shared, and scoped reconciliation reuse the existing reader. |

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
project’s usual arguments; `PERF_TREE` selects the tree.

Results land in `benchmarks/results/realtree/`, which is gitignored — they are
machine-specific and large.
What gets committed is the ledger entry: the numbers that mattered, the verdict, and the
reasoning.

### Comparing against other tools

`--reference dust=/path/to/dust` measures a third-party tool on the same tree in the
same run. These numbers live in their own table and never enter the accept rule, because
the tools answer slightly different questions with different guarantees.
What they are for is calibration: they establish what a mature tool achieves on this
hardware, which is the only thing that makes an fdu number mean anything.

`dust` is the most useful comparison because it solves the same problem — walk a tree,
roll up sizes — with a well-tuned parallel walker.
Its source is worth reading; check it out into `attic/` (gitignored) and note that it is
Apache-2.0, so designs may be described and reimplemented but not copied.

* * *

*Part of the fdu project documentation.
See [AGENTS.md](../../../AGENTS.md).*
