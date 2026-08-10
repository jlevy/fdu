# Research: End-to-End Performance Evidence for fdu

**Date:** 2026-08-09

**Author:** fdu project

**Status:** Complete

## Overview

This research defines what fdu must measure before it can make a performance claim and
which parts of the Flowmark benchmark work are worth carrying into this repository.
The decision is not merely which timing tool to run.
It is how to build a reproducible evidence system for a stateful filesystem CLI whose
work changes with the snapshot state, operating-system page cache, output mode, scan
scope, and retained metrics.

The immediate output is the companion
[end-to-end performance testing plan](../specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md).
That plan is self-contained; this document preserves the source review, rejected
shortcuts, and reasoning behind it.

## Questions to Answer

1. Which lessons from Flowmark’s corpus, comparison, profiling, parallelism, and cache
   work transfer to fdu?
2. Which benchmark jobs and state transitions correspond to real fdu product paths?
3. How can fdu compare tools honestly when they retain different metadata and expose
   different cache behavior?
4. Which workload families are minimal but sufficient to reveal scale, tree-shape,
   metadata, cache, churn, and output bottlenecks?
5. What evidence, provenance, and statistical rules make a result reproducible and a
   regression gate defensible?
6. Which measurements belong in every pull request, on a stable scheduled runner, and in
   a release report?

## Scope

Included:

- Source review of `jlevy/flowmark-rs` v0.3.2 at commit
  [`015f23989af3e5cfb3f8b58dfc72822c534df25a`](https://github.com/jlevy/flowmark-rs/commit/015f23989af3e5cfb3f8b58dfc72822c534df25a)
- Flowmark’s corpus generator, benchmark runners, comparison report, profiling script,
  parallel-processing plan, and incremental-cache roadmap
- fdu’s current scan, index, snapshot, CLI, Python, Phase 1, golden-test, and
  engineering quality contracts
- End-to-end latency, phase probes, peak memory, throughput, resource counters,
  correctness equivalence, and regression governance

Excluded:

- Producing performance results from the current portable scaffold
- Choosing an optimization before a profile identifies it
- Treating third-party published numbers as fdu baselines
- Adding a third workspace crate, a benchmark-only production dependency, or a public
  diagnostics API without a demonstrated consumer
- Replacing the exact CLI golden suite with performance assertions

The Flowmark checkout lives under ignored `attic/` and was treated as read-only source
material.
Its repository instructions, agent/editor configuration, hooks, submodules, and
build script were inspected without executing code from the checkout.

## Findings

### What Flowmark Got Right

Flowmark’s most transferable result is its measurement decomposition.
It did not treat one `time command` invocation as a performance program.

- It generated a deterministic, nested corpus from known source material rather than
  depending on a developer’s current working tree.
- It separated file discovery, one-file formatting, whole-corpus formatting, semantic
  mode, fresh input, already-formatted input, first cached run, and second cached run.
- Its comparison runner restored a pristine working corpus before a mutating fresh run.
- It recorded tool versions, platform, corpus count and size, raw samples, means,
  variability, throughput, and caveats about capability differences.
- It measured thread scaling at one, two, four, and all available threads instead of
  assuming that more workers helped.
- It established output parity and retained the existing behavioral tests before making
  a performance optimization.
- It profiled after measuring.
  Callgrind identified repeated whole-string searches as the dominant cost; the
  implementation removed that work, then re-profiled to verify the causal explanation
  rather than trusting wall time alone.
- Its incremental-cache plan treated first-run cost, unchanged second-run cost,
  invalidation correctness, corruption recovery, and cache lifecycle as different
  acceptance questions.

The reusable sequence is:

1. define the job and state;
2. create a reproducible input;
3. prove behavioral equivalence;
4. collect repeated end-to-end measurements;
5. profile the measured bottleneck;
6. optimize one cause;
7. rerun correctness, timing, and the profile.

### What fdu Should Improve on Flowmark’s Harness

Flowmark’s scripts were effective project tools, but copying them would preserve limits
that matter more for a filesystem engine.

**One corpus is not a workload model.** Flowmark replicated repository Markdown files
into one 4-5-level tree.
That was enough to find formatter hot spots, but it cannot separate entry-count scale,
wide versus deep topology, inode density, hardlinks, symlinks, sparse allocation,
partial errors, or churn.
fdu needs parametric workload families and scale points, not one headline corpus.

**Fresh input is not the same as a cold filesystem cache.** Restoring files gives a
formatter unformatted content, but it does not evict their dentries, inode metadata, or
pages from the operating-system cache.
fdu must name snapshot state and filesystem-cache state independently.
A new process is not proof of a cold filesystem cache.

**Commands need structured execution.** The Flowmark comparison runner stores shell
commands as strings and executes them with `eval`; it also uses shared fixed paths under
`/tmp`. A reusable fdu runner should pass argument vectors directly, allocate a unique
run directory, record every state transition, and support paths containing whitespace
without shell interpretation.

**Mutating tools need explicit postconditions.** Restoring a pristine corpus before a
run is necessary but insufficient.
A runner also needs to verify the observed entry manifest, output digest, exit status,
cache source, and expected mutation after every trial.
A fast run that silently skipped work is a failed trial, not a good result.

**Cross-tool tables need a capability contract.** Flowmark correctly disclosed that the
compared formatters perform different transformations, but some ranking prose still
compresses those differences into one relative-speed number.
fdu should publish the exact job, flags, retained fields, traversal scope, hardlink
policy, and output handling next to every comparison.
Ratios do not make unlike jobs equivalent.

**Raw data must generate the report.** Flowmark retained JSON and Markdown output from
some runs, while its long-lived report also contains manually assembled snapshots from
different corpus revisions.
fdu should store a versioned result record for every trial and render summary tables
from those records. A table without its run manifest is not a release artifact.

**Variability is evidence, not an inconvenience.** A coefficient of variation above a
declared stability threshold should trigger investigation of thermal state, background
load, filesystem state, or an undersized workload.
The runner must never discard an unfavorable sample or repeat until the result looks
good.

### fdu Has Two Independent Cache Axes

The overloaded words *cold* and *warm* are not precise enough for fdu.
Every result needs both axes below.

| Axis | States | What it changes |
| --- | --- | --- |
| Snapshot state | absent, compatible unchanged, compatible changed, corrupt, incompatible scope/fingerprint | Whether fdu scans, loads, revalidates, or falls back |
| Filesystem cache state | uncontrolled, verified warm, controlled cold | Cost of directory enumeration, metadata lookup, snapshot reads, and output writes |

Process state is held constant for CLI end-to-end trials: every timed trial starts a new
process. Component probes may deliberately reuse a process, but their result names must
say so.

The full cold-cache procedure is necessarily host-specific.
On Linux it may require a dedicated runner with authority to evict caches or a corpus
larger than available cache; doing that on a shared developer machine is intrusive and
unreliable. Standard local and pull-request runs should report filesystem state as
`uncontrolled` or `verified-warm`, never infer `cold` from a deleted fdu snapshot.

### Benchmark Jobs Must Name the Work

The current architecture exposes natural boundaries that can be measured without
inventing benchmark-only production behavior.

| Job | Included work | Excluded work | Why it exists |
| --- | --- | --- | --- |
| `scan-producer` | Enumerate, stat, build observation batches, and produce a compact aggregate summary | Index apply, snapshot, rendering | Measures the delta producer and replaces the vague phrase *raw walk* |
| `scan-index` | `scan-producer` plus index construction and roll-ups | Snapshot and rendering | Measures a complete stat-tier engine scan |
| `snapshot-save` | Serialize and atomically replace one complete index | Scan and query | Detects writer, compression, and write-amplification costs |
| `snapshot-load` | Validate and materialize or open one snapshot | Revalidation | Separates storage-format cost from filesystem truth checking |
| `revalidate` | Load baseline, compare filesystem, apply effective deltas | Final CLI rendering unless named | Measures unchanged and churned warm paths honestly |
| `cli-human` | New process through selected human output | Nothing | Product latency, including startup and rendering |
| `cli-json` | New process through complete machine output | Nothing | Agent path and serialization/output-volume cost |
| `python-open-query` | Import installed wheel, open, and make a bulk query | Wheel build/install | Protects the embedding path from FFI or conversion regressions |
| `delta-apply` | Apply a deterministic mutation stream to an existing index | Filesystem I/O | Explanatory component probe for watch/reconcile work |

The external CLI timing is authoritative for user-visible latency.
Component probes explain where a change came from and can guard a local regression, but
they cannot substitute for the end-to-end result.

The complete normalized-record digest belongs to an untimed validation run.
Sorting or hashing every record inside an engine timing would measure the oracle as much
as the scan. Timed component probes return compact counts and aggregate summaries, while
the scenario is accepted only when exact validation with the same binary, arguments,
corpus, and state also passes.
Complete CLI output remains hashed because producing it is part of that end-to-end job.

For future lazy snapshots or stale-while-revalidating behavior, the runner must record
both time to first labeled output and time to complete trustworthy output.
They are the same in the current blocking `open()` implementation; preserving both
fields now prevents a future feature from redefining *open time* silently.

### A Minimal Workload Matrix Is Parametric, Not Cartesian

The matrix should vary one load-bearing dimension at a time, plus a small set of mixed
product scenarios.

| Family | Representative points | Question answered |
| --- | --- | --- |
| Scale curve | 10k, 100k, 500k, 1M entries | Fixed cost, throughput curve, and the Phase 1 target |
| Topology | wide, deep-within-platform-limits, balanced, mixed | Queueing, path reconstruction, recursion, and directory overhead |
| Metadata | tiny files, mixed sizes, sparse files, hardlink groups, symlinks | Stat fields, allocated/apparent bytes, and attribution cost |
| Snapshot | absent, unchanged, corrupt, wrong scope/fingerprint | Open path and fail-closed fallback cost |
| Churn | one change, 1% modify, 1% add/remove/rename, directory-local burst | Revalidation and delta proportionality |
| Output | digest only, bounded human tree, complete JSON | Engine cost versus presentation and pipe cost |
| Resilience | unreadable/racing subtree where portable | Partial-result cost and correctness; not a speed ranking |

The committed input is a compact recipe with a fixed seed, not hundreds of thousands of
files. Generation produces an observed manifest containing entry and directory counts,
depth, logical and allocated bytes where portable, hardlink/symlink counts, and a recipe
hash. Every timed trial validates that manifest or a declared post-mutation manifest.

Synthetic corpora are the release gate because fdu’s Phase 1 stat tier does not inspect
file content and synthetic recipes are reproducible.
Optional public real-world trees may challenge the model, but their results stay
exploratory unless acquisition, licensing, checksums, and exact layout are reproducible.

### Correctness Is a Precondition to Timing

Every scenario has a semantic oracle independent of elapsed time.

- A small contract corpus is checked against exact expected paths, kinds, apparent and
  allocated sizes, counts, mtimes under normalized assertions, and extension tallies.
- Large generated corpora are checked against generator-derived totals plus a stable
  digest of normalized records.
  Sampling alone is not enough for a claimed run.
- Snapshot scenarios assert the reported open path and cache postcondition.
- Churn scenarios assert the exact added, removed, changed, and unchanged result digest.
- CLI scenarios assert exit status, parse complete JSON where selected, and hash the
  output artifact. The existing tryscript suite remains the text contract.
- Comparator adapters validate traversal scope and parse a result sufficient to catch a
  no-op, partial traversal, or hidden exclusion.
  If a tool cannot expose enough data, the limitation is recorded and the comparison
  cannot support a correctness claim.

Performance optimization never relaxes the delta-only mutation contract, error policy,
fingerprint, retained metrics, or output schema.
A faster implementation with reduced semantics is a new job, not an improvement to the
old one.

### Comparisons Need Adapters and a Capability Matrix

The Phase 1 plan names dut and gdu because they represent a syscall-efficient walker and
a multi-metric disk-usage tool.
Neither performs exactly fdu’s job.

Each adapter must declare:

- source repository, revision or released version, license, and binary digest;
- exact build profile and command argument vector;
- root, filesystem-boundary, symlink, hidden/ignored, hardlink, and error policy;
- apparent versus allocated size semantics;
- which metadata and per-directory aggregates survive the run;
- whether output generation is included, redirected, or suppressed;
- whether the tool has any persistent cache and how it was reset;
- a parser or postcondition strong enough to reject a partial or empty run.

Results should be grouped by job compatibility:

1. **Traversal baseline:** the narrowest common directory walk and metadata job.
2. **Normal product job:** each tool’s documented useful operation, with capabilities
   shown rather than called equivalent.
3. **fdu full-stat job:** fdu retains the complete index and all configured roll-ups;
   this is the Phase 1 product gate even when the comparator retains less.

The existing target, fdu full-stat controlled-cold scan within roughly 1.5x of dut on
the same corpus, is intentionally ambitious.
The report must call it a cross-capability product target, not a like-for-like proof
that both tools did identical work.

### Measurement and Provenance Are One Design

A useful result record needs enough information to reproduce or reject it.

Required run metadata:

- fdu source revision and dirty state;
- release profile, features, target triple, Rust version, and binary checksum;
- tool/adaptor revisions and binary checksums;
- operating system, kernel, architecture, CPU model and logical count, memory, and
  filesystem type;
- power/virtualization state when discoverable, without recording hostnames, usernames,
  or personal absolute paths;
- corpus recipe id, seed, recipe hash, and observed-manifest hash;
- scenario, job, snapshot state, filesystem-cache state, output sink, and exact argv;
- a minimal normalized environment allowlist covering locale, timezone, cache roots,
  thread controls, allocator/build controls, and disabled implicit configuration;
- warmups, trial count, randomized execution order, timeout, and collector availability;
- per trial: wall, user and system CPU, peak RSS, major/minor faults, bytes
  read/written, exit status, output digest, and any supported phase or syscall counters.

Wall time is measured outside the process.
An fdu-emitted phase trace is diagnostic and must not redefine the total.
Unavailable platform counters are explicit `null` values with a reason, never zeros.

The runner should execute paired competitors in randomized or rotating order so one tool
does not consistently receive a warmer cache or cooler CPU. Setup, corpus restore, cache
preparation, and oracle validation are outside the timed region but remain in the run
log.
Corpus, snapshot, and filesystem-cache state must be re-established for every warmup
and timed invocation; otherwise later trials measure state left by earlier tools.

### Statistical Rules Must Resist Favorable Reruns

The result renderer should report every raw trial plus median, median absolute
deviation, p95 when the sample count supports it, range, and throughput.
Means and standard deviation may be included for continuity with common tools, but they
should not be the only summary for noisy I/O.

For release evidence:

- use at least ten independent timed samples per headline scenario, and enough total
  measured time to dominate timer resolution and process startup noise;
- require at least twenty valid samples before reporting p95;
- interleave paired tools and compare the paired ratio on the same host and corpus;
- treat coefficient of variation above 10% as an investigation trigger, not a reason to
  hide samples;
- record thermal, background-load, or collector anomalies and rerun the *whole declared
  set* only after the cause is corrected;
- do not delete outliers unless a predeclared mechanical rule identifies an invalid
  trial, and retain invalid trial records with the reason;
- compare against a baseline only when the job, corpus, host class, build contract, and
  result schema are compatible.

A regression gate needs both a practical effect threshold and noise evidence.
Tiny statistically visible changes should not block work; large noisy changes should not
pass. The first stable baseline run sets scenario-specific noise bands rather than
inventing one universal percentage now.

### Performance Testing Has Three Automation Tiers

| Tier | Trigger | Purpose | Claim strength |
| --- | --- | --- | --- |
| Smoke | Pull requests and `make check` only if cheap and deterministic | Build probes, validate recipes/schema/oracles, run tiny scenarios | Correctness of harness; no speed claim |
| Regression | Scheduled or protected stable runner | Medium corpus, repeated key paths, compatible-baseline comparison | Detect material regressions on that runner class |
| Release evidence | Manual/protected dedicated host | Full scale/state/job/comparator matrix, controlled cold runs, profiles | Supports README and release claims |

Generic hosted CI is appropriate for smoke tests and perhaps coarse catastrophic
timeouts. It is not a stable stopwatch.
A pull request should not fail because a shared runner was noisy, and a green shared
runner must not be cited as proof of a speed ratio.

## Key Insights

1. **State is part of the input.** A corpus path alone does not identify a benchmark;
   snapshot and filesystem-cache state are independent required fields.
2. **Benchmark jobs, not executable names.** `fdu`, `dut`, and `gdu` are not units of
   work. Exact commands, semantics, and retained results are.
3. **The corpus recipe is production code.** If it cannot deterministically recreate and
   validate the measured tree, the baseline cannot outlive the machine that made it.
4. **Output can dominate the product path.** Engine throughput, first trustworthy
   output, bounded human rendering, and complete JSON need separate results.
5. **Correctness oracles prevent benchmark gaming.** A no-op, narrower scan, stale
   cache, or discarded metric must fail before its duration enters a table.
6. **Profiles explain; external timing decides.** Instruction, syscall, allocation, and
   phase measurements guide optimization, while new-process end-to-end wall time owns
   the user claim.
7. **A benchmark report is generated evidence.** Raw immutable records plus a renderer
   are the source of truth; prose summarizes them and never becomes a second database.

## Options Considered

### Option A: Copy the Flowmark Shell Harness

**Description:** Adapt its corpus and comparison shell scripts directly.

**Pros:**

- Proven to produce useful measurements quickly
- Familiar workflow and hyperfine-compatible output

**Cons:**

- Shell-string execution, platform-specific timing, shared temporary paths, and mutable
  state are poor foundations for a long-lived multi-state filesystem benchmark
- One corpus and one summary table cannot express fdu’s workload or cache dimensions

**Decision:** Reject as an implementation, retain its decomposition and reporting
lessons.

### Option B: Use Only Criterion Microbenchmarks

**Description:** Benchmark Rust functions in-process.

**Pros:**

- Strong statistical machinery for CPU-bound component work
- Easy local profiling of deterministic functions

**Cons:**

- Does not measure process startup, filesystem-cache state, snapshot lifecycle, CLI
  output, Python embedding, or external competitors
- A new dependency would need supply-chain review and still would not solve
  orchestration

**Decision:** Reject as the primary system.
Focused in-process probes may be added later when they answer a measured component
question.

### Option C: Repository-Owned Scenario Runner and Probe

**Description:** Commit parametric recipes, a direct-argv state-machine runner, a small
non-production probe built from the existing fdu crate, comparator adapters, a versioned
JSON result schema, and a report renderer.

**Pros:**

- Models fdu’s cache and mutation transitions directly
- Keeps setup, timing, verification, provenance, and reporting in one contract
- Supports external CLI, Python, library phases, resources, and competitors
- Requires no third workspace crate or benchmark-only runtime dependency

**Cons:**

- More initial design than a shell timing loop
- Platform resource collectors and controlled cold-cache runs need explicit capability
  handling

**Decision:** Adopt.

## Recommendations

1. Implement Option C under `benchmarks/`, with generated corpora and results ignored
   but recipes, schemas, adapters, tests, and report templates committed.
2. Name every job and state explicitly; prohibit unqualified *cold*, *warm*, and *raw*
   in generated result records.
3. Share one corpus generator and manifest validator across the 500k revalidation spike,
   snapshot-format spike, packed-record memory gate, and final cross-tool report.
4. Make semantic validation a mandatory pre- and post-trial step.
5. Keep full-stat fdu performance, traversal-only probes, output modes, and comparator
   capabilities in separate rows.
6. Use shared CI only for harness correctness.
   Establish a stable scheduled runner before introducing numeric regression gates.
7. Generate Markdown reports from versioned raw records and require every README claim
   to link to a compatible committed report plus its reproduction manifest.
8. Profile only a stable, material regression or the dominant Phase 1 path; preserve the
   before/after profile and end-to-end evidence with the optimization.

## Next Steps

- [ ] Implement the corpus recipes, generator, observed manifest, and semantic oracle
- [ ] Implement the scenario runner, result schema, probe, collectors, and renderer
- [ ] Add dut and gdu adapters with an explicit capability matrix
- [ ] Execute the revalidation, snapshot, memory, concurrency, and full comparison gates
- [ ] Establish smoke, scheduled-regression, and release-evidence workflows

## Methodology

The Flowmark source files and completed plan documents were read at the pinned revision.
No Flowmark build, hook, script, submodule, or benchmark was executed.
Its approach was compared with fdu’s current public functions and binary behavior, the
active Phase 1 and Rust-quality plans, the completed CLI golden plan, and the original
file-roll-up research.
Existing fdu beads were searched before proposing a new work graph so that revalidation,
snapshot, packed-memory, concurrency, and final benchmark ownership remain with their
current issues.

This document does not validate a performance number.
It defines the evidence required before one can be validated.

## References

- [Flowmark performance report at the reviewed revision](https://github.com/jlevy/flowmark-rs/blob/015f23989af3e5cfb3f8b58dfc72822c534df25a/benchmarks/REPORT.md)
- [Flowmark corpus generator](https://github.com/jlevy/flowmark-rs/blob/015f23989af3e5cfb3f8b58dfc72822c534df25a/benchmarks/generate_corpus.sh)
- [Flowmark cross-tool runner](https://github.com/jlevy/flowmark-rs/blob/015f23989af3e5cfb3f8b58dfc72822c534df25a/benchmarks/run_comparison.sh)
- [Flowmark profiling runner](https://github.com/jlevy/flowmark-rs/blob/015f23989af3e5cfb3f8b58dfc72822c534df25a/benchmarks/profile_rust.sh)
- [Flowmark performance and profiling plan](https://github.com/jlevy/flowmark-rs/blob/015f23989af3e5cfb3f8b58dfc72822c534df25a/docs/project/specs/done/plan-2026-02-26-perf-comparison-profiling.md)
- [Flowmark parallel-processing plan](https://github.com/jlevy/flowmark-rs/blob/015f23989af3e5cfb3f8b58dfc72822c534df25a/docs/project/specs/done/plan-2026-02-27-parallel-file-processing.md)
- [Flowmark cache and performance roadmap](https://github.com/jlevy/flowmark-rs/blob/015f23989af3e5cfb3f8b58dfc72822c534df25a/docs/project/specs/done/plan-2026-02-27-incremental-cache-and-performance-roadmap.md)
- [fdu Phase 1 plan](../specs/active/plan-2026-08-08-fdu-phase-1.md)
- [fdu file-roll-up engine research](research-2026-08-06-file-rollup-engine.md)
- [fdu CLI golden-test plan](../specs/done/plan-2026-08-09-fdu-cli-golden-tests.md)
- [fdu Rust engineering quality plan](../specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
