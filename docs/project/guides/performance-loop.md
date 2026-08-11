# The fdu Performance Loop

How we make fdu faster without fooling ourselves.

This is a development workflow, not a numeric CI gate.
Nothing here runs in `make check`, and the hosted environment run is an on-demand
evidence producer rather than a pass/fail speed threshold.
It exists so that any contributor — human or agent — can pick the loop up months later,
re-run it, and get numbers comparable to the ones already recorded.

The companion documents are the
[experiment ledger](../reports/report-2026-08-10-fdu-performance-experiments.md), which
records every experiment and its verdict, and the
[end-to-end performance plan](../specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md),
which owns the generated-corpus evidence harness this loop borrows from.

The current cumulative result is exp-012. Against the correctness-normalized PR base,
the reviewed candidate improved cold-scan wall time by 50.95% while regressing total CPU
by 83.11%; the latency gate passed and the resource gate failed, so the experiment is
rejected as a universal performance win.
Warm revalidation and snapshot load both improved in that same 60k-entry APFS cell.
The separate 10k–1M curve validates loader correctness and near-linear fanout behavior
only; it does not select the cache read path.

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
| Blocked residual | `blocked_ns` | `wall - process CPU` for known single-threaded jobs only; null for parallel jobs |
| Peak RSS | `peak_rss_bytes` | Whether a speedup was bought with memory |
| Page faults | `minor_faults`, `major_faults` | Allocation churn; major faults mean real disk |
| Context switches | `voluntary_*`, `involuntary_*` | Contention, and whether threads are thrashing |

`cpu_ns / wall_ns` is the parallelism actually achieved, and it is the number that
explains most surprises.
Process CPU accumulates across threads, so subtracting it from wall time is not an
off-CPU measurement when a job can run in parallel.
Those jobs record `blocked_ns: null` unless the harness gains a real off-CPU collector.

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

Representative optimization work therefore runs against a real directory the operator
nominates and treats it as immutable and confidential:

- It is never written to.
  Snapshots go to a scratch directory outside it.
- No path from it is ever recorded.
  The ephemeral run uses `root_id`, the SHA-256 of its absolute path, only to detect a
  different local root.
  Archiving replaces that value with an identity derived from the content digest and
  replaces the operator label with a public label.
- Every trial is checked against an independent oracle.
  A Python walk computes the same `fdu-index-record-v1` entry digest and the same
  `fdu-rollup-record-v1` digest across every directory, including newest mtime and named
  extension tallies. The producer digest covers the records emitted by that invocation
  and rejects duplicate paths; it is not borrowed from a second scan.
  If a digest disagrees, the sample is marked invalid and kept, never silently dropped.
  A faster number that came from skipping entries or corrupting a reducer is not a
  faster number.
- The tree is fingerprinted before and after every run.
  If it changed, the run says so and exits nonzero, because timings taken against two
  different trees are not comparable.

Generated corpora serve the complementary purpose: they make the *same workload*
recreatable on another filesystem. Their portable semantic digest excludes inode,
ctime, device, and allocated blocks, while each invocation retains a second engine
digest with those local fields for the exact probe oracle. A generated result does not
replace the real-tree result; it isolates the environment axis that a private laptop
tree cannot.

## Cross-environment cells

Every v3 run records two environment identities:

- a logical cell such as `local-macos-apfs-arm64` or
  `github-ubuntu-24.04-x64`, used to group repeated evidence; and
- an exact SHA-256 over the path-free host, compiler, runner image, CPU, memory,
  architecture, operating system, and filesystem facts, used to detect drift within
  that logical cell.

A decision matrix accepts runs only when their portable corpus identity, engine and
probe revisions, variant flags, full job contracts, trial count, warmups, schedule,
and page-cache condition match. Binary hashes and target triples are expected to differ
across architectures. Absolute medians remain inside their own cell; the matrix compares
only whether each cell passed the latency, CPU, RSS, and overall gates.

