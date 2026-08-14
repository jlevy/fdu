# fdu Performance Campaign: Status, Method, and What Remains

**Date:** 2026-08-14

**Author:** fdu project, with Claude Code assistance

**Status:** Current

## Who this is for

You need no prior context.
This report explains what fdu is, how its performance work is organized, every
improvement made so far and in what order, what is still open, and where the evidence is
weak. It is written to be handed to someone — or some agent — arriving cold.

If you want the condensed method rather than the history, read
[the instrumentation playbook](../guides/performance-instrumentation-playbook.md).
If you want the protocol and the live hypothesis list, read
[the performance loop](../guides/performance-loop.md).
This report is the orientation that sits above both.

## 1. What fdu is, and what makes it hard to optimize

`fdu` reports disk usage.
It walks a directory tree, collects size and metadata for every entry, and renders
views: a tree, a file list, extension tallies, a summary.
It competes with `du`, `dust`, `dut`, `pdu`, and `diskus`.

Three properties make its performance work harder than a typical benchmark exercise.

**It has three tiers of retained state, and they behave differently.** An *aggregate*
run keeps only running totals and discards paths.
An *index* run retains an entry per file so later views are projections rather than
rescans. A *content* run additionally reads file contents for derived metrics.
A change that helps one tier routinely does nothing for the others, so every result must
name the tier it applies to.

**It is warm as well as cold.** A cold run walks the filesystem.
A warm run loads a snapshot from a previous run and revalidates it.
These paths share code but have opposite cost profiles: cold is syscall-bound, warm is
deserialization-bound.

**It is parallel on one side and serialized on the other.** Worker threads read
directories concurrently and hand observations to a single consumer that applies them
under a delta contract, so snapshots, queries, and change feeds cannot diverge.
That serialization is a correctness feature and the dominant cost on Linux.

Finally, correctness is not negotiable: every optimization must leave output
byte-identical. A faster wrong answer is not a result.

## 2. The method

The campaign runs a loop.
Its value is not any single step but that each pass leaves the next one cheaper.

1. **Instrument**, so a run says what it *did*, not only how long it took.
2. **Profile** before forming a hypothesis.
   Read a caller tree, not a flat profile.
3. **Write the hypothesis down**, naming the metric it moves and the tier and platform
   it applies to.
4. **Change one thing.**
5. **Verify identical output**, then measure paired and interleaved against a control.
6. **Apply the accept rule** without negotiation: median at least 3% better, the whole
   95% interval on the right side of zero, and the complexity worth it.
7. **Record the result** in a schema-validated experiment — the rejections especially.
8. **Re-screen the queue**, because the change just landed may have eaten the next
   hypothesis’s headroom.

### Why the accept rule is strict

Of 56 recorded experiments, **24 were rejected** — close to the 27 accepted.
Several were rejected despite a real, working mechanism, because the measured effect did
not clear 3%. That is the rule doing its job: a real mechanism is exactly what makes a
small number feel worth keeping, and a codebase that accumulates 1% wins for 50 lines
each becomes unmaintainable without becoming fast.

### Why rejections are recorded as carefully as wins

The negative results are the most reusable part of the ledger.
They stop the next person re-running a dead end.
Two examples that have already paid for themselves:

- **H13** proposed accumulating roll-up contributions per parent instead of merging to
  the root per entry. Obvious, and refuted at −2.5% — because H18’s interning had already
  taken the expensive part.
  Anyone who re-derives H13 from first principles will find the refutation before
  spending a day on it.
- **H11** was resolved *against* the change on the grounds that its target function has
  no production caller at all.

### The measurement harness

Two binaries are built — a control and a candidate — and run **paired and interleaved**
(A, B, A, B) against a real tree, so drift in machine state hits both arms.
Output equivalence is verified across every mode and view before any timing.
Surprising results are re-run in both orderings; if A:B and B:A disagree on the sign,
the effect is position bias or noise.

Performance gates are deliberately **not** in CI, because a timing gate on a shared
runner measures the runner.
Section 8 revisits whether that is still the whole truth.

## 3. Instrumentation: the three tiers

Instrumentation lives at three levels, and using one alone is the most common route to a
confident wrong conclusion.

| Tier | Source | Cost | Answers |
| --- | --- | --- | --- |
| Application | counters at call sites | ~1 ns/event | *which layer* did the work |
| Process | kernel, sampled per phase | one file read | what the process *really* did |
| External | `strace -c`, `perf`, callgrind | 2×–50× | ground truth, and call sites |

The application tier attributes cost to a layer, which is what tells you where to change
code — but it counts what the code *believes* it did.
The process tier is real kernel data that cannot be fooled and cannot attribute.
The external tier is authoritative and far too slow to leave on.

