# Research: Optimizing Systems Programs in Rust

**Date:** 2026-08-14 (last updated 2026-08-14)

**Author:** fdu project, with Claude Code research assistance

**Status:** In Progress — Linux complete, macOS sections are placeholders

## Overview

How do you make a Rust systems program measurably faster, know that you did, and leave
the next person able to go faster still?

This is the reference document for that question.
It exists because the answer turned out to be mostly about **measurement**, not about
optimization: the changes that worked were easy once the right number was visible, and
the expensive mistakes were all cases of optimizing against a number that did not mean
what it appeared to.

The motivating system is `fdu`, a parallel filesystem walker competing with `du`,
`dust`, `dut`, `pdu`, and `diskus`. Its shape is representative of the class:
syscall-bound at one layer, allocation-bound at another, parallel in the producer and
serialized in the consumer, with results that must stay byte-identical while the code
underneath changes. Nothing here is specific to walking a filesystem.

The companion documents are
[the instrumentation playbook](../guides/performance-instrumentation-playbook.md), which
is the condensed method, and [the performance loop](../guides/performance-loop.md),
which is fdu’s specific protocol and hypothesis registry.
This document is the reasoning behind both, plus the ecosystem survey that justifies the
tool choices.

**Scope note:** Linux is covered in depth.
macOS sections are deliberately left as structured placeholders for a later pass;
Windows is out of scope for this round and is noted only where a decision depends on it.

## Questions to Answer

1. What instrumentation does a Rust systems program need, and what does it cost?
2. Which of that should be built, and which already exists in the Rust ecosystem?
3. How do you get accurate syscall-level visibility on Linux, and what is genuinely
   unavailable?
4. What profiling tools are worth standardizing on, and which question does each answer?
5. How do you measure a change so the result is trustworthy?
6. Can any of this run as an automated regression gate in CI?

## Scope

**Included:** in-process instrumentation design and cost; the Rust metrics and tracing
crate ecosystem as it applies to hot paths; Linux kernel-level counters and their real
coverage; profiling tools; benchmark methodology for comparing two binaries against a
real workload; CI gating.