Runner control is part of the claim. Local developer runs are `local-uncontrolled`.
GitHub-hosted runners are `shared-cloud-exploratory`: their fresh VM gives a useful
Linux/filesystem counterexample, but neighboring load and the underlying hardware are
not controlled. GitHub documents the current hosted-runner hardware separately in its
[runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).
A platform default requires repetition on a `self-hosted-controlled` cell; a hosted
result can support or challenge a hypothesis but cannot promote itself to product
policy.

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
- CPU and peak-RSS regressions each stay within the default 10% guardrail, or the record
  carries an explicit product rationale waiving that guardrail, and
- the complexity is worth it.

The latency arithmetic lives in
[`benchmarks/realtree/ledger.py`](../../../benchmarks/realtree/ledger.py), and recording
enforces the resource and evidence gates in
[`benchmarks/realtree/record.py`](../../../benchmarks/realtree/record.py).
The fifth is a judgment and is written down as one.
Latency and resource guardrails are recorded separately: a latency win bought with twice
the CPU is never hidden by a single “accepted” label.

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
(currently H48) so no id ever means two things.
Each is stated so it can be wrong, with the metric that would show it.
Status is updated as experiments resolve them; see the ledger for results.

### Traversal and syscalls

| # | Hypothesis | Predicted effect | Status |
| --- | --- | --- | --- |
| H1 | The walk is serial, so it uses one core while the machine has ten. A bounded parallel producer feeding a single index consumer will cut wall time several-fold. | `wall_ns` down 3–4×, `cpu_ns` roughly flat or slightly up, `cpu_ns/wall_ns` from 1.0 to 4+ | **Latency confirmed, universal win rejected.** In exp-012 the repaired implementation cut cold-scan wall 50.95% but raised CPU 83.11%, failing the default resource guardrail. |
| H2 | `fs::read_dir` opens each directory by absolute path, so the kernel re-resolves every component from the root. Opening relative to the parent’s fd (`openat`) would resolve one component. After H1 landed, `open` is the single largest cost in the cold profile at **28%** of self time. | `system_cpu_ns` down, biggest effect on deep trees | **Blocked**: needs `libc` as a runtime dependency and a scoped `unsafe` allowance. Not a decision to take without the supply-chain review in [SUPPLY-CHAIN-SECURITY.md](../../../SUPPLY-CHAIN-SECURITY.md). |
| H3 | One `fstatat` per entry (19% of self time after H1) is the floor for a portable walker, but macOS `getattrlistbulk` and Linux `statx` batch metadata per directory. | `system_cpu_ns` down substantially | **Blocked** on the same dependency decision as H2. |
| H4 | Depth-first traversal order has worse locality than breadth-first for a tree whose directories were written breadth-first. | `system_cpu_ns`, `minor_faults` | — |

### Index and allocation

| # | Hypothesis | Predicted effect | Status |
| --- | --- | --- | --- |
| H5 | `normalize()` allocates a `Vec<OsString>` per path, and the caller allocates the `PathBuf` it parses. Per entry that is several allocations for information the walker already had. Allocator is 17.6% of baseline self time. | `user_cpu_ns` down, `minor_faults` down | **Implemented; standalone evidence superseded.** exp-004 used the v1 oracle and remains audit history, not a current isolated claim. |
| H6 | `merge_upward` walks to the root for every entry, merging a `RollUp` that owns a `BTreeMap<String, ExtTally>`. Extension tallies are merged O(depth) times per file. | `user_cpu_ns` down | **Implemented via H18; standalone evidence superseded.** The private integer representation now has a named public boundary and reclamation, so the reviewed implementation differs materially from exp-008. |
| H7 | Children are `BTreeMap<OsString, EntryId>`; a hash map with a cheap hasher would beat comparison-based lookup at node_modules fan-out. Needs no new dependency: FxHash is ~15 lines. | `user_cpu_ns` down | — |
| H14 | The exclusive reconcile path re-derives child expectations through path joins and root descents that the shared-handle path already reads off entry ids. | warm `user_cpu_ns`, `minor_faults` down | **Implemented; standalone evidence superseded.** exp-007 remains a useful exploratory result but predates v2 provenance and the full reducer oracle. |
| H18 | Extension tallies keyed by owned `String` cost ~523k clones and string-keyed descents per 60k scan; interning to `u32` ids makes merges integer work. | cold `user_cpu_ns` down, RSS down | **Implemented with a repaired public contract; standalone evidence superseded.** exp-008 measured the unsafe API shape and is not transferable to the reviewed implementation. |
| H32 | The snapshot loader reads the whole image twice — once for CRC, once to parse; folding the digest into the parse removes a full pass. | `warm-snapshot-load` component −15–25% | **Implemented; standalone evidence superseded.** exp-009 predated committed raw v2 provenance; the true-base cumulative run is the current evidence. |
| H17 | Replacing the transient per-directory expectation map with a sorted claim-list (and deferring path joins) removes per-entry allocation on the warm sweep. | warm `user_cpu_ns`, `minor_faults` down | **Refuted at 60k-warm** (exp-010): −0.03% [−1.37, +1.64]. After H14 the map already read straight off entry ids; the residue is noise next to one `fstatat` per entry. |
| H13 | Accumulating consecutive same-parent insert contributions and merging once per run cuts upward merges ~7×. | cold `user_cpu_ns` down | **Refuted after H18** (exp-011): −2.53% [−8.39, +0.23]. Interning had already removed the expensive part of each merge — the two hypotheses competed for the same cost, and interning captured it alone. Re-test if content-tier reducers make roll-ups heavy again. |
| H8 | The observation batch allocates a `PathBuf` per op and then clones it. Moving instead of cloning removes one allocation per entry. | `user_cpu_ns`, `minor_faults` | **Refuted** (exp-003): removing ~120,000 clones per scan changed nothing measurable. The allocator cost is in the producer, not in apply. |