The mechanism lives in the [`fdu::counters`](../../../crates/fdu/src/counters.rs)
subsystem: thread-local non-atomic storage folded into process globals, a runtime enable
flag, a certified counting global allocator, and capability-specific Linux and macOS
process collectors.

Recording is off by default and enabled per run with `FDU_COUNTERS=1` — a runtime toggle
rather than a build flag, so visibility costs an environment variable instead of a
rebuild.

**The overhead is measured, not asserted.** With ~13 million counter increments per run
plus a counting allocator on every allocation:

| Question | Comparison | Result |
| --- | --- | --- |
| Idle cost | no instrumentation vs. compiled-in but off | −1.26% [−2.96%, +1.40%] |
| Recording cost | off vs. on, same binary | +0.64% [−0.68%, +2.13%] |

Both intervals span zero.
Precisely: recording is bounded below about 2.1%; it is not shown to be zero.

### What the tiers catch that each other cannot

A worked example.
Application counters reported 2,559 directory opens over a 17,128-entry
tree. `strace -c` on the same run reported 2,565 `openat` and 17,131 `statx` — both
matching, so those counters are sound — and **5,118 `getdents64`, exactly 2.00 per
directory read**.

Half of every directory-read pair carries no data: the second call returns zero to say
the directory is exhausted.
No application counter could show that, because at the call site there is exactly one
call. The counter was accurate about what the code did and wrong about what the kernel
did.

### A counter that reads zero is worse than no counter

A page of zeroes invites the conclusion that the work did not happen.
This failed three times while the instrumentation was being built: a counter added to
the serial walker while the parallel walker went untouched; a lint fix that hoisted a
call out of a `match` scrutinee and took the counter with it; and an entire platform
backend (`getattrlistbulk` on macOS) that replaces both `read_dir` and `stat` and
reported neither. All three compiled, passed every other test, and reported zero.

The guard is a test asserting against the system’s own totals, covering every path the
work can take, and verified by deleting a counter and watching it fail.

## 4. What has been achieved

### End-to-end, this campaign’s most recent branch

Measured on Linux, 450,463-entry tree, 18 interleaved trials per variant, control is the
branch point and candidate is its tip:

| Job | Control | Candidate | Change | 95% interval |
| --- | ---: | ---: | ---: | --- |
| `warm-snapshot-load` | 1897.2 ms | 1303.8 ms | **−31.4%** | [−32.0%, −30.8%] |
| `warm-revalidate` | 2317.6 ms | 1726.6 ms | **−25.3%** | [−26.4%, −23.8%] |
| `cold-scan-index` | 2107.8 ms | 1909.2 ms | **−9.1%** | [−13.2%, −7.6%] |
| `cold-scan-producer` | 2381.3 ms | 2200.2 ms | −7.3% | [−8.9%, −6.1%] |

Component times isolate where the work actually moved: the snapshot loader’s component
fell from 939.7 ms to 390.0 ms (**−58.5%**), and the index-build component from 979.3 ms
to 797.3 ms (**−18.6%**).

**One caveat, stated rather than buried.** `cold-scan-producer`’s *component* is
unchanged (345.3 ms against 346.8 ms) while its wall time improved 7.3%. Nothing in this
branch targeted the producer, so that wall difference is most likely outside the
measured component — process-level effects or environment drift — and should not be read
as a producer improvement.

### The largest individual wins

**Load a snapshot beneath the parent you already hold (−51.9%).** The loader had the
parent’s `EntryId` in a local variable, then spent a `PathBuf` join, a `normalize`
vector, and a descent from the root through one `BTreeMap` lookup per level to
rediscover it. A callgrind profile put the allocator at ~27.5% of the work and
path-component iteration at ~15%. Removing the re-derivation took snapshot load down
51.9% and warm open down 41.9%.

**Resolve an upsert’s parent once per directory, not once per entry (exp-051).** The
cold scan carried the same defect.
A walker reports a directory’s children consecutively, so remembering the previous
upsert’s parent answers almost every entry with one path comparison.
Wall −7.35% [−10.42%, −6.12%]; the index-build component −16.6%. The parent-memo hit
rate is 93.6%, and `normalize` instructions fell 89%.

**Look up content-cache candidates by hash, not by path order** (−3.0% / −3.5%).

**Per-platform tuning as data, not conditionals.** Tuning constants became a table
checked at compile time, with a parity test sweeping every table’s settings on every CI
platform, verified by deliberate breakage.

### The inversion that closed

The README long carried the claim that a warm run was *slower* than a cold one on a warm
laptop — measured, not assumed, and the stated current work.
**That has closed on Linux:** warm open now runs 22.6% faster than a cold scan, where
the campaign began with it 69% slower.

