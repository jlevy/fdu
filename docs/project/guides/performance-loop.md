# The fdu Performance Loop

How we make fdu faster without fooling ourselves.

This is a development workflow, not a CI gate.
Nothing here runs in `make check`, and nothing here blocks a merge.
It exists so that any contributor — human or agent — can pick the loop up months later,
re-run it, and get numbers comparable to the ones already recorded.

New here? Start with
[the campaign status report](../reports/report-2026-08-14-performance-campaign-status.md),
which assumes no context and covers what has been achieved, what remains, and where the
evidence is weak. This document is the protocol it describes.

The companion documents are the
[experiment ledger](../reports/report-2026-08-10-fdu-performance-experiments.md), which
records every experiment and its verdict, the
[platform tuning guide](platform-tuning.md), which records which regime each tuning
constant was measured in and therefore where it is evidence, and the
[end-to-end performance plan](../specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md),
which owns the generated-corpus evidence harness this loop borrows from.

The division between them is that this guide owns the protocol, the ledger owns the
results and is regenerated from artifacts rather than edited, and the platform guide
owns the mapping from a shipped constant back to the run that chose it.

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

The names “cold scan” and “warm start” describe **fdu snapshot state**, not the
operating-system filesystem cache.
Disk-usage tools read directory and inode/vnode metadata rather than file payloads, so
“disk-cache warmth” here mostly means namespace and metadata state, not cached file
contents.

The local optimization loop and published M1 comparison use `os_cache: warm-steady`. The
comparison establishes that state with one complete independent fingerprint walk, then
at least one full-tree warmup per tool; the definitive run used three.
It means a repeated workload after explicit warmup.
It does **not** claim every metadata object remains resident: the near-million-entry
subject is larger than the host’s vnode target, and macOS may recycle metadata while a
scan is in progress.
That capacity-pressured steady state is representative of rerunning a CLI over a large
working tree and is the right primary regime for judging user-space and syscall
overhead.