### Warm start

| # | Hypothesis | Predicted effect | Status |
| --- | --- | --- | --- |
| H9 | Warm revalidation is currently *slower* than a cold scan. Reconciliation does a full walk plus expectation lookups plus a snapshot load, so the cache costs more than it saves. | `warm-revalidate` wall below `cold-scan-index` wall | **Open, and the largest outstanding defect.** exp-002 refuted the obvious fix: parallelising the sweep gained only 2.6%, because the warm path is bound by the single index consumer, not by traversal. |
| H10 | Snapshot load is ~320 ms of the warm start. A format whose on-disk layout can be used without rebuilding the tree would make the warm path open-latency-bound instead of parse-bound. | `warm-snapshot-load` wall down | **Partly addressed; old speed claim superseded.** Direct parent/name lookup removes the reviewed fanout-quadratic path; the new 10k–1M wide curve is the topology evidence, while the format change remains open. |
| H11 | `revalidate` builds a `BTreeSet<OsString>` of seen names per directory, cloning every name. Comparing against the index’s existing sorted children directly would remove that. | `user_cpu_ns` down on warm jobs | — |

### Rejected or superseded

Recorded in the ledger with the numbers that killed them.

## Running it

Prerequisites: a release probe, a profiling probe, a nominated tree, and—for a
claim-grade comparison—one provenance manifest per binary.
A manifest pins the engine revision separately from the probe revision and digest,
target, release profile, feature set, and a path-redacted build command.
This separation is what lets the same v2 semantic probe be compiled against an older
engine without pretending the older tree contained the newer harness.