### Findings that changed direction rather than code

**mimalloc wins one tier and was not adopted.** Confirmed at −23.0% on the aggregate
tier — and rejected: a C-building dependency for a one-tier win, +139% RSS on the tier
whose pitch is low memory, unmeasured on macOS. Its real value was diagnostic.
Since H51 and H62 both cut allocation *counts* and were refuted, while mimalloc changes
no counts and wins, the cost is glibc’s cross-thread free path — one thread allocating,
another freeing. That became H85, with a dependency-free design.

**Allocation is producer-side, not index-side.** The campaign assumed for weeks that
allocation was concentrated in the index consumer.
Two counters inverted it: `scan-producer`, which walks without building an index,
allocates *more* than `scan-index` does — 8.8M against 6.9M. The jobs differ in what
they retain, so this is a direction rather than a clean subtraction, but it points away
from the consumer.

**Every `read_dir` costs two `getdents64` calls.** Half carry no data.
The cost scales with directory count rather than entry count, so it lands hardest on
wide shallow trees.

### Correctness and tooling improvements that came out of the work

- **Four Windows-gated defects**, two of them real bugs: `usize::is_multiple_of` is
  stable since Rust 1.87 while the project declares MSRV 1.85, so a Windows user on the
  declared minimum could not build the crate.
  The MSRV job runs on ubuntu, where those code paths do not exist.
- **`make cross-lint`**, which runs clippy against macOS and Windows targets locally.
  It checks rather than builds, so no cross-linker is needed.
  Before it existed, the module holding the repository’s only `unsafe` block had never
  been linted anywhere.
- **An ecosystem survey** establishing that hot-path counters in an optimization loop
  are a use case no mainstream Rust crate targets, and that on Linux there is no
  unprivileged, in-process, per-type syscall count at all.

## 5. Platform status: Linux and macOS

This is the most important section for anyone reading a number and deciding what it
means.

**The evidence is overwhelmingly macOS.** Of 56 recorded experiments, **53 were measured
on Darwin and 3 on Linux.** The macOS work is mature — a bulk-attribute reader using
`getattrlistbulk`, tuning constants measured across three APFS regimes, a scheduler
tuned against a real device.
The Linux work is recent and thin.

**A constant measured on one platform is inherited, not proven, on the other.** Which
shipped constants were measured where is recorded in
[the platform tuning guide](../guides/platform-tuning.md), and the distinction is
enforced in code: tuning is a table with a per-entry provenance marker, and a parity
test sweeps every platform’s table on every CI platform.

|  | macOS | Linux | Windows |
| --- | --- | --- | --- |
| Experiments recorded | 51 | 3 | 0 |
| Profiler available | bespoke script | callgrind | none |
| Bulk directory read | `getattrlistbulk` | `read_dir` (2 syscalls/dir) | `read_dir` |
| Process counter tier | total syscalls, faults | read/write syscalls, faults | none yet |
| Cross-lint coverage | yes | yes (host) | yes |
| Instruction-count gating | unavailable (no Valgrind on Apple Silicon) | possible | unavailable |

The macOS-specific engine work — the bulk reader and its scheduler — has no Linux
equivalent and needs none; `getdents64` plus `statx` is a different cost structure.
The open question is the reverse: which Linux findings transfer back.
The doubled `getdents64` calls are Linux-specific by construction.
The allocator diagnosis is not, and mimalloc’s −23% on the aggregate tier is explicitly
untested on macOS.

## 6. What remains

Nine hypotheses are open in the registry, plus structural items from a recent review.
Ordered by expected value:

| Target | Size | Blocked on |
| --- | --- | --- |
| `fdu-926e` — classification on every content open | ~34% of a warm content open | nothing |
| `fdu-2ubt` — batch-shaped observations | producer still clones a `PathBuf` per entry | nothing |
| `fdu-h7sw` (H85) — cross-thread free | screened against mimalloc’s own −23% | nothing |
| `fdu-fnfc` / `fdu-uv0s` — name arena, children as arena slices | RSS is the clearest defect | nothing |
| `fdu-vnwk` — cold bootstrap without arbitration | consumer is ~2.3 µs/entry vs dut’s ~0.1 | do the cheaper ones first |
| `fdu-refc` — per-directory extension tallies | retained everywhere, read almost nowhere | nothing |
| `fdu-jnuo` — the doubled `getdents64` | ~50% of directory-read syscalls | safety analysis |
| `fdu-zgxd` — 11 reallocations per entry | unattributed | needs `dhat-rs` |
| `fdu-73hj` (H73) — inode-ordered statting | 4–6× claimed in prior art | **bare metal** |