Controlled-cold results answer a separate question: first access after boot or eviction,
cache pressure beyond RAM, and remote or provisioned storage.
They may change both absolute time and relative rankings, so no warm result is
extrapolated into a cold claim.
The future Linux product matrix measures both states because Linux offers a repeatable
privileged control. Following the useful part of
[diskus’s published protocol](https://github.com/sharkdp/diskus/blob/90196e950017d25b2940e8e0fda51a321ca66e1a/README.md#benchmark),
each controlled-cold sample runs `sync` and writes `3` to `/proc/sys/vm/drop_caches`; a
separate verified-warm job performs explicit warmups.
Those labels remain invalid unless the runner records preparation success per sample.
The paired schedule, exact oracle, immutable fingerprint, work classes, binary and host
provenance, and raw resource metrics still apply in both regimes.

On macOS, `/usr/sbin/purge` only promises to *approximate* initial-boot buffer-cache
conditions. A purge-cold run is therefore a separately labeled diagnostic, not
controlled-cold release evidence; a dedicated APFS test volume remounted between samples
is the stronger future protocol.

A third axis crosses these two: whether the host is bare metal or virtualized, recorded
as `host_virtualization`. It changes what a *cold* sample can mean and nothing else.
Virtualization has not been measured to distort user-space cost, syscall cost,
allocation, or thread scheduling, so a **warm** result from a VM is ordinary evidence
about the environment most fdu runs happen in — a container, a CI job, a cloud instance,
a WSL session — and is not second-class for being virtual.
What a hypervisor does distort is the storage beneath the guest: its page cache sits
under the guest’s, so writing `3` to `drop_caches` inside the guest does not reach the
disk.
That makes exactly one class of claim untestable on a VM — anything whose mechanism
is device latency or I/O ordering, which is why H73 and the queue-depth hypotheses are
marked as needing bare metal and the io_uring results were not treated as settling the
cold question. All three axes belong in every recorded result; the ledger counts them
into its regime coverage table.

### Host-pressure regimes

Scheduling and release claims also declare one host-pressure regime before measurement:

| Regime | Contract | What it can support |
| --- | --- | --- |
| `quiet` | At most 25% instantaneous CPU busy before and after every sample | Quiet-host confirmation |
| `controlled-interactive` | A declared number of synthetic CPU workers, with at most 25 percentage points of unexplained busy time | Reproducible interactive-load evidence |
| `uncontrolled` | Pressure is recorded but does not invalidate timing | Exploration and discovery only |

On macOS the decision gate uses a one-second delta of Mach host CPU counters.
Load average remains descriptive because it remembers the benchmark’s own worker threads
after a sample exits.
AC power, normal thermal pressure, the synthetic load process, and the boundary
observations must all remain available.
Missing or out-of-envelope data invalidates the sample; a later clean sample does not
replace it inside the fixed-N cell.

### Per-layer counters

Wall time says how long a run took; it does not say what the run *did*. Two results in
this ledger were hard to read for exactly that reason — the allocator turned out to be
about 35% of a cold scan’s engine work and only callgrind could see it, and `exp-051`
predicted a component and was scored on a wall.

The probe and shipped CLI always contain thread-local tallies at the filesystem, index,
and allocation layers.
`FDU_COUNTERS=1` records and prints them to stderr after the run; the probe’s JSON on
stdout is unchanged, because counters describe an implementation rather than the
measurement contract.

```shell
cargo build --release -p fdu --example perf_probe
FDU_COUNTERS=1 ./target/release/examples/perf_probe scan-index --root TREE 2>&1 >/dev/null
```

A 450k-entry cold scan reports, per entry: 15.4 allocations, 11.0 reallocations, 11.9
roll-up merges, and a 93.6% parent-memo hit rate.
That last number is `exp-051`’s mechanism, which previously took a callgrind run to see.

The counters are cheap enough to leave on — `exp-052` measured the overhead at +0.03%
[−3.31%, +3.76%] cold and −1.06% [−1.96%, +0.31%] warm, both spanning zero, which bounds
it below about 3.3% rather than establishing zero.
Three choices buy that: counters are thread-local and non-atomic, per-entry paths are
counted rather than timed, and allocation counting rides on an operation that already
costs tens of nanoseconds.

They localize cost to a layer without attributing it to a call site — no stack sampling,
no live-byte tracking, both of which cost enough to change what they measure.
When a counter raises a question it cannot answer, the answer is a callgrind caller
tree, read as a tree and not as a flat profile.
`fdu-zgxd` is currently that question.

## The reference tree

Timings against a generated corpus answer a different question than timings against a
real one.
Generated corpora are uniform; real trees have a `node_modules` with 6,800 tiny
JavaScript files at depth 12 next to a `.git` with a handful of large packs, and the
distribution is what stresses a walker.

The size of that difference is now measured rather than asserted, and it is larger than
the accept gate. Against a syscall floor, on subjects matched for entry count and
directory width, replacing generated filenames with a real tree’s moves fdu from 1.35×
to 1.42× and `/usr` itself sits at 1.59×, while the same substitution moves a peer
walker barely at all.
Roughly 15 percentage points of fdu’s distance from the floor is invisible on a uniform
corpus — enough to reverse a peer ranking, which
[the floor report](../reports/report-2026-08-23-metadata-walk-floor.md) records
happening to its own first draft.
Treat a generated-corpus figure as a lower bound on real-tree cost, and never let one
establish an ordering between tools.

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

The repository’s `explorations/benchmarks/` subtree is the standard self-contained large
local testbed.
Its 2026-08-13 comparison fingerprint contained 901,963 entries across the
ignored generated corpus, benchmark environment, harness, schemas, and result fixtures.
It excludes volatile Git and Rust build state while retaining realistic language,
dependency, and generated-file distributions.
Generated state moves the exact count, so the protocol fingerprints every run rather
than treating one million as an eligibility cliff.
It is not a committed reproducible corpus and must never be compared by path or assumed
count across machines.
The inactive `cache-pressure-12x` tree under the ignored benchmark corpus is the stable
720,805-entry replication subject on this checkout.
Use it when background builds make the live workspace mutate, and fingerprint it afresh
because it is generated state, not a committed fixture.

Clean up generated replication subjects after an experiment unless the next planned run
needs them. `explorations/benchmarks/corpus/realtree-scratch/` is ignored, can grow to
tens of gigabytes and hundreds of thousands of entries, and is reproducible from the
retained base tree. First confirm that no benchmark process is using it, then move that
scratch directory to Trash on macOS or use the platform’s equivalent recoverable
cleanup. Keep `explorations/benchmarks/corpus/realtree/` unless the base tree itself is
intentionally being rebuilt.
Moving data to Trash does not guarantee physical space is reclaimed until the user
empties Trash.

### Temporary volumes on macOS

A RAM disk is not the default place for builds, temporary files, or claim-grade
benchmarks. It can isolate a named synthetic experiment from device latency, but that is
also why its result does not describe a normal APFS or HFS+ volume backed by storage.
Use the ordinary filesystem for real-tree evidence and ordinary task-scoped scratch
space for Cargo targets, virtual environments, package caches, and test output.

If a synthetic experiment specifically requires a RAM-backed filesystem, use one image
at a time and no larger than the measured need.
Record its purpose, capacity, mount point, and cleanup owner in the active bead.
Keep the primary checkout and every unique result outside the image.
An experimental linked worktree may live on the image only while the experiment runs; it
must be clean or have its changes committed or stashed before the experiment ends.

Audit before creating an image so an interrupted agent does not leave a second one:

```shell
hdiutil info
git worktree list --porcelain
df -h /System/Volumes/Data
```

For every existing `ram://` image, map the image to its exact device and mount point
from `hdiutil info`, then inspect the mount before deciding it is disposable:

```shell
diskutil info /Volumes/FDUPerfExperiment
lsof +D /Volumes/FDUPerfExperiment
git -C /Volumes/FDUPerfExperiment/fdu-worktree status --short --branch
df -h /Volumes/FDUPerfExperiment
```

Do not interpret moving a file to Trash on the image as reclamation.
macOS moves it to that volume’s `.Trashes` directory, so it still occupies the
RAM-backed filesystem.
APFS clones can also make Trash’s logical total exceed the physical blocks in `df`.
Detaching destroys everything left on the RAM disk and is the reclamation step only
after the preservation checks below.

Before handoff, stop every writer, preserve dirty source and unique results outside the
image, and verify `lsof` reports no open paths.
Resolve the exact mount point again and detach without `-force`:

```shell
hdiutil info
lsof +D /Volumes/FDUPerfExperiment
hdiutil detach /Volumes/FDUPerfExperiment
git worktree prune --dry-run --verbose
```

Run `git worktree prune --verbose` only when the dry run names the now-absent temporary
worktree and no other entry.
Then verify that `hdiutil info` no longer lists the image, the mount is gone, and system
pressure has fallen:

```shell
hdiutil info
git worktree list --porcelain
df -h /System/Volumes/Data
memory_pressure
```

If detach reports that the resource is busy, do not force it.
Find the remaining process with `lsof`, stop that work normally, and repeat the
preservation checks.
A new agent must perform this audit before starting another performance run; an
unexplained RAM disk is cleanup work, not a shared cache.

For a publishable `explorations/benchmarks/` run, first finish every benchmark-harness,
environment, corpus, schema, and result-fixture change.
Copy immutable release binaries outside the root; write scratch snapshots, immediate
baselines, JSON, Markdown, and command output outside the root too.
The Make defaults use `/tmp/fdu-realtree`, and the harness rejects explicit state paths
inside the measured root.
Then run no benchmark test, environment update, corpus mutation, or other writer until
the post-run fingerprint completes.
The v2 fingerprint records redacted counts, depth, byte totals, and in-tree hard-link
duplication.
A precomputed baseline is optional: the tool-comparison harness always takes
its own immediate pre-run fingerprint, and any pre/post drift makes the run
non-publishable.

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
[`explorations/benchmarks/realtree/ledger.py`](../../../explorations/benchmarks/realtree/ledger.py).
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

### The denominator, and when the loop is done

The accept rule compares a candidate against yesterday’s binary, which can say a change
paid and can never say how much is left.
[The floor report](../reports/report-2026-08-23-metadata-walk-floor.md) supplies the
missing denominator — the parallel syscall floor per tier per subject — and three rules
follow from having one:

- **A hypothesis states its budget.** Alongside tier and platform, it names the tier’s
  current ×floor and the share of the remaining gap its mechanism can reach.
  On a tier at or under 1.25× the floor, that statement is the justification for running
  the experiment at all, and “the profile shows a hot symbol” does not meet it — the
  floor gap is the budget every hypothesis on that tier shares.
- **Re-screening is floor-aware.** After an accepted change, re-derive the tier’s ×floor
  (`make perf-floor`, `fdu-33ri`) before trusting any queued number: two hypotheses
  aimed at the same µs/entry divide one budget, which this record has confirmed three
  times (H13/H18, H74/`fdu-91ts`, H89/H86).
- **A tier can close.** A tier in a regime is closed when its ×floor on the nominated
  real subjects reaches its threshold — 1.25× aggregate, 1.4× index, stat-floor-bound
  warm open, sidecar-load-bound warm content — or when two consecutive re-screens
  produce no hypothesis naming at least 3% of the remaining gap.
  Closure is a recorded result in this registry, not a fade-out, and it is what lets the
  campaign end somewhere other than exhaustion.

Two tracks run under one accept rule.
A **tuning** experiment changes one variable and faces the gate as always.
A **structural** experiment changes a representation, runs once as a composite —
exp-022’s 542-line precedent, restated for H86 by
[the headroom review](../research/research-2026-08-15-consumer-structural-headroom.md) —
and is gated on the differential oracle plus pre-registered targets that must include
tail spread, because a 3.3× wall spread is user-visible latency the paired median cannot
see.
Forcing a representation change through per-piece 3% gates measures conversion costs
the end state deletes; the composite is the honest unit of decision.

Accept evidence also names its subject class: at least one nominated real tree in the
paired set, with generated corpora as screening — the reference-tree section above
carries the measured reason.
Current prioritization across tiers lives in
[the campaign-2 plan](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md),
not here; this document owns the protocol and the registry.

### Qualifying a scheduling policy takes one extra step

A change to *how much work runs concurrently* — worker counts, scaling triggers, opener
pools — has a failure mode the accept rule above cannot see.
Its behavior can depend on the order work completes in, and completion order is not a
property of the tree.
When it does, two runs over the same corpus are not repeated samples of one thing; they
are samples of whichever policy path each run happened to take.
A paired interval computed across them describes the mixture, and it will look like
ordinary noise.

So a scheduling policy is qualified in three stages, in this order:

1. **Characterize the policy deterministically.** Replay explicit completion orders
   through the real decision code and assert what it concludes.
   This needs no host and no quiet machine, it runs on every platform in CI, and it is
   what distinguishes a policy defect from ambient variance.
   `scan::tests::completion_order` is the worked example; it holds a tree constant,
   varies only completion order, and shows the shipped calibration reaching opposite
   decisions.
2. **Screen on discovery samples.** Verify every real phase from the bounded runtime
   trace rather than from generation order.
   Discovery may eliminate candidates and select one arm, but it cannot confirm that
   arm.
3. **Confirm a survivor on held-out samples.** Use a disjoint paired/interleaved matrix
   in the required quiet and controlled-interactive regimes.
   For paired percent change where positive is slower, the 95% interval’s upper bound
   must be at most +3% for noninferiority; a lower bound above +3% establishes
   inferiority. CPU, system CPU, RSS, faults, context switches, exactness, and policy
   stability have independent pre-registered gates.

Running these in the other order wastes the measurement.
A sweep over a trigger constant whose decision window still closes on an
unrepresentative prefix is partly measuring the prefix, and a heterogeneous corpus will
not reproduce its own result.

The first stage is not a substitute for the second.
Determinism establishes that a policy asks a well-posed question; it says nothing about
what the answer is worth, and a candidate that is only order-robust is screening output
rather than a confirmed winner.
[The adaptive-worker gap-closure report](../reports/report-2026-08-15-adaptive-worker-gap-closure.md)
is the worked example: repeated and staged controllers corrected the late-window
sensitivity but regressed wall time by about 59–61%, so no arm survived discovery and
there was nothing legitimate to confirm.

Policy claims consume `fdu-scan-diagnostics-v1`. The trace is bounded by policy events,
not entries, and records decision ordinals, service windows, ready and in-flight work,
worker counts, handoff backlog, and backend usage.
An outcome difference is recorded as completion-order sensitivity; it is structural harm
only when the history itself is impossible or violates a pre-registered invariant.
Missing or truncated fields, an unverified generated phase, stale binary provenance,
corpus mutation, baseline drift, or fewer than the fixed number of valid pairs makes the
dependent decision inconclusive.

Release comparisons add two more proofs.
A claim-grade provenance manifest binds a clean source revision, lockfiles, build argv,
artifact hashes, host, filesystem, and collectors.
An installation attestation then proves the native Cargo or wheel-installed command, its
effective bash/zsh resolution, its native payload, and a real cache-off scan.
The competitor adapter must establish exact work before any timing is accepted.

## The record is a soft-schema artifact

Every turn of the loop leaves one Markdown file in
[docs/project/experiments/](../experiments/), and the split inside that file is what
makes the loop cumulative rather than a pile of session notes.
The YAML frontmatter carries what a tool reads: the tree fingerprint and host, the
paired medians and bootstrap intervals per job and metric, the complexity cost, and a
`decision` drawn from a fixed set.
It is validated against
[`experiment.schema.yaml`](../experiments/experiment.schema.yaml) at `status: enforced`.
The Markdown body carries what no schema can hold: what the profile suggested, what was
actually tried, and why the number meant what we said it meant.
Neither half has to pretend to be the other, so a refuted hypothesis costs one file and
stays findable months later.
That is what lets the ledger lead with its failures.

- **The contract is compiled, not hand-written.** `make perf-schema` compiles the schema
  from the Pydantic model in
  [`explorations/benchmarks/realtree/experiment.py`](../../../explorations/benchmarks/realtree/experiment.py),
  and `make perf-schema-check` fails on drift.
  The model is the source of truth, and its field descriptions are the documentation
  every artifact ships with.
- **The measured half is never retyped.** `make perf-record` lifts the medians and
  intervals straight out of the run JSON and asks the operator only for what a
  measurement cannot supply: the hypothesis, the change being tested, the complexity,
  the verdict, and one sentence of reasoning.
- **The ledger is a view, not a document.** `make perf-ledger` regenerates it from the
  validated artifacts, so the report cannot drift from the record; an artifact that
  stopped matching the contract fails the build instead of quietly contributing a wrong
  row. Even the regime-coverage table is counted from artifacts rather than maintained by
  hand.

Structure earns its place here only because something reads it: the accept rule and the
ledger tables. Any loop that proposes something, measures it, and decides can keep its
record the same way; [softschema](https://github.com/jlevy/softschema) documents the
format, the CLI, and this pattern as a playbook.
Extracting this whole loop — the contract, the statistics, the generated views, and the
protocol — into a framework other campaigns can adopt is planned in
[the framework extraction spec](../specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md),
which also records what metabrowser’s independent re-implementation of the method
confirmed about which parts are invariant.

## Publishing the evidence

Two documents are generated from the artifacts, and neither is ever edited by hand.
`make perf-ledger` builds the per-experiment ledger; `make perf-report` builds
[the charted evidence report](../reports/report-2026-08-20-fdu-performance-evidence.md),
which is what a reader outside the project is given.

Adding an experiment is therefore three commands and one judgement.

1. `make perf-record ARGS="..."` writes the artifact from the run JSON.
2. `make perf-ledger` regenerates the ledger.
3. `make perf-report` regenerates the projection and the page.
4. Commit all of it together.
   `make check` runs `perf-report-check`, which re-derives both generated files and
   fails if the committed copies no longer match the artifacts, so a new experiment that
   is not published is caught before it is merged rather than after.

The judgement is the preparation date.
It lives in the projection rather than being read from the clock, so regenerating
without changing an artifact changes nothing and the drift check does not fail at
midnight for reasons unrelated to the evidence.
Stamp a new one when the page is republished for people to read:

```
make perf-report PREPARED=2026-09-01
```

Three properties of the page are worth preserving deliberately, because each was got
wrong once and each fails silently.

- **The paired change and the two medians are different numbers.** `change_pct` is the
  median of the paired differences; `control_median` and `candidate_median` are marginal
  medians of each arm.
  Under host drift they disagree, and in this record they differ by more than two
  percentage points on 21% of measurements.
  Anything new that reports a change must take it from the paired figure, and must not
  invite a reader to divide the absolute values into it.
- **Absolute values only mean something against their own subject.** A new figure that
  puts two subjects on one axis has to normalise, and has to keep synthetic subjects
  visibly apart. `SYNTHETIC_SUBJECTS` in
  [`timeline.py`](../../../explorations/benchmarks/realtree/timeline.py) is a
  hand-maintained set; a new adversarial tree must be added to it or it will be averaged
  in with ordinary work.
- **A rejected candidate is not the product’s state.** Anything plotting “where we are
  now” must read the kept arm, which is the candidate only for an accepted experiment.

A new figure belongs in `report_html.py` beside the others, drawn as inline SVG from the
projection.
It must not fetch anything: the page is a single file that has to render from
a `file://` URL, offline, on a machine that has never opened it.
Its stylesheet and script must be ASCII, because the output is emitted with character
references and neither `<style>` nor `<script>` decodes them; the tests in
[`test_timeline.py`](../../../explorations/benchmarks/realtree/tests/test_timeline.py)
enforce this along with the standalone document shape.

## Hypotheses

Kept as a live list; the *ordering* — which of these to run next and why — is owned by
[the campaign-2 plan](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md).
Numbering is shared with the
[performance-frontier research](../research/research-2026-08-10-performance-frontier.md),
whose backlog owns H12–H46; new hypotheses from any source take the next free number
(currently H94) so no id ever means two things.
Each is stated so it can be wrong, with the metric that would show it.
Status is updated as experiments resolve them; see the ledger for results.

### Traversal and syscalls

| # | Hypothesis | Predicted effect | Status |
| --- | --- | --- | --- |
| H1 | The walk is serial, so it uses one core while the machine has ten. A bounded parallel producer feeding a single index consumer will cut wall time several-fold. | `wall_ns` down 3–4×, `cpu_ns` roughly flat or slightly up, `cpu_ns/wall_ns` from 1.0 to 4+ | **Confirmed** (exp-001, −50.0%) |
| H2 | `fs::read_dir` opens each directory by absolute path, so the kernel re-resolves every component from the root. Opening relative to a retained dirfd (`openat`) removes repeated prefix resolution. After H26 landed, `open` is the largest remaining cold cost at **33.86%** of self time. | `system_cpu_ns` down, biggest effect on deep trees | **Refuted in both measured forms.** One retained root fd was neutral at 720k (exp-024); a bounded parent-relative frontier changed 1M indexed wall by −0.69% [−1.49%, +0.49%] and regressed RSS/faults (exp-038). |
| H3 | One `fstatat` per entry (20.08% of post-adaptive self time) is the floor for a portable walker, but macOS `getattrlistbulk` batches enumeration and complete stat-tier metadata per directory. | `system_cpu_ns` down substantially | **Confirmed on macOS** (exp-022): 720k cold-index wall −30.13%, producer wall −41.60%, system CPU −46.62%/−61.40%; 60k wall −5.22%/−9.25%. Linux `statx` remains open. |
| H4 | Depth-first traversal order has worse locality than breadth-first for a tree whose directories were written breadth-first. | indexed wall, component, `system_cpu_ns`, `minor_faults` | **Confirmed for breadth-first** (exp-037): depth-first regressed 1M indexed wall 3.57% [2.42%, 5.23%] and component 6.72%, while saving only 1.03% RSS. Breadth-first also preserves progressive shallow coverage. |
| H31 | A latency-bound walk needs more in-flight metadata operations than the six-worker warm-small knee. Calibrating aggregate chunk service time over the first 16k entries should select sixteen workers in the slow state and retain six in the fast state. | slow cold wall down 5–10%; fast wall and resources unchanged | **Confirmed for the portable high-latency path** (exp-015–021): service-time calibration improved 720k cold-index wall 5.31% and producer wall 10.09%. H52 confirms that the same trigger correctly remains at six after bulk metadata removes the per-entry wait. |
| H52 | H26 removed the per-entry metadata wait that made sixteen workers the pre-bulk 720k knee. On the bulk backend, six workers should now match or beat deeper fixed pools while using less CPU and memory, and the H31 service-time trigger should remain inactive. | 6-worker large-tree wall no worse than 8/12/16; CPU, context switches, and RSS lower | **Confirmed** (exp-025): sixteen workers regressed indexed wall 19.19%, producer wall 12.65%, total CPU 107–117%, and RSS about 33%. Eight was neutral in the exploratory curve, and automatic calibration remained at six. |
| H54 | The macOS bulk reader creates and drops one `Vec<Entry>` per directory: 7,350 allocations at 60k and 88,201 at 720k. Retaining the staging vector in each reader and draining it after a successful complete-directory parse should reuse capacity without weakening atomic fallback. | cold/warm `user_cpu_ns` and faults down; at least one primary wall/component down 3%; RSS neutral | **Refuted at 60k** (exp-028): cold-index wall +0.21%, producer +1.32%, warm −0.85%; predicted CPU/fault reductions were absent and producer RSS/faults regressed. Reverted without a 720k run. |
| H55 | A 256 KiB `getattrlistbulk` buffer should reduce repeat calls in wide directories versus H26’s 64 KiB buffer. | cold wall/component and system CPU down at least 3%; warm may compose; account for about 1.1 MiB more capacity across six workers | **Refuted at 60k and 1M** (exp-029/039): the 1M revisit changed indexed wall by +2.22% [−1.62%, +8.19%], with component, RSS, and faults all moving the wrong way. |
| H57 | Breadth-first scheduling and bulk enumeration may have moved the optimal worker depth above the automatic policy on the heterogeneous 1M tree. | indexed wall down at least 3%; CPU/RSS proportional | **Refuted** (exp-036): eight workers gained only 1.30% while CPU rose 33.5%; twelve and sixteen workers regressed wall 2.46% and 10.65%. Automatic/six remains the operating point. |
| H58 | On the portable backend, splitting metadata work from very wide directory reads into small stealable chunks may expose parallelism that directory-level scheduling cannot, as `dua` 2.41.1 does with four-entry jobs. | portable/Linux wall and system CPU down at least 3%; queue wait and RSS bounded | **Queued** (`fdu-r9he`). Profile first; preserve region order/progress and do not add a queue dependency merely to screen it. |
| H71 | Rearranging the Linux enumeration and metadata layer — raw `getdents64`, narrow `statx` masks, or io_uring-batched `statx` — reduces warm wall time, as the macOS bulk reader did. | warm wall down at least 3% against a `getdents64`+`statx` control | **Refuted on the scouting rig** ([Linux first measurements](../research/research-2026-08-13-linux-first-measurements.md)): all masked and raw variants sat within ±1.6% of the control, and io_uring at queue depth 128 cost +327% warm and +78% controlled-cold. Rust’s standard library already issues `statx`+`getdents64`, so there is no enumeration gap to close. Re-test only on real hardware (`fdu-dzs0`). |
| H72 | The transient summary needs no directory or symlink attributes, so `d_type` can skip their `statx` calls entirely. | produced stat calls down by the directory share; warm wall down at least 3% on a directory-heavy tree | **Queued** (`fdu-i2f3`). Measured −1.4% alone on a 6.4%-directory tree, below the gate; the planner must prove the tier and `one_filesystem` still forces directory stats. |
| H73 | Sorting each directory’s entries by `d_ino` before statting turns random inode reads into near-sequential ones on ext4 and btrfs. | controlled-cold wall down substantially; warm unchanged | **Unresolved** (`fdu-lf3v`). The scouting rig measured −2.3% cold with an interval crossing zero and +6.8% warm from the sort itself, but its guest-cold reads were host-cached, so the cold claim was untestable there. Needs bare metal, and any implementation must stay behind a cold-suspected trigger. |
| H76 | Linux cold scans are under-parallelized: `diskus` runs three times the core count and the automatic policy’s thresholds were calibrated on APFS. | controlled-cold wall down at least 3% at the swept depth; warm unchanged | **Queued** (`fdu-tk1b`). The scouting rig measured `diskus` 22.8% ahead of the summary plan cold while tying warm. Sweep depth per regime and filesystem rather than inheriting the APFS constants. |
| H77 | Both fdu and dumac pay at least one directory open plus one bulk call across 110,369 directories, and exp-045/046 profiles put about 95% of worker samples there. macOS `searchfs` reads the volume catalog without opening each directory, removing that per-directory work rather than shaving it. | macOS cold and warm wall down substantially at near-million scale, with exact oracle parity | **Speculative, unscreened** (`fdu-9716`). Needs parent-id tree reconstruction, subtree scoping, a permission-semantics audit, non-UTF-8 handling, and probe-and-fallback. Prototype standalone before any production path. |

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
| H19–H22 | Pack reusable full-index entries: inline arena storage, one stored name, directory-only roll-ups, and compact identities/revisions. The post-CLI 1M result spends about 631 MiB peak RSS, 44% above `origin/main`. | substantial peak RSS and fault reduction; wall neutral or at least one arm ≥3% faster | **Queued** (`fdu-prph`). Measure layout and one structural arm at a time; snapshots, stable identities, deltas, and query results remain exact. |
| H59 | A cache-off report might retain only state required by the complete requested view set, as `pdu` discards subtrees below its output depth. | large RSS reduction and wall down at least 3%, with byte-identical output | **Confirmed, topology-sensitive** (exp-040): heterogeneous wall −14.56% [−18.55%, −9.04%], RSS −95.28%, and one stable semantic hash. Exact-final-binary uniform-tree replications retained the CPU/RSS mechanism but measured only 1.8–2.8% wall. |
| H62 | Rich summaries can reduce inside scan workers rather than materializing file paths and sending observation batches through one consumer. | wall/user CPU/channel work down at least 3%; exact summary hash | **Refuted as production code** (exp-041): user CPU fell 36.23% and RSS 34.77%, but wall improved only 1.38%; the independent 720k replication was 1.26%. Reverted after the H63 composition also failed. |
| H63 | A macOS bulk request derived from rich-summary requirements can omit index-only fields and avoid copying names for files. | system/user CPU and wall down at least 3%; strict parser and portable fallback parity | **Refuted in composition with H62** (exp-042): wall changed +1.86% [−1.96%, +4.56%] despite 50.96% lower user CPU and 39.70% lower RSS. Both layers were reverted. |
| H64 | A selected-total projection can gather only the requested size metric for a workload matched to dumac. | beat or match dumac wall with exact fdu path-accounting oracle | **Refuted** (exp-044): the full narrow-reader and in-buffer-folding composition changed wall −1.15% [−2.24%, +0.44%] and did not beat dumac despite halving user CPU; all prototype API and engine code was reverted. |
| H65 | Removing the index consumer may move the reduction-only worker-depth knee above six. | 6/8/10/12/16 curve; wall down at least 3% with bounded CPU/RSS | **Refuted** (exp-043): eight workers’ promising 901k screen did not replicate at 720k (+0.67% wall [−1.56%, +3.99%]); CPU rose 40.66%. Automatic/six remains shared by transient and indexed scans. |
| H66 | An unfiltered cache-off tree-only request can fold file observations into exact directory roll-ups and retain directory topology without file records. | byte-identical tree; wall down at least 3% or decisive RSS reduction without meaningful latency regression at 60k and near-million scale | **Queued** (`fdu-sk7v`). The planner must fall closed for cache, filters, multiple views, watch, or reusable-index requests; compare the Linux arm with dut’s rendered-tree job. |
| H60 | Cold bootstrap workers can build disjoint local subtree arenas and splice them plus one roll-up at region completion, replacing one path operation per entry through the single consumer. | cold-index component/user CPU and channel allocation down; end-to-end wall down at least 3%; RSS bounded | **Queued** (`fdu-weey`). Preserve deterministic identity, progressive publication, errors, and the delta contract. |
| H61 | A completed bootstrap can live in a dense immutable base while subsequent changes use a sparse overlay and bounded compaction, avoiding the full mutable-entry overhead on nearly every record. | million-scale RSS down at least 40% plus cold indexed wall down at least 3% or a decisive warm/query win | **Queued after H19–H22** (`fdu-f67r`). Preserve stable identities, exact snapshots, all views, progressive publication, errors, deltas, and watch semantics. |
| H74 | Producers allocate paths and observation batches that the consumer frees, which is the cross-thread pattern glibc `malloc` handles worst. A different global allocator should recover that cost where the scan is not syscall-bound. | transient-summary wall down at least 3%; RSS increase bounded at million scale | **Confirmed for one tier only, and not adopted.** A local mimalloc build on 450k entries measured the aggregate tier at −23.03% [−28.36%, −16.72%], the index tier at +3.66% [−17.62%, +17.53%] (spans zero), and snapshot load at −0.81% [−26.42%, +38.01%] (spans zero). The load result is the interaction to remember: `fdu-91ts` removed roughly four of five per-record allocations structurally first, so the allocator had nothing left to win there — the same competition H13 lost to H18. Peak RSS on the aggregate tier rose 18.1→43.3 MB, +139% on the tier whose whole pitch is low memory. Not adopted: it is a C-building dependency for a one-tier win, unmeasured on macOS, and H85 may capture the same cost without it. |
| H85 | The aggregate tier’s cost is not allocation *volume* but glibc’s cross-thread free path: workers allocate, one consumer frees. Returning drained batch buffers to their producing worker would make each arena allocated and freed on one thread, capturing H74’s win without a dependency or its memory cost. | transient-summary wall down at least 20%, i.e. comparable to mimalloc; peak RSS no worse than today | **Queued** (`fdu-h7sw`). The evidence for the diagnosis is that H51 and H62 both *reduced allocation counts* on this tier and were both refuted on wall, while mimalloc changes no counts and wins 23%. Anything below about 20% is not capturing the same cost. Note H86’s worker-local arenas dissolve the same pathology structurally; screen H85 only if H86 stalls. |
| H86 | The remaining consumer cost is the representation, not any one part of it: boxed `BTreeMap` entries, twice-stored names, per-op `PathBuf`s, per-entry ancestor merges, and the channel are one decision wearing seven hypothesis numbers (S1–S4, H19–H22, H60, `fdu-2ubt`). An oracle-checked spike (`explorations/benchmarks/spikes/arena_spike.rs`) holding worker-local arena entries, one name arena, per-directory tallies, and a single bottom-up roll-up runs the 450k Linux tree in ~199 ms / ≤23 MiB where the tree view spends ~849 ms / ~279 MiB, so the composite — measured as **one structural experiment**, exp-022 precedent — has a measured ceiling rather than an argued one. Intermediate forms pay conversion costs the end state deletes, which is why the pieces must not be gated separately. | cold-scan-index wall down at least 50% and peak RSS down at least 60% on the 450k Linux subject; engine digests byte-identical at every worker count via the differential harness | **Queued** (`fdu-xde5`, epic over `fdu-2ubt`, `fdu-prph`, `fdu-weey`, `fdu-fnfc`, `fdu-uv0s`), now the centerpiece of [campaign 2](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md). The floor report hardened the ceiling from a spike number into a denominator: `arena_spike` runs at **1.06× the parallel syscall floor** where the index tier runs 2.68×, and the ~15-point real-tree tax lands in exactly the name/path handling this deletes, so real-tree evidence understates the win. Floor-anchored targets supersede the piecemeal predictions: index ≤1.4× floor, aggregate ≤1.25× on real subjects, p95/median spread ≤1.5×, RSS ≤3× `arena_spike`; absorbs `fdu-0pzh` and the H89 headroom, and decides `fdu-refc` as layout. See [the structural-headroom review](../research/research-2026-08-15-consumer-structural-headroom.md). |
| H87 | `spawn_save` deep-clones the entire index synchronously before its thread spawns, so the clone sits on the render path of every cache-writing run — the first-run default shape, not only the changed-warm path. Measured: `--cache off` 860 ms vs `--cache refresh` 1,341 ms on the 450k Linux subject. Sharing the index with the writer (Arc, or serialize-before-return) removes it. | cache-writing end-to-end wall down at least 10% on the 450k subject; `--cache off` unchanged | **Confirmed on Linux, at its predicted size** (exp-063): the new `cold-open-save` job measured wall −10.50% [−21.96%, −8.11%], component −17.26%, and peak RSS −35.26% [−38.16%, −33.08%] — the second copy of the index disappearing. `cold-scan-index`, which writes no cache, was the placebo and did not move. **Validated on macOS/APFS** (`fdu-m4r6`): a cross-branch paired A/B over a 494,031-entry tree measured the cache-writing CLI path −174 ms (−4.3%), 95% CI [−418, −60], user-CPU −254 ms, on an uncontrolled host — conservative, since host noise widens rather than narrows that interval. |
| H88 | Snapshot CRC-32C is computed one byte at a time through a lookup table over a ~28.7 MB image on both save and load; slicing-by-8 needs no unsafe and hardware CRC32C (SSE 4.2 / ARMv8) is a follow-up behind runtime detection. | `warm-snapshot-load` and `cold-snapshot-save` component down at least 3% | **Confirmed on the pre-registered signal, on Linux** (exp-061): save component −12.20% [−36.02%, −0.10%], load component −2.02% [−7.00%, −0.34%], both walls improved with intervals below zero; accepted for ~55 dependency-free lines under the exp-009 exception. The digest is bit-identical by construction, but the speed claim is Linux evidence: the macOS A/B recorded on `fdu-m4r6` exercised this change inside H87’s confirmed interval, with no separate component measurement yet; the hardware-CRC32C follow-up (ARMv8 first, so a macOS agent’s item) is `fdu-6kyn`. |
| H89 | `derive_ext` allocates a `Vec` per file, validates it with `String::from_utf8`, and `intern_ext` resolves it by string comparison — per file, for an id that directory-listing extension runs make nearly constant. A one-slot `(bytes, ExtId)` memo beside the parent memo removes the allocation, the UTF-8 pass, and most lookups. | cold-scan-index `user_cpu_ns` down and wall down at least 3%, alone or folded into H86 | **Refuted on Linux** (exp-060): wall +1.59% [−1.62%, +6.08%] and user CPU regressed +1.42% [+0.39%, +3.98%] — the memo’s own extraction and compare cost what derive-and-intern saved, the H51/H62 shape on a new site. The reasoning is glibc-specific, so the refutation is Linux-scoped; `fdu-m4r6` notes the optional libmalloc re-screen. The headroom accrues to H86, where arena entries intern from batch context. |
| H90 | `apply_baseline` clones each batch’s effective ops — `PathBuf`s included — into the journal, then `establish_baseline` clears it: one cloned-and-freed path per entry per cold scan. exp-003 refuted the invasive twin-loop form on macOS at 60k before the consumer was the bottleneck; the residue is a one-flag change. | cold-scan-index `minor_faults` and `user_cpu_ns` down; wall likely under the bar alone, expected to ride with H86 | **Confirmed beyond its own prediction, on Linux** (exp-062): cold wall −5.06% [−6.03%, −3.54%] on the confirming re-run (−4.62% first), user CPU −6%; the first run’s RSS and warm alarms did not reproduce, and the reproducing context-switch increase costs no wall or CPU. exp-003’s macOS refutation was of a twin-loop form on a pre-consumer-bottleneck build; a parameter on the shared loop measures differently three campaigns later. **macOS re-screen** (`fdu-m4r6`): a tie on an uncontrolled host — median −11 ms, 95% CI [−159, +133], user-CPU −53 ms — the consumer-side saving sits below the kernel-bound noise floor there; direction right, not a regression, re-screen on a quiet host if a macOS number is ever load-bearing. |
| H91 | The observation channel is unbounded `std::sync::mpsc` while producers outrun the consumer ~4× on Linux, so queued batches are an unmeasured store at exactly the moment RSS peaks; a bounded channel caps it. Measure occupancy first. | peak RSS down on 450k+ trees with wall unchanged; superseded if H86 removes the channel | **Queued** (`fdu-0pzh`). |
| H92 | Snapshot load re-derives what save discarded — roll-ups re-merged per record, extensions re-interned — though both are deterministic functions of the tree the snapshot pins. Persisting them converts load from re-derivation to adoption. | `warm-snapshot-load` component down several-fold on the 450k subject | **Queued** (`fdu-yr23`); composes with H78 (`fdu-pdra`) and H35 per-block checksums. |
| H93 | The release profile already takes fat LTO and one codegen unit; profile-guided optimization on the branchy consumer code typically returns another 5–15% with no source change. | cold-scan-index and warm-revalidate wall down at least 3% under the loop; adopt into release builds only if the interval clears | **Queued** (`fdu-pdne`). |
| H78 | H10’s remaining half: once load stops rebuilding the tree per record, the residue is parse-and-allocate. A format whose on-disk layout is usable directly, with roll-ups persisted rather than recomputed, makes warm open bound by the reconcile walk instead of the load. | `warm-snapshot-load` component down several-fold; warm open below cold-scan wall on Linux | **Queued after `fdu-91ts`** (`fdu-pdra`). Preserve exact snapshot semantics, a completeness boundary in the format version, endianness and alignment discipline, and allocation that is never sized from untrusted counts. |

### Content analysis

| # | Hypothesis | Predicted effect | Status |
| --- | --- | --- | --- |
| H79 | The basic analyzer always starts a scoped worker pool, even for a few hundred files. On the immutable 233-file self-host corpus, thread bootstrap and synchronization account for more profile samples than allocation. Running auto-selected workloads of at most 512 files and 8 MiB inline should remove that fixed cost without changing large-tree parallelism or an explicit worker request. | Self-host `content-basic` wall and component down at least 3%; CPU, RSS, and semantic digest no worse; large-tree automatic and explicit parallel paths unchanged | **Refuted** (exp-047): inline analysis regressed self-host wall 66.34% and the content component 92.93%; serial reads saved aggregate CPU but discarded useful I/O parallelism. Reverted. |
| H80 | `BasicAccumulator::push_text` is the largest named analyzer cost in the frozen SLOC profile because it computes prose-only word, paragraph, and logical-word statistics for code files and discards them afterward. Selecting its prose collectors from the already-known content family should remove that work without changing admission or line semantics. | Immutable self-host `code-sloc` wall and component down at least 3%; CPU and semantic digest no worse; `content-basic`, cache-hit, golden, and unit semantics unchanged | **Refuted** (exp-048): the 12-pair `code-sloc` wall interval crossed zero and the median failed the 3% bar; cache-hit and basic jobs were neutral. Reverted. |
| H81 | `markdown-prose-v1` knows the admitted file size before reading but grows its retained parser buffer from zero. Reserving the bounded file size once should remove repeated growth and copying from the Markdown path. | Generated `markdown-prose` wall and component down at least 3% with both intervals below zero; peak RSS, document-cache-hit, semantic digest, goldens, and self-host behavior no worse | **Refuted** (exp-049): Markdown wall moved −3.55%, but its paired interval [−14.49%, +7.45%] crossed zero; cache-hit behavior was neutral. Most files already fit one 64 KiB read, so the hint removed too little work. Reverted. |
| H82 | `BasicAccumulator::push()` allocates and copies every read chunk into a temporary vector even when no split UTF-8 sequence is pending. Decoding ordinary complete chunks in place should remove one allocation and one full byte copy per read while retaining the carry path for boundary splits. | Generated `text-prose` and `markdown-prose` wall and component down at least 3% with intervals below zero on the primary Markdown job; peak RSS, cache-hit, semantic digests, chunk-boundary tests, goldens, and self-host behavior no worse | **Confirmed** (exp-050, `2fef9bf`): the 32-pair Markdown run improved wall 12.04% [−16.46%, −8.38%], component 13.67%, user CPU 12.24%, and peak RSS 9.12%. Plain text, self-host wall, and cache hits were neutral; all semantic oracles passed. |

### Warm start

| # | Hypothesis | Predicted effect | Status |
| --- | --- | --- | --- |
| H9 | Warm revalidation is currently *slower* than a cold scan. Reconciliation does a full walk plus expectation lookups plus a snapshot load, so the cache costs more than it saves. | `warm-revalidate` wall below `cold-scan-index` wall | **Closed on Linux; re-measure on macOS.** H12/exp-030 brought the verified 60k warm open from about 508 ms to 351 ms against about 296 ms cold, leaving the load to blame. H75’s two fixes removed it: warm open is now below cold-scan wall on Linux with lower RSS. The macOS numbers predate both fixes and should be re-taken before the claim is made there. |
| H12 | After H14 elided exclusive no-op applies and H26/H53 batched metadata, workers can compare bounded directory waves against one immutable baseline and send only changes through the delta contract. This revisits exp-002 without its single-consumer funnel. | 60k warm wall down at least 15% with its interval below zero; reconciliation component down at least 25%; exact parity and RSS increase no greater than 10% | **Confirmed** (exp-030): four-worker waves improve warm wall 30.25% at 60k and 59.53% at 720k; reconciliation component falls 50.31%/72.55%, 60k RSS rises 3.29%, and large RSS improves 0.99%. Shared/scoped/one-worker paths retain serial arbitration. |
| H56 | exp-030’s post-profile attributes about 13% of 60k warm samples to scoped thread startup/waiting. Quadrupling the directory wave should amortize that residue while keeping both deferred changes and progressive publication bounded. | 60k warm wall or component down at least 3% with its interval below zero; RSS increase no greater than 5%; exact parity | **Refuted at 60k** (exp-031): 4,096-directory waves changed warm wall +1.64% [−3.88%, +10.07%] and component +13.24%; CPU/context-switch signals were unclear. Reverted without a 720k run. |
| H10 | Snapshot load is ~320 ms of the warm start. A format whose on-disk layout can be used without rebuilding the tree would make the warm path open-latency-bound instead of parse-bound. | `warm-snapshot-load` wall down | **Largely addressed without a format change** (exp-005, then `fdu-91ts`): the loader was rediscovering, per record, a parent it already held. Inserting beneath the known `EntryId` measured Linux load −51.9% [−53.2%, −51.0%]. What is left for H78 is the parse-and-allocate residue, now a much smaller share. |
| H11 | `revalidate` builds a `BTreeSet<OsString>` of seen names per directory, cloning every name. Comparing against the index’s existing sorted children directly would remove that. | `user_cpu_ns` down on warm jobs | **Not a target, resolved without measuring.** The clone is real, but `scan::revalidate` has no production call site: `open` uses `reconcile`, and even the probe’s `revalidate` job calls `reconcile`. It is a public observation-only reference API exercised only by this crate’s tests, so the change would speed up nothing a user runs — and it is the one function whose clarity matters more than its speed, because its job is to be the obviously-correct reference the parallel paths are checked against. Re-open only if it acquires a caller. |
| H53 | Full reconciliation still uses portable enumeration plus one `fstatat` per entry even though H26’s audited macOS reader returns the same complete stat-tier contract in bulk. Reusing it per directory should remove the warm profile’s 29.25% `fstatat` and 6.76% `getdirentries64` costs while preserving complete-directory fallback. | `warm-revalidate` wall and component down at least 3%; `system_cpu_ns` down; oracle parity at 60k and, if scale-sensitive, 720k | **Confirmed on macOS** (exp-026): warm wall −18.97% at 60k and −34.39% at 720k; large component −39.05%, CPU −44.06%, system CPU −53.97%, RSS neutral. Direct, shared, and scoped reconciliation reuse the existing reader. |
| H75 | H9’s inversion persists on Linux, where no bulk reader hides it: snapshot load rebuilds every record through the full apply path, reconciliation re-stats every entry, and a quiet warm open still deep-clones the index and rewrites a byte-equivalent snapshot. Removing the load and save bookends should make warm open beat a cold scan. | verified warm open below cold-scan wall on Linux; warm RSS no higher than cold | **Confirmed and closed.** Both bookends are gone. `fdu-maxn` removed the byte-identical rewrite and the clone it required (−20.6% [−21.2%, −16.6%], RSS 411→195 MB); `fdu-91ts` removed the per-record path rediscovery in the loader (load −51.9% [−53.2%, −51.0%], warm open −41.9% [−43.3%, −40.6%]). At 450,463 entries a warm open now runs **762 ms against a 984 ms cold scan — 22.6% faster, with lower RSS** (191 vs 278 MB), where it began this campaign 69% slower. The cold path is untouched: +0.8% [−2.6%, +4.3%]. `fdu-niuz` still owns the clone on the *changed* path, which no longer sits on the quiet warm run. |
| H83 | The content sidecar rebuilds per-record state on load exactly as the metadata snapshot does, so warm content runs are bound by restoring precomputed metrics rather than by analysis. | `--cache only` content load component down several-fold; warm content wall below the profile-independent floor | **Open, measured** (`fdu-78q6`): the sidecar load costs about 370 ms for 14,542 files, roughly 25 µs per file against about 3 µs per metadata record, and all three analysis profiles converge on the same ~520 ms warm floor regardless of how much analysis the sidecar saved. Same shape as H78 and probably the same answer. |
| H84 | `ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY` was placed between APFS regimes of ~18, 22 and 42 µs per entry, but the Linux warm floor is about 1.5 µs, so the adaptive unlock never fires on Linux and an automatic scan stays at its six-worker cap in every regime the threshold was meant to separate. | calibration never crossing the threshold on a Linux warm scan; a `--threads` sweep finding a knee above six | **Queued** (`fdu-mjwr`, with H76/`fdu-tk1b`). A mechanism for the cold scalar-class gap, not yet a measurement — but the sign now has independent support: guest-cold at the raw syscall floor itself, sixteen workers beat four by 32% on four cores, which can only be queue depth. Gated on the aggregate probe (`fdu-tyjx`); bare metal (`fdu-lf3v`) sizes it. See [platform tuning](platform-tuning.md). |
| S1 | `apply_upsert` resolved every entry’s parent by splitting the path into a component vector and descending from the root, one `BTreeMap` lookup per level, to reach a directory the walker was standing in when it produced the record. A walker reports a directory’s children consecutively, so remembering the previous upsert’s parent answers almost every entry with one path comparison. | cold-scan-index wall down at least 15% | **Confirmed, at a different number than predicted** (exp-051, `fdu-ypk2`). Wall fell 7.35% [−10.42%, −6.12%]; the index-build *component* fell 16.6%, which is what the 15% prediction actually described. Stating a component prediction against a wall-clock accept rule is the mistake to avoid repeating. `normalize` instructions fell 89%, so the memo hits; the remaining gap to the loader’s −51.9% is the producer’s per-entry `PathBuf`, which needs a batch-shaped observation (`fdu-2ubt`). |

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
  --trials 12 --baseline-fingerprint explorations/benchmarks/results/realtree/tree-mytree.json \
  --name exp007-parallel-producer
```

`make perf-baseline`, `make perf-profile`, and `make perf-compare` wrap these with the
project’s usual arguments; `PERF_TREE` selects the tree.
Evidence qualification is explicit: `PERF_STAGE`, `PERF_HOST_REGIME`,
`PERF_BACKGROUND_LOAD_WORKERS`, `PERF_PROVENANCE`, and `PERF_CORPUS_MANIFEST` map
directly to the harness contracts.
The defaults are exploratory and uncontrolled, so an omitted variable cannot
accidentally create a held-out claim.

Results land under `/tmp/fdu-realtree/results/` by default because the standard measured
tree contains `explorations/benchmarks/results/`; writing evidence into the subject
would invalidate the run.
Host-specific raw artifacts remain outside the repository.
What gets committed is the ledger entry: the numbers that mattered, the verdict, and the
reasoning.

### Comparing against other tools

`make perf-compare-tools` runs each competitor immediately beside an immutable fdu
anchor, alternates pair order, and reports paired bootstrap intervals.
`PERF_TOOL_LABEL` and `PERF_TOOL_CONTRACT` identify that anchor in its provenance
manifest; held-out installed-command cells also set `PERF_INSTALLATION_ATTESTATION` and,
for a wheel, pass its native extension through `PERF_TOOL_SUPPORTING_ARGS`. The
immediate pre/post v2 fingerprints must agree; results and the baseline live outside the
measured root. Each artifact retains binary hashes, versions, command templates, work
classes, resource use, and redacted output hashes.
When fdu summary contracts are measured, the harness hashes the stable report payload
after removing only generator, absolute root, and timestamps.
It also checks all five summary tallies against the independent v3 tree fingerprint:
files, descendant directories, apparent bytes, allocated bytes, and newest regular-file
mtime. Partial, stale, cached, or error-bearing reports are invalid.
A semantic or oracle mismatch invalidates the sample; a timing for a changed answer is
not performance evidence.

Which anchor contract to use is a real choice, because the contracts measure different
questions. `fdu-transient-summary` and `fdu-index-summary` isolate engine work with
`--cache off`, which is what a change to the walker or the index should be judged on.
`fdu-default-tree` is the bare `fdu PATH` invocation — cache `auto`, tree view, snapshot
written on every run — and is the only contract that measures what a user gets by typing
nothing else.

Prefer the summary contracts for engine work and the default contract for user-visible
claims; a change can move one and not the other, and for three campaigns nothing
measured the second, which is how a default-path regression stayed invisible while every
`--cache off` cell looked healthy.
The default contract writes a snapshot, so the harness gives it an isolated cache
directory for the run: measuring against the operator’s own cache would let an unrelated
earlier run set this one’s starting state, and would leave a snapshot of the subject
tree behind. It is a legal anchor but not a summary contract, so it cannot carry a
held-out release claim.

The default comparison should include rendered-tree peers (`dust`, `gdu`, `pdu`) and
fast total-only lower bounds (`dua`, `diskus`, and macOS `dumac`). `ncdu` is a useful
indexed-tree peer; BSD/GNU `du` are serial floors.
These numbers never enter an fdu experiment verdict because the jobs differ.
In particular, fdu builds a reusable exact metadata index and roll-ups; total-only tools
retain much less state, and tools differ in hard-link attribution.
The fingerprint quantifies that semantic difference rather than pretending byte totals
are interchangeable.

Source review is part of calibration.
Pin each ignored `attic/` checkout to the exact revision whose binary is measured,
record its license, and describe transferable mechanisms rather than copying
implementation. Current findings are reflected in the queue: `dua`’s portable
wide-directory chunks motivate H58; `pdu`’s bounded retention motivates the design-gated
H59 and its local aggregation helps motivate H60. H59 is now confirmed by exp-040.
`dumac` confirms fdu’s existing `getattrlistbulk` mechanism and motivated H62
worker-local reduction, H63 report-derived macOS metadata, H64 a selected-total matched
workload, and H65 reduction-only worker calibration.
Exp-041 through exp-044 rejected all four additional layers for wall time.
Their decisive CPU and memory reductions show that the remaining elapsed-time floor is
directory-open and kernel work, not the rich summary representation.
The 2026-08-13 dut refresh adds H66, an exact directory-only transient tree, and
strengthens the Linux proof gates: distinguish verified warm, dut-style
pagecache-drop-only, and `sync` plus `echo 3` controlled cold; reject partial scans even
when dut exits zero; and exercise multi-buffer directories, hard-link-table growth, and
sparse/preallocated size ordering before timing.
`dust`, `gdu`, and `diskus` mainly reinforce work already measured: recursive high
concurrency is not a new hypothesis after H52/H57 rejected over-threading on APFS.

* * *

*Part of the fdu project documentation.
See [AGENTS.md](../../../AGENTS.md).*