```shell
# One-time: record what the tree looked like, so later runs can prove it is the same.
uv run --project benchmarks/realtree --frozen python -m benchmarks.realtree baseline \
  --root /path/to/tree --label mytree

# Profile: where does the time actually go?
cargo build --locked --profile profiling -p fdu --example perf_probe --no-default-features
uv run --project benchmarks/realtree --frozen python -m benchmarks.realtree profile \
  --root /path/to/tree --job cold-scan-index --label baseline

# Measure: is the candidate faster than the control?
cargo build --locked --release -p fdu --example perf_probe --no-default-features
uv run --project benchmarks/realtree --frozen python -m benchmarks.realtree measure \
  --root /path/to/tree --label mytree \
  --variant control=/tmp/perf_probe.control \
  --variant candidate=target/release/examples/perf_probe \
  --variant-metadata control=/tmp/control-build.json \
  --variant-metadata candidate=/tmp/candidate-build.json \
  --job cold-scan-index --job warm-revalidate \
  --trials 12 --baseline-fingerprint benchmarks/results/realtree/tree-mytree.json \
  --name exp007-parallel-producer

# Cross-environment run: generate the same portable workload on every host, then pass
# the printed run_root to measure. Use the same seed, entries, run group, jobs, and
# trial schedule in every cell.
uv run --no-project python -m benchmarks.generate create \
  --recipe balanced --entries 60000 --seed pr3-cache-cross-environment-v1 \
  --work-dir /path/to/owned-scratch
uv run --project benchmarks/realtree --frozen python -m benchmarks.realtree measure \
  --root /printed/run_root/corpus --label generated-balanced-60k \
  --corpus-manifest /printed/run_root/observed-corpus.json \
  --variant corrected=/path/to/corrected --variant candidate=/path/to/candidate \
  --variant-metadata corrected=/path/to/corrected.json \
  --variant-metadata candidate=/path/to/candidate.json \
  --job cold-scan-index --job warm-revalidate --trials 12 --warmups 3 \
  --environment-cell local-macos-apfs-arm64 \
  --runner-class local-uncontrolled --run-group pr3-cache-balanced-60k-v1 \
  --name macos-local

# After archiving each raw run, compare per-cell decisions. This fails closed if any
# supposedly equivalent workload, source, flag, job, or schedule field differs.
uv run --project benchmarks/realtree --frozen python -m benchmarks.realtree \
  environment-matrix \
  --run docs/project/experiments/evidence/pr3-cache-macos-local-run.json \
  --run docs/project/experiments/evidence/pr3-cache-linux-cloud-run.json \
  --id env-001 --control-variant corrected --candidate-variant candidate \
  --output docs/project/experiments/evidence/env-001-matrix.json \
  --report benchmarks/results/realtree/env-001-matrix.md

# Archive and record the exact paired samples. `record` refuses an accepted result
# without v2/v3 provenance, an unchanged tree, zero invalid samples, a significant 3%
# latency win, and passed or explicitly waived CPU/RSS guardrails.
uv run --project benchmarks/realtree --frozen python -m benchmarks.realtree record \
  --run benchmarks/results/realtree/run-exp007-parallel-producer.json \
  --id exp-007 --title "Parallel producer" --hypothesis H1 \
  --control "base revision" --candidate "candidate revision" \
  --control-variant control --candidate-variant candidate \
  --decision rejected --primary-job cold-scan-index \
  --reason "State the evidence-backed disposition"
```

`make perf-baseline`, `make perf-profile`, and `make perf-compare` wrap these with the
project’s usual arguments; `PERF_TREE` selects the tree.

Working results land in `benchmarks/results/realtree/`, which is gitignored.
A reviewed claim archives a path-scrubbed immutable copy under
`docs/project/experiments/evidence/`; the experiment records its SHA-256, exact expanded
schedule digest, raw paired samples, invalid-sample reasons, binary identities, and
toolchain. `make perf-ledger` resolves and verifies that bundle before rendering it.
Legacy v1 bundles remain available for audit history but cannot validate as accepted or
become the current cumulative headline.

The `Cache performance environments` GitHub Actions workflow reruns the frozen
generated-corpus comparison on Ubuntu 24.04 and retains the raw trials as a downloadable
artifact. After it is merged to the default branch, use its manual dispatch for future
Linux repetitions. Artifacts are convenient transport, not the durable record: a
reviewed result is still archived under `docs/project/experiments/evidence/` with its
content digest.

The raw producer diagnostic is `cold-scan-producer-raw`. It intentionally omits the
semantic digest and can attribute oracle overhead, but it is never claim-grade
correctness evidence.

Snapshot fanout claims also require the deterministic scale curve:

```shell
uv run --project benchmarks/realtree --frozen python -m benchmarks.realtree snapshot-scale \
  --variant candidate=target/release/examples/perf_probe \
  --variant-metadata candidate=/tmp/candidate-build.json \
  --work-dir benchmarks/corpus/snapshot-scale \
  --output docs/project/experiments/evidence/snapshot-load-wide-scale-v1.json
```

This generates and removes one wide corpus at a time at 10k, 100k, 500k, and 1M, and
accepts a row only when the loaded snapshot agrees with both independent v2 digests.

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

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