Two items are *unblocking* rather than optimizing, and multiply what later work can
settle: `fdu-tyjx` (the aggregate tier has no probe job, so it cannot be measured under
the accept rule at all) and `fdu-c65j` (adopt `samply`, so Linux and Windows can take
the loop’s first step).

## 7. Where the evidence is weak

Stated plainly, because these are the places a confident number could mislead.

**Platform asymmetry.** 53 experiments on macOS, 3 on Linux.
Every Linux constant not explicitly measured is inherited.

**No bare metal.** All Linux measurement here is virtualized.
A hypervisor’s page cache sits under the guest’s, so writing to `drop_caches` inside the
guest does not reach the disk.
That makes one class of claim untestable: anything whose mechanism is device latency or
I/O ordering.
H73 and the queue-depth hypotheses are marked accordingly, and the io_uring
results were not treated as settling the cold question.

**The aggregate tier cannot be measured under the accept rule.** There is no probe job
for it, so the tier where fdu competes with `diskus` has no gate.

**Windows has no performance evidence at all.** It builds and passes tests; nothing more
is claimed.

**Two hypotheses can compete for one cost.** This has happened twice — H13 lost to H18,
and H74 lost to the loader fix on a path a profile had put it at 27.5% of.
Any queued estimate is an upper bound until re-screened after the preceding change
lands.

**Some overhead bounds are bounds, not values.** Instrumentation recording cost is
bounded below ~2.1%; it is not zero, and the interval is wide because tightening it
costs more trials than the answer is worth.

**One cited figure is soft.** The `tracing` enabled-span cost used in the ecosystem
comparison is derived from a third party’s comparative benchmarks rather than
`tracing`’s own, which are not published for the enabled case.
The order of magnitude is not in doubt; the specific number is.

## 8. Open questions about the method itself

**Can a regression gate exist at all?** The standing position — a timing gate on a
shared runner measures the runner — is true of wall clock and false of instruction
counts. `iai-callgrind` counts user-space instructions deterministically.
It would be a *partial* gate: Valgrind does not instrument kernel code, and 29–62% of
this workload’s time is system CPU, so it would have missed the `getdents64` finding
entirely. A partial gate honestly labelled is worth having; one trusted for what it
cannot see is not. Tracked as `fdu-slgp`.

**Does an instruction-count regression predict a wall-time regression** in a parallel
program, given Valgrind serializes threads?
Unsettled, and it should be settled before adopting the gate.

**How much does the harness cost?** A probe’s own verification digest measured 38.8% of
one profile. Harness cost must be subtracted before any percentage is quoted.

## 9. How to reproduce any of this

```shell
# Build a control and a candidate probe.
cargo build --release -p fdu --example perf_probe --no-default-features

# Paired, interleaved measurement against a real tree.
uv run --project benchmarks --frozen python -m benchmarks.realtree measure \
  --root TREE --label NAME \
  --variant "control=PATH_A" --variant "candidate=PATH_B" \
  --job cold-scan-index --job warm-revalidate --trials 20

# Per-layer counters on any run.
FDU_COUNTERS=1 ./target/release/examples/perf_probe scan-index --root TREE 2>&1 >/dev/null

# Ground-truth syscall counts.
strace -f -c -e trace=getdents64,statx,openat ./target/release/fdu --cache off TREE

# Record the verdict, including rejections.
make perf-record ARGS="--run RESULTS.json --id exp-NNN --decision rejected ..."
```

The full protocol, including the accept rule and the hypothesis registry, is in
[the performance loop](../guides/performance-loop.md).

## 10. Document map

| Document | What it is |
| --- | --- |
| This report | Orientation: status, method, history, what remains |
| [Instrumentation playbook](../guides/performance-instrumentation-playbook.md) | The reusable method, domain-neutral |
| [Performance loop](../guides/performance-loop.md) | Protocol and live hypothesis registry |
| [Platform tuning](../guides/platform-tuning.md) | Which constants were measured where |
| [Experiment ledger](report-2026-08-10-fdu-performance-experiments.md) | Every experiment, accepted and rejected |
| [Performance architecture](report-2026-08-12-fdu-performance-architecture.md) | Cost model and architectural conclusions |
| [Systems optimization research](../research/research-2026-08-14-systems-performance-optimization-rust.md) | The general problem, Rust-specific |
| [Ecosystem survey](../research/research-2026-08-14-instrumentation-ecosystem-survey.md) | What the ecosystem already solves |
| [Structural review](../research/research-2026-08-14-structural-performance-review.md) | What 30 hypotheses had in common, and missed |

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