**Excluded:** distributed tracing and service observability (a different problem — see
[Eliminated Options](#eliminated-options)); micro-benchmarking individual functions,
which is well covered elsewhere; Windows, beyond noting where it changes a decision;
macOS specifics, deferred to a later pass.

## Findings

### The three tiers of instrumentation

Instrumentation lives at three levels.
Each answers a question the others cannot, and using one alone is the most common route
to a confident wrong conclusion.

| Tier | Source | Cost | Answers |
| --- | --- | --- | --- |
| Application | counters at call sites | ~1 ns/event | *which layer* did the work |
| Process | kernel, sampled per phase | one file read | what the process *really* did |
| External | `strace -c`, `perf`, callgrind | 2×–50× | ground truth, and call sites |

The application tier attributes cost to a layer, which is what tells you where to change
code — but it counts what *your code believes* it did.
The process tier is real kernel data that cannot be fooled and cannot attribute.
The external tier is authoritative about both and far too slow to leave on.

The discipline is cross-checking the application tier against the process tier, and
reaching for the external tier only when they disagree or when neither can name a call
site.

### What the cross-check catches

Application counters reported 2,559 directory opens over a 17,128-entry tree.
`strace -c` on the same run reported 2,565 `openat` and 17,131 `statx` — both matching,
so those counters are sound — and **5,118 `getdents64`, exactly 2.00 per directory
read**.

Half of every directory-read pair carries no data: the second call returns zero to say
the directory is exhausted, because the standard library cannot know otherwise.
No application counter could show that, because at the call site there is exactly one
call. The counter was accurate about what the code did and wrong about what the kernel
did.

That cost scales with directory count rather than entry count, so it lands hardest on
wide shallow trees — the common case.

### Linux syscall visibility: what is actually available

`/proc/self/io` reports `syscr` and `syscw`, which read like syscall counts and are not.
They count the read and write families only.
Measured directly: a walk over 17,128 directory entries — every one a `getdents64` or a
`statx` — moved `syscr` by **30**.

The full picture for a process wanting to count its own syscalls on Linux:

| Mechanism | Counts enumeration/stat? | Privilege | Overhead |
| --- | --- | --- | --- |
| `/proc/self/io` | No — read/write only | None | one file read |
| `/proc/self/stat` | N/A — page faults | None | one file read |
| `getrusage` | No | None | one syscall |
| `taskstats` netlink | No — same counters | `CAP_NET_ADMIN` | netlink round-trip |
| `perf_event_open` tracepoint | Yes | `CAP_PERFMON` | ~8 ns/event |
| eBPF | Yes | `CAP_BPF` + `CAP_PERFMON` | <1% |
| seccomp-BPF | **Cannot count at all** | None | N/A |

The conclusion is worth stating plainly: **on Linux there is no unprivileged,
in-process, per-type syscall count.** Application counters at the call site, validated
periodically against `strace -c`, are not a compromise — they are the only instrument
that works without privilege.
The three-tier structure is a consequence of that fact rather than a design preference.

### Keeping instrumentation from distorting what it measures

Instrumentation that changes what it measures is worse than none, because it is
believed. Four rules, in descending order of how much they matter:

1. **Thread-local and non-atomic.** A shared atomic in a parallel section measures the
   counter’s contention, not the work.
   Threads fold into a global total when they finish.
2. **Count, do not time, on per-event paths.** A clock read costs an order of magnitude
   more than an integer increment.
   Time whole phases; count events.
3. **Toggle at runtime, not only at build time.** A build flag means two binaries and a
   rebuild to see anything — friction in exactly the loop that should be frictionless.
4. **Measure the overhead and record it**, re-running when the instrumentation grows.

Measured on this system, with ~13 million counter increments per run plus a counting
global allocator on every allocation:

| Question | Comparison | Result |
| --- | --- | --- |
| Idle cost | no instrumentation vs. compiled-in but off | −1.26% [−2.96%, +1.40%] |
| Recording cost | off vs. on, same binary | +0.64% [−0.68%, +2.13%] |

Both intervals span zero.
Precisely stated, that bounds recording below about 2.1% — it does not establish zero,
and tightening it further would cost more trials than the answer is worth.

The **idle** measurement is the one people forget, and it is the one that justifies
leaving the code in permanently.

### A counter that reads zero is worse than no counter

A page of zeroes invites the conclusion that the work did not happen.
This failed three times while building the instrumentation described here:

- A counter added to the serial walker while the parallel walker — the one actually used
  — went untouched.
- A lint fix that hoisted a call out of a `match` scrutinee and took the counter with
  it.
- An entire platform backend (`getattrlistbulk` on macOS) that replaces both `read_dir`
  and `stat`, and reported neither.

All three compiled. All three passed every other test.
All three reported zero.

The guard is a test asserting **equality against the system’s own totals**, not
non-zero, covering every path the work can take.
An earlier version covered one of two walkers and passed with the other’s counter
deleted.
Verify the guard by deleting a counter and watching it fail; a test you have not
seen fail is not yet evidence.

Where a number genuinely cannot be obtained, leave the counter out and say why.
Absent is honest; pinned to zero is a lie with a plausible face.

### What to count

Aim at the layers where systems actually spend time:

- **Syscalls by kind, at the call site.** Enumeration, stat, open, read.
- **Allocation** — count, reallocation count, bytes.
  Routinely the largest line in a systems profile and the least visible without a
  profiler. Here it was ~35% of a cold scan’s engine work, and finding that took
  callgrind.
- **Work the code chose to do**: cache hits versus misses, retries, fallbacks, batch
  sizes. These distinguish a change that removed work from one that moved it.
- **Ratios per unit of work.** “Allocations per entry” transfers between workloads of
  different sizes; “6.9 million allocations” does not.

A worked example of why ratios matter: this system reports 15.4 allocations, 11.0
reallocations, and 2,456 bytes allocated **per directory entry**. Those numbers
immediately raise questions that the absolute totals do not.

### Measurement methodology

**Verify identical output before timing anything.** A faster wrong answer is not a
result. Compare across every mode and view, ignoring only genuinely nondeterministic
fields — and check the control against *itself* first, so you learn which fields those
are rather than assuming.

**Pair and interleave.** Run A, B, A, B, so drift in machine state hits both arms.

**Run both orderings** when a result is surprising.
If A:B and B:A disagree on the sign, the effect is position bias or noise.

**Have an accept rule, written down, applied without negotiation.** Here: median at
least 3% better, the whole 95% interval on the right side of zero, and the complexity
worth it.

**Predict the metric the rule scores.** A prediction of “15%” describing a component
while the rule scores wall time is a category error, not a near miss.
It happened here: the component moved 16.6%, the wall 7.35%.

**Expect a surprising result to be noise until it survives.** A 25-pair run reported a
3.42% regression with an interval clear of zero.
It was mechanically plausible — the code path really was reached.
At 45 pairs it vanished; both orderings disagreed; a third harness put it at +0.54%
spanning zero.

### Recording results

Record every experiment, and the rejections most of all.
Negative results stop the next person re-running a dead end; in this campaign more
experiments were refuted than confirmed, and the refutations are what make the queue
trustworthy.

Use a **schema-validated format** so every entry answers the same questions: hypothesis
and the tier it applies to, control and candidate precisely enough to rebuild, primary
job and primary metric, median and interval and trial count, decision with a reason
someone can disagree with, lines changed, new dependencies, new unsafe, and the
host/platform/cache regime.
A constant measured on one platform is *inherited*, not proven, on others.

## Key Insights

**Most of optimization is measurement.** Every change that worked here was easy to write
once the right number was visible.
Every expensive mistake was optimizing against a number that did not mean what it
appeared to.

**Instrument before optimizing, not after.** A loop with good visibility gets cheaper
every pass. Building the counters mid-campaign meant re-deriving with callgrind facts
that a counter now prints for free — the parent-memo hit rate took a 50×-slowdown
profiling tool to establish and is now a number on an ordinary build.

**The obvious counter is often measuring the wrong layer.** `syscr` looks like a syscall
count. `read_dir` looks like one syscall.
Both assumptions were wrong, and both were caught only by cross-checking tiers.

**Two hypotheses can compete for one cost.** Whichever lands first captures the win and
the second measures noise.
This happened twice.
Re-screen the queue after every landing rather than working down the list.

**Defensive code you cannot test is worse than none.** A guard for a hazard that cannot
occur reads as evidence the hazard exists.
If no test fails without it, delete it and leave the reasoning as a comment — or better,
as a test that pins the reasoning.

**Assumptions about which layer is expensive should be checked, not carried.** This
campaign assumed for weeks that allocation was concentrated in the index consumer.
Two counters inverted it in the time it took to run two probe jobs: the walk-only job
allocates *more* than the walk-plus-index job.

## Comparison Matrix

Instrumentation approaches for a hot path, ~13M events per run:

| Criterion | Bespoke thread-local | `metrics` + recorder | `tracing` spans | eBPF/`perf` |
| --- | --- | --- | --- | --- |
| Per event | 1–2 ns | ~20–100 ns | ~1,000–4,000 ns | ~8 ns kernel-side |
| Cost at 13M events | ~20 ms | 260 ms–1.3 s | prohibitive | n/a |
| Dependencies | 0 | 20+ | 10+ | external tooling |
| Per-type syscalls | yes, at call site | yes, at call site | yes, at call site | yes, real |
| Allocation counting | yes | no | no | partial |
| Privilege needed | none | none | none | `CAP_PERFMON`/`CAP_BPF` |
| Attribution to call site | no | no | yes | yes |

Profiling and benchmarking tools, by question answered:

| Tool | Answers | Platform | Overhead | Deterministic |
| --- | --- | --- | --- | --- |
| callgrind | exact instruction counts, caller tree | Linux | 20–50× | yes |
| `samply` | where wall time goes | Linux/macOS/Win | ~1–2% | no (sampling) |
| `dhat-rs` | which call site allocates | Linux/macOS/Win | high | yes |
| `strace -c` | true syscall counts by type | Linux | 2–60× | yes |
| `bpftrace` | syscall counts, latency histograms | Linux | <1–2% | yes |
| `iai-callgrind` | instruction-count regressions in CI | Linux | 4–20× | yes |
| paired A/B harness | does the change make it faster | any | none | no |

## Options Considered

### Option A: Build a small bespoke instrumentation crate

**Description:** A `counters!` macro declaring named `u64` counters in thread-local
storage, folded into process-global atomics; a runtime enable flag; a generic counting
global allocator; a process-tier snapshot reading `/proc`. Roughly 700 lines, no
external dependencies.

**Pros:**
- 1–2 ns per event, which is what makes always-on affordable at this event rate
- Zero dependencies, which matters under a supply-chain cool-off policy
- Allocation counting, which nothing in the ecosystem currently provides
- The process tier, which nothing in the ecosystem provides

**Cons:**
- Code to maintain that is not the product
- No call-site attribution — counters localize cost to a layer and stop there
- Process-global state, so concurrent tests must serialize
- Written before surveying the ecosystem, which is the wrong order even when the answer
  comes out the same

### Option B: Adopt an ecosystem observability crate

**Description:** Use `tracing`, the `metrics` family, `fastrace`, or `opentelemetry` for
in-process instrumentation.

**Pros:**
- Maintained by others, familiar to contributors
- Rich ecosystem of exporters and subscribers
- Call-site attribution and structured context come free

**Cons:**
- Built for observability — telemetry exported to a collector at request granularity —
  which is a different problem from paired A/B measurement in an optimization loop
- 20–100 ns per event with a recorder attached, against a total runtime of a few hundred
  milliseconds
- 10–20+ transitive dependencies under a 14-day cool-off
- None counts allocations; `tracking-allocator`, the nearest thing, last released July
  2022

### Eliminated Options

- **seccomp-BPF for syscall counting:** eliminated because classic BPF has no writable
  state. A filter can decide but cannot tally, and the return codes that would let a
  supervisor count (`SECCOMP_RET_USER_NOTIF`, `SECCOMP_RET_TRACE`) block the caller
  until the supervisor answers.
  The idea is obvious and does not work.
- **`perf_event_open` on syscall tracepoints, in-process:** eliminated for always-on use
  because tracepoint access needs `CAP_PERFMON` or `perf_event_paranoid = -1`, which
  containers and CI runners do not have.
  Retained as an external validation tool.
- **eBPF self-attachment:** eliminated for the same privilege reason (`CAP_BPF`).
  Retained as `bpftrace` for investigation.
- **`taskstats` over netlink:** eliminated because its `read_syscalls`/`write_syscalls`
  are the same counters as `/proc/self/io`, so it adds a `CAP_NET_ADMIN` requirement and
  no information.
- **Windows Performance Counters:** eliminated because collection granularity is once
  per second, which Microsoft’s own documentation says is unsuitable for application
  profiling.

## Recommendations

**Build the hot-path counters; adopt everything else.** The bespoke crate is justified
narrowly — hot-path counting in an optimization loop is a use case no surveyed crate
targets — and that argument does not extend to profiling, allocation attribution, or
benchmarking, where mature tools exist and should be used.

Specifically, for a Rust systems program on Linux:

1. **Thread-local counters at call sites**, runtime-toggled, with the overhead measured
   and recorded. Count syscalls by kind, allocations, and chosen work.
2. **A counting global allocator**, installed by the binary rather than the library, so
   library consumers keep the choice.
3. **A process tier** reading `/proc/self/io` and `/proc/self/stat`, sampled per phase,
   used to cross-check the application tier — never as the primary instrument, given its
   coverage limits.
4. **`strace -c` as periodic ground truth**, run deliberately rather than continuously.
5. **`samply` for “where does time go”** and **callgrind for “exactly what changed”**.
   They answer different questions and both are worth keeping.
6. **`dhat-rs` when a counter localizes a cost it cannot attribute.**
7. **A paired, interleaved A/B harness** comparing two binaries against a real workload,
   with output equivalence verified before any timing.

**On a CI gate:** instruction counting via `iai-callgrind` makes one reachable, and it
must be advertised as partial.
Valgrind does not instrument kernel code, and 29–62% of this workload’s time is system
CPU — a gate would catch index-build, allocator, and serialization regressions, and
would have missed the `getdents64` finding entirely.
A partial gate honestly labelled is worth having; one trusted for what it cannot see is
not.

## macOS *(placeholder — to be completed in a later pass)*

> The sections below are deliberately unfilled.
> Linux findings above should not be assumed to transfer: the syscall interface, the
> allocator, and the filesystem all differ, and this project has already found constants
> that were measured on one platform and merely inherited on the other.

### macOS syscall visibility

> To investigate: `proc_pidinfo` with `PROC_PIDTASKINFO` gives `pti_syscalls_unix` and
> `pti_syscalls_mach` unprivileged and unaffected by SIP, but as a total rather than a
> per-type breakdown. Establish what that total includes, whether the `i32` fields wrap
> in practice, and how it compares against `dtrace` per-type counts.

### macOS profiling tools

> To investigate: `samply` via DTrace, Instruments’ System Trace and Time Profiler, and
> what SIP restricts for a self-signed binary versus a system one.
> Note that Valgrind does not run on Apple Silicon, so callgrind and `iai-callgrind` are
> unavailable.

### macOS allocator behaviour

> To investigate: how the system allocator differs from glibc under the same workload.
> Relevant because mimalloc measured −23% on one tier on Linux and that result is
> explicitly untested on macOS.

### Platform-specific syscall interfaces

> To investigate: `getattrlistbulk` returns directory entries with metadata in one call,
> which is a different cost structure from `getdents64` plus per-entry `statx`. Quantify
> the difference and note which Linux findings it invalidates.

### What transfers and what does not

> To be written once the above is filled in.
> The existing [platform tuning guide](../guides/platform-tuning.md) records which
> shipped constants were measured on which platform and which are merely inherited; this
> section should reconcile with it.

## Next Steps

- [ ] Fill in the macOS sections (assigned to a later pass)
- [ ] Evaluate `iai-callgrind` for a partial CI gate — `fdu-slgp`
- [ ] Adopt `samply`, closing the Linux and Windows profiling gap — `fdu-c65j`
- [ ] Use `dhat-rs` to attribute the eleven reallocations per entry — `fdu-zr3a`
- [ ] Extend the process tier to macOS and Windows — `fdu-3b7v`
- [ ] Investigate the doubled `getdents64` calls — `fdu-jnuo`

## Methodology

Findings came from three parallel literature searches (ecosystem crates, profiling
tools, OS-level syscall visibility), each cross-referenced against primary sources —
kernel documentation, crate documentation and release history, and published benchmarks
— plus direct measurement on this system for every number attributed to `fdu`.

Empirical claims about this codebase were measured rather than cited: the `syscr`
coverage limit, the doubled `getdents64` calls, the instrumentation overhead, and the
per-entry allocation ratios were all produced by running the code described.

**Known uncertainties.** The `tracing` enabled-span cost is derived from `fastrace`’s
comparative benchmarks rather than `tracing`’s own, which are not published for the
enabled case; the order of magnitude is not in doubt but the specific figure is soft.
The `metrics`-with-recorder figure is an estimate from the operations involved rather
than a published benchmark.
Whether an instruction-count regression under Valgrind’s thread serialization reliably
predicts a wall-time regression in a parallel program is an open question that should be
settled before adopting a CI gate.

## References

Kernel and platform documentation:

- [perf security and `perf_event_paranoid`](https://docs.kernel.org/admin-guide/perf-security.html)
  — official
- [taskstats struct](https://docs.kernel.org/accounting/taskstats-struct.html) —
  official
- [`getrusage(2)`](https://man7.org/linux/man-pages/man2/getrusage.2.html) — official
- [seccomp filter overhead](https://lwn.net/Articles/832428/) — kernel patch benchmarks
- [`IO_COUNTERS`](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-io_counters)
  — official

Tools:

- [`perf` overhead notes](https://www.brendangregg.com/perf.html) — independent
- [bpftrace one-liners](https://github.com/bpftrace/bpftrace/blob/master/docs/tutorial_one_liners.md)
  — official
- `iai-callgrind` v0.19.4 (renamed Gungraun), `samply` v0.13.1, `dhat-rs` v0.3.3,
  `criterion` v0.8.2, `divan` v0.1.21, `tango` v0.7.2 — crate documentation and release
  history

Project documents:

- [The instrumentation playbook](../guides/performance-instrumentation-playbook.md)
- [The performance loop](../guides/performance-loop.md)
- [Ecosystem survey](research-2026-08-14-instrumentation-ecosystem-survey.md)
- [Structural performance review](research-2026-08-14-structural-performance-review.md)
- [Experiment ledger](../reports/report-2026-08-10-fdu-performance-experiments.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
