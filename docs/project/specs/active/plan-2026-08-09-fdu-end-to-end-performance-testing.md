# Feature: End-to-End Performance Testing and Evidence

**Date:** 2026-08-09

**Author:** fdu project

**Status:** Active

## Overview

Build a reproducible performance-evidence system for fdu from corpus generation through
release claims. The system measures the actual CLI and installed Python binding, exposes
the engine phases needed to explain results, compares dut and gdu without pretending the
tools perform identical work, and preserves every trial as structured data from which
reports are generated.

This is the detailed design for the Phase 1 benchmark gate.
It turns the existing requirement—cold and warm, raw walk and full stats—into explicit
jobs, snapshot and filesystem-cache states, workload recipes, correctness oracles,
measurement rules, regression policy, and automation tiers.

The plan incorporates the useful parts of Flowmark’s performance work after reviewing
`jlevy/flowmark-rs` v0.3.2 at commit
[`015f23989af3e5cfb3f8b58dfc72822c534df25a`](https://github.com/jlevy/flowmark-rs/commit/015f23989af3e5cfb3f8b58dfc72822c534df25a).
The companion
[performance-evidence research](../../research/research-2026-08-09-end-to-end-performance-evidence.md)
records the source analysis and rejected alternatives.
This plan is independently actionable.

## Goals

- Define stable benchmark jobs for scan production, index construction, snapshot I/O,
  revalidation, CLI output, Python embedding, and delta application
- Generate deterministic workload families from compact committed recipes and validate
  the observed filesystem before and after every state transition
- Name snapshot state and operating-system filesystem-cache state independently
- Make correctness, completeness, exit status, output digest, and cache source mandatory
  preconditions for accepting a timing sample
- Record exact binaries, revisions, build settings, host class, filesystem, corpus,
  commands, collectors, and raw trials in a versioned machine-readable format
- Compare fdu, dut, and gdu through reviewed adapters and a capability matrix, with
  like-for-like and cross-capability results labeled differently
- Measure user-visible new-process latency externally and use component probes only to
  explain it
- Measure the Phase 1 scale, warm-revalidation, memory, snapshot-open, and contention
  targets without duplicating their existing beads
- Add non-flaky smoke, scheduled-regression, and release-evidence tiers
- Generate benchmark reports from raw artifacts and block unreferenced README claims

## Non-Goals

- Claim that the current `read_dir` + `symlink_metadata` scaffold is fast
- Put absolute timing thresholds on generic hosted pull-request runners
- Reduce retained metadata, skip correctness checks, weaken cache validation, or alter
  the output contract to improve a benchmark
- Treat deleting an fdu snapshot, starting a new process, or copying a corpus as proof
  that the operating-system filesystem cache is cold
- Benchmark every Cartesian product of scale, topology, metadata, state, output, and
  platform
- Commit generated large corpora, mutable result directories, profiles, or host-specific
  baselines
- Add Criterion, a third workspace crate, a production telemetry dependency, or a stable
  public performance API before a measured need justifies it
- Make watch-backend latency or content-tier metrics Phase 1 release gates; the probe
  schema leaves room for them later
- Replace the CLI golden suite, index reference model, snapshot corruption tests, or
  ordinary correctness gates with performance tests

## Current Status

The design and Flowmark synthesis are complete; planning bead `fdu-f0or` is closed.
The first two implementation children, `fdu-rq5m` and `fdu-d8kq`, are complete.
The standard-library-only corpus tool generates and independently verifies all eight
recipe families, applies deterministic local and distributed churn, and exposes
structured create, verify, mutate, and cleanup commands.
The strict evidence layer now validates versioned scenario and result contracts, creates
a deterministic paired schedule, executes every sample in a fresh marked run, records
pipe-drained first-output and completion timing, retains invalid samples, and renders a
deterministic report from immutable self-checking results.
The fdu probe and portable collector child `fdu-oj25` is complete.
The excluded Rust probe covers all eight component jobs in the committed smoke matrix.
Each invocation is checked against the exact per-run engine digest, and the result keeps
external wall time, internal component time, and per-child resource usage distinct.
The smoke suite also proves that deliberately wrong counts, digests, cache sources,
snapshot postconditions, and corrupt or absent snapshot inputs are rejected.
Claim-grade build/host manifests and intrusive Linux diagnostic collectors are separate
release-evidence children, `fdu-849g` and `fdu-bmhr`; neither blocks local scale spikes.

The repository-wide correctness, supply-chain, and concurrency implementation gates are
closed; final approval bead `fdu-sn43` is closed and PR #1 has merged.
The next measurement steps are the revalidation and snapshot cost-curve spikes under
`fdu-p2i1` and `fdu-1vd0`.
Comparator acquisition under `fdu-k5t5` has cleared its executable-dependency policy
blocker but now waits only on the reviewed adapter implementation shown below.
No current timing result supports a product claim.

The portable harness now has 57 deterministic and adversarial tests, passes Python
3.9 and the current interpreter, and is included in `make check` without a numeric
timing assertion. See the [performance harness README](../../../../benchmarks/README.md)
for its commands, manifest contract, mutation model, and cleanup rules.

The first exact-oracle revalidation curve measured 72.258 ms at 10k, 725.023 ms at
100k, 8.186 s at 500k, and 62.906 s at 1M on one uncontrolled local APFS host. It is an
exploratory design result, not a product claim, but it proves the current 500k target is
not met. A focused index fast path then improved every one of nine alternating 100k
pairs, with a -18.15% paired median change. The linked research note preserves the raw
samples and concurrency boundary.

The first exact probe run exposed and then verified the fix for a product correctness
defect: symlinks and special nodes contributed to regular-file roll-ups despite the
documented contract. Closed bead `fdu-6x07` owns that correction; no affected timing was
accepted. Closed bead `fdu-s23t` then removed redundant caller-owned absolute-path
construction on Unix with exact-oracle before/after evidence. Windows retains a fresh
non-following query because its cached directory-enumeration attributes are not a sound
fingerprint source; `fdu-k9zq` records that cross-platform correction.
The next Windows stage exposed an independent corpus portability defect: Python accepts
the `follow_symlinks` keyword for `os.utime()` but raises `NotImplementedError` where a
platform cannot honor `False`. `fdu-tqz1` centralizes capability-aware timestamp writes,
rejects symlinks before the fallback, and emulates the unsupported platform in tests.

## Background

### Current State

fdu already has the boundaries a useful performance system needs:

- `scan()` is a delta producer and can measure enumeration, stat collection, and
  observation batching without applying to an index.
- `scan_into_index()` adds parent-pointer insertion and hierarchical roll-ups.
- `snapshot::save()` and `snapshot::load()` isolate persistence work.
- `open()` reports `ColdScan` or `WarmRevalidate` and blocks until the returned index is
  trustworthy or explicitly partial.
- The CLI exposes bounded human output and complete schema-versioned JSON.
- The Python wheel exposes bulk open and query operations with the GIL released.
- The golden suite specifies the executable contract, and the active Rust-quality plan
  adds reference-model, fault-injection, toolchain, and feature-matrix gates before the
  Phase 1 representation changes.

The current walker is deliberately portable scaffolding.
The benchmark system may be built against it to validate the harness, but no speed
ranking or target pass is published until the syscall layer and optimized revalidation
land.

### Lessons Carried from Flowmark

The adopted Flowmark practices are deterministic corpus generation, separation of
discovery from full work, fresh versus steady and cached paths, pristine restoration for
mutating trials, repeated measurements with raw output, thread-scaling tests,
correctness before optimization, and profile-before-change discipline.

The fdu design strengthens those practices in five places:

1. a family of parametric corpora replaces a single replicated tree;
2. snapshot and filesystem-cache states are independent fields;
3. direct argument vectors replace shell command strings and `eval`;
4. a semantic oracle rejects incomplete or skipped work before timing is accepted;
5. generated reports replace hand-maintained result tables.

### Relationship to Existing Work

This plan does not take ownership away from existing Phase 1 tasks.
It supplies their common corpus, evidence schema, and runner.

| Existing bead | Evidence this plan supplies |
| --- | --- |
| `fdu-p2i1` | 10k-1M revalidation scale curve; unchanged, directory-shortcut, controlled-cold, and verified-warm filesystem states |
| `fdu-1vd0` | Compatible 500k snapshots; open, first-listing, and steady-listing scenarios |
| `fdu-1gbl` | Record/arena accounting and separate whole-process RSS on validated corpora |
| `fdu-r27g` | Repeated read/write contention workloads with query-latency distribution |
| `fdu-atqk` and `fdu-aky1` | Producer/full-index scan throughput, syscall/resource counters, worker and traversal-order scaling |
| `fdu-xihx` | Snapshot size, save/load cost, first-listing latency, and corrupt-fallback behavior |
| `fdu-wbis` | Unchanged and churned revalidation proportionality |
| `fdu-ywu0` | Final dut/gdu comparison and generated release report |
| `fdu-zga3` | Pinned compiler and supported feature contract required for comparable binaries |

## Design

### Approach

Commit a small performance system under `benchmarks/`:

```text
benchmarks/
├── README.md                  commands, host requirements, and claim policy
├── scenarios.json            named jobs, states, corpora, and acceptance class
├── corpora.json              parametric recipes and scale points
├── schema/
│   ├── scenario-v1.schema.json
│   ├── observed-corpus-v1.schema.json
│   └── result-v1.schema.json
├── generate.py               deterministic corpus creation and transition application
├── run.py                    direct-argv state-machine runner and collectors
├── report.py                 validation, statistics, comparison, and Markdown rendering
├── adapters/
│   ├── fdu.json
│   ├── dut.json
│   └── gdu.json
├── expected/                 small-corpus semantic expectations
└── reports/                  reviewed release reports and reproduction manifests

crates/fdu/examples/
└── perf_probe.rs             non-production component probe using existing public APIs
```

The exact split may consolidate the three Python entry points when implementation shows
that one typed module is simpler.
The contracts remain separate: generation changes filesystem state, running measures
commands, and reporting never executes a benchmark.

The orchestration scripts use the Python standard library and complete type annotations.
They pass argument arrays directly to subprocesses, allocate unique run directories, and
pass a minimal allowlisted environment instead of inheriting the developer shell, and
have no third-party runtime dependency.
The probe stays inside the existing `fdu` crate and is excluded from published package
contents. Timed modes print compact summaries; an untimed validation mode prints the
complete stable semantic digest.
It does not expose benchmark-only methods in the supported library API.

### Terminology and State Model

Generated data and reports must not use unqualified *cold*, *warm*, or *raw*.

**Snapshot state:**

- `absent` — no snapshot exists for the root
- `compatible-unchanged` — snapshot matches root/scope/fingerprint and the tree has not
  changed
- `compatible-changed` — compatible snapshot followed by a declared mutation recipe
- `corrupt` — snapshot bytes are deliberately invalid
- `incompatible-scope` — snapshot has another semantic scan scope
- `incompatible-engine` — snapshot has another format/engine fingerprint

**Filesystem-cache state:**

- `uncontrolled` — normal local or hosted-runner state; useful for development, not a
  cold-cache claim
- `verified-warm` — the exact setup job ran immediately before timing and its
  postcondition was checked
- `controlled-cold` — a dedicated host performed a documented eviction protocol and the
  collector evidence supports it

**Process state:**

- `new-process` for every CLI and Python end-to-end trial
- `same-process` only for an explicitly named component probe

The scenario id includes the job, corpus, snapshot state, filesystem-cache state, and
output sink. For example:

```text
scan-index/wide-500k/snapshot-absent/fs-verified-warm/output-digest
cli-json/mixed-100k/snapshot-compatible-unchanged/fs-uncontrolled/output-file
```

### Benchmark Jobs

| Job id | Timed boundary | Required postcondition | Primary use |
| --- | --- | --- | --- |
| `scan-producer` | `scan()` through a streaming count/aggregate summary | Entry/dir counts and aggregate summary match | Walker and syscall layer |
| `scan-index` | `scan_into_index()` through O(1) index totals | Length and root totals match | Full stat-tier scan |
| `snapshot-save` | `snapshot::save()` including atomic replace | Reload succeeds; size and digest match | Writer and format |
| `snapshot-load` | `snapshot::load()` through available index/listing | Index digest matches | Load/open and lazy format |
| `revalidate-unchanged` | Compatible load plus reconciliation | Zero semantic change; result fresh/complete | Compatible-snapshot unchanged path |
| `revalidate-churn` | Compatible load plus declared mutation/reconciliation | Exact changed digest and counts | Incremental proportionality |
| `cli-human` | Process spawn through EOF | Exit, selected source, and output digest match | Human product path |
| `cli-json` | Process spawn through EOF and JSON parse | Schema, complete/fresh state, and digest match | Agent product path |
| `python-open-query` | Installed-wheel process spawn through bulk query | Result digest and cache path match | Python product path |
| `delta-apply` | Deterministic in-memory observation stream | Reference digest after every phase | Watch/reconcile explanation |
| `concurrent-query` | Reconciler stream with fixed query schedule | All queries valid; latency samples complete | `IndexHandle` contention |

`scan-producer` is the precise replacement for *raw walk*: it includes every operation
the producer contract requires, including stat fields and observation construction, but
not index application or rendering.
A lower-level getdents-only microprobe may explain syscall work but cannot replace this
gate.

Exact normalized-record hashing and sorting happen in untimed validation runs.
Putting them inside `scan-producer` or `scan-index` would measure the oracle as much as
the engine. Each timed trial instead returns a compact summary that catches skipped,
partial, duplicated, or wrong-size work at negligible fixed per-entry cost; the scenario
is accepted only when the exact validation run for the same binary, arguments, corpus,
and state also passes.
Complete CLI output is still hashed inside the end-to-end job because producing and
draining those bytes is part of that job.

CLI output has two sinks:

- `output-file` includes real writes and hashes the completed artifact;
- `output-digest` drains through a pipe and hashes bytes without terminal rendering.

`/dev/null` alone is insufficient because it cannot prove that complete output was
produced. Bounded human output and complete JSON remain different jobs.

### Workload Recipes

The committed recipe is declarative and parametric.
It contains a schema version, seed, target counts, directory fan-out and depth,
name/extension distributions, size/allocation classes, link rules, timestamps, and
expected platform capabilities.
Generation is idempotent only after an explicit run-directory reservation; it never
deletes an unresolved or shared path.

Create, verify, mutate, and cleanup serialize through one atomic per-run operation lock.
Concurrent or stale-lock access fails closed instead of racing a state transition.
The portable deep topology uses short path segments and a bounded directory depth so its
committed recipe remains usable on narrower path-budget platforms.

The first comprehensive matrix is:

| Recipe family | Scale points | Shape and purpose |
| --- | --- | --- |
| `contract` | tens | Exact cross-platform semantic oracle; kinds, sizes, extensions, links where supported |
| `wide` | 10k, 100k, 500k, 1M | Many siblings; directory listing, maps, sorting, and non-invertible reducer pressure |
| `deep` | 10k, 100k | Long but platform-safe chains mixed with leaves; parent reconstruction and stack safety |
| `balanced` | 10k, 100k, 500k, 1M | Fixed fan-out over several levels; general throughput and scale target |
| `mixed-metadata` | 100k, 500k | Tiny, mixed, and sparse files; hardlink groups and symlinks; apparent/allocated semantics |
| `churn-local` | 100k, 500k | One directory receives one change, then 1% modify, then 1% add/remove/rename |
| `churn-distributed` | 100k, 500k | Same change volume spread across directories; shortcut and queue behavior |
| `partial` | small, platform-specific | Unreadable or racing subtrees; resilience and completeness, never ranking |

The smoke tier uses `contract` and a 1k form of `balanced` that is too small for a speed
claim. The scheduled tier uses 100k. The full Phase 1 evidence includes 10k, 100k, 500k,
and 1M where host time and inode capacity allow it.

File content is minimal because Phase 1 is stat-only.
Sparse allocation is used only where its semantics are verified; the generator never
consumes hundreds of gigabytes to simulate a size field.
Platform-unsupported link or permission cases are marked absent in the observed manifest
rather than silently substituted.

### Observed Corpus Manifest

Generation ends by walking the corpus with a simple independent verifier and writing an
observed manifest. It contains:

- recipe schema/version, id, seed, and canonical recipe hash;
- generator revision and platform capability decisions;
- file, directory, symlink, other-kind, hardlink-group, and total-entry counts;
- maximum depth and directory fan-out summary;
- apparent and allocated totals where the platform can establish them;
- extension counts and byte totals for the contract corpus;
- normalized record digest for every corpus small enough to enumerate into the oracle;
- mutation-state id and the expected digest after each declared transition.

The manifest also records a generator source hash and constant-memory
`sha256-multiset-v1` components.
Semantic records use normalized relative paths and nanosecond mtimes but omit inode and
ctime so equivalent regeneration remains portable.
The eventual probe records per-run fingerprint identity separately where a job needs
those non-portable fields.

The result record references the manifest hash.
A trial whose precondition or postcondition no longer matches is invalid and is never
included in statistics.

### Correctness Oracle

The oracle is intentionally independent of fdu’s output implementation.

1. `contract` has committed exact semantic expectations.
2. Generated large recipes derive totals and a normalized record digest from creation
   operations, then cross-check them with the independent verifier.
3. The fdu probe’s untimed validation mode digests normalized path units, kind,
   fingerprint fields, and retained roll-ups in a stable order.
   Timed component modes return the compact summaries defined by their job contract.
4. `cli-json` parses the complete document and derives the comparable semantic digest;
   the tryscript suite remains authoritative for exact text.
5. Each mutation recipe produces an exact next manifest and verifies that the
   compatible-snapshot path reports the expected source and final truth.
6. Snapshot corruption and incompatibility must take the documented fallback path and
   may not return cached data.
7. Comparator adapters prove at least the root, visited count or parsed output total,
   exit status, and selected size semantics.
   Missing observability is recorded as a capability limit.

No duration is accepted unless its oracle passes.
The runner writes invalid trials with their failure reason so harness defects and tool
failures remain visible.

### Runner State Machine

Each scenario expands into explicit steps:

```text
reserve unique run directory
→ build/select exact binary
→ choose and record randomized invocation order
→ for each declared warmup and timed invocation:
    restore or generate the corpus
    → verify the exact pre-trial manifest
    → establish snapshot state
    → establish and record filesystem-cache state
    → run one command with its compact timed postcondition
    → verify exact output and filesystem postconditions outside the timer
    → append the valid or invalid trial record
→ write immutable raw result
→ release run directory according to retention policy
```

Mutating scenarios restore from the recipe or use a copy/reflink strategy whose time is
outside the measurement.
Snapshot and output paths are siblings of the corpus, never inside the scanned root.
Every destructive cleanup resolves and verifies its unique run-directory marker before
removing data.

State preparation is repeated for every invocation, including warmups.
Controlled-cold eviction occurs after corpus and snapshot validation, immediately before
the timed command.
Verified-warm preparation runs the exact declared warming operation at
that same point. No timed trial inherits a mutated corpus, snapshot, or cache state from
the preceding tool or sample.

The runner accepts an explicit scenario allowlist, output directory, trial count, and
host capability profile.
Unknown fields fail closed.
Timeouts kill the whole child process group and produce an invalid trial; they do not
convert a hang into a slow valid sample.

Each adapter declares the environment variables its tool needs.
The runner sets locale, timezone, cache roots, thread controls, and output controls
explicitly, removes unrelated inherited variables, and records the normalized allowlist.
Absolute run-root values are tokenized in persisted results.

### Result Schema and Artifacts

One immutable JSON document records one run set.
The schema contains:

```text
identity       schema, run id, UTC time, source revision, dirty state
build          profile, features, target, rustc/cargo, binary checksum
host_class     OS, kernel, arch, CPU, logical count, memory, filesystem
corpus         recipe id/seed/hash, observed-manifest hash
scenario       job, all states, scope, output sink, adapter, argv
environment    normalized allowlisted variables and explicitly unset controls
method         warmups, trials, order seed, timeout, collectors
trials[]       order, wall/user/system time, RSS, faults, I/O, exit, digests
validation     precondition, per-trial postcondition, invalid-trial reasons
```

Hostnames, usernames, environment dumps, personal absolute paths, and unrelated process
lists are forbidden.
Paths in commands are replaced with run-root tokens in persisted metadata while the
exact resolved commands remain in the ephemeral log.

Generated corpora, current results, profiles, and run directories remain ignored.
A reviewed release report commits:

- the Markdown report;
- the compact raw result sets that support its tables;
- the reproduction manifest and result-schema version;
- adapter revisions and binary checksums;
- profiles only when they materially support a finding and are reasonably sized.

Release evidence requires a clean source revision, locked dependency resolution, and a
binary built once before the measured set and reused for every trial.
Exploratory runs may record a dirty source state, but they cannot support a README or
release claim.

### Metrics and Collectors

Required everywhere:

- external monotonic wall time;
- exit status and signal;
- output byte count and digest;
- entry/directory count and semantic digest;
- first-output and completion timestamps where output is streamed.

Required on the dedicated Linux evidence host:

- user and system CPU;
- peak RSS;
- major and minor page faults;
- bytes read and written where the kernel exposes them;
- context switches;
- syscall counts for selected diagnosis runs.

Optional diagnosis:

- hardware counters through `perf stat`;
- sampled CPU profiles and flamegraphs;
- allocation profiles;
- platform-native Instruments or Windows Performance Recorder captures;
- fdu phase events from the non-production probe.

Collector support is capability-negotiated.
Missing values are `null` with a reason.
A collector failure invalidates only scenarios that require it; the runner never records
an unavailable counter as zero.

### Comparator Adapters

Adapters are data plus a small parser, not shell snippets.
Each one pins:

- source/release revision, acquisition method, license, and binary checksum;
- build command and release settings outside the measured region;
- exact direct argument vector for each supported job;
- symlink, mount-boundary, hidden/ignored, hardlink, error, and size semantics;
- output handling and parser/postcondition;
- minimal environment and explicitly disabled implicit configuration;
- supported operating systems and filesystem assumptions.

Third-party source acquisition follows the tbd read-only checkout workflow.
Repository instructions, hooks, submodules, build scripts, and dependency inputs are
inspected before any comparator code is built or run; revisions and resulting binary
checksums are then frozen in the adapter.
The GPL dut binary is a benchmark input, never linked into or distributed with fdu.

The initial capability report includes:

| Capability | fdu | dut | gdu |
| --- | --- | --- | --- |
| Full retained inventory | yes | adapter records exact limitation | adapter records exact limitation |
| Apparent and allocated bytes in one run | yes | verify at pinned revision | yes |
| Counts and newest mtime | yes | verify at pinned revision | verify at pinned revision |
| Per-directory extension tallies | yes | no | verify/record |
| Persistent snapshot and revalidation | yes | no | persistence is not equivalent to fdu revalidation |
| Machine output sufficient for oracle | yes | verify/record | verify/record |

The table is completed from the pinned source revisions before the first comparison.
No unavailable comparator is silently skipped in a release run.

### Trial Scheduling and Statistics

Setup and validation are untimed.
The runner rotates or randomizes paired tool order with a recorded seed.
Each release headline uses at least ten valid timed trials and enough aggregate measured
duration to dominate timer resolution and startup noise.

The generated report shows:

- every raw trial;
- median and median absolute deviation;
- p95 when the sample count supports it;
- minimum, maximum, mean, standard deviation, and coefficient of variation;
- entries per second and relevant bytes per second;
- paired median ratio for compatible cross-tool runs;
- peak/retained RSS and distinct record/arena accounting for memory scenarios.

P95 is omitted below 20 valid trials rather than estimated from an undersized sample.
Coefficient of variation above 10% is an investigation trigger.
It is not an automatic retry and not an outlier filter.
Invalid samples remain in the artifact with a mechanical reason.
Correcting a host or harness problem reruns the whole declared scenario set; it does not
retain only favorable old trials.

Numeric regression thresholds are scenario-specific and are established from the first
stable scheduled baseline.
Each needs:

- a practical effect size larger than ordinary noise;
- compatible job, schema, corpus, host class, build, and collector versions;
- enough paired evidence to distinguish a material regression from variation;
- an explicit override record when maintainers accept a trade-off.

### Phase 1 Performance Gates

The final evidence report must answer all of these independently:

1. **Controlled-cold product scan:** on the dedicated Linux host and same validated
   corpus, the median new-process full-stat `scan-index` probe is within roughly 1.5x of
   dut’s new-process configured comparison job.
   Both minimize presentation work, validate a result, and show the capability
   difference beside the ratio.
2. **Compatible-snapshot revalidation:** compatible unchanged snapshot load plus
   complete revalidation is well under one second at 500k entries, with snapshot and
   filesystem-cache states named.
   The actual cached CLI path, including any snapshot rewrite and bounded output, is
   reported separately and must meet the same product target before *near-instant*
   language is used.
3. **Memory:** layout or arena accounting under `fdu-1gbl` demonstrates the 25-32-byte
   regular-file record target.
   Peak and retained RSS report whole-process cost separately, including names,
   allocator overhead, directory records, reducers, and bounded diagnostics; RSS is
   never mislabeled as struct size.
4. **Snapshot UX:** report snapshot size, save time, open time, time to first directory
   listing, and steady listing latency; do not substitute decode throughput for open.
5. **Scale:** show the 10k, 100k, 500k, and 1M curves rather than extrapolating from one
   point.
6. **State sensitivity:** report snapshot absent, unchanged, representative churn,
   corrupt fallback, and incompatible-scope paths.
7. **Output cost:** distinguish engine-only, bounded human, and complete JSON results.
8. **Parallelism:** show one, two, four, eight, and selected/default worker settings
   where configurable; retain a slower result rather than assuming monotonic scaling.
9. **Correctness:** every accepted trial passes its oracle and the normal handoff gate.
10. **Claims:** README language is generated or copied from the reviewed report and
    links to its reproduction manifest.

### Automation Tiers

#### Pull-Request Smoke

The smoke path belongs in `make check` only after its runtime is demonstrated to be
small and stable. It validates:

- recipe/schema parsing and unknown-field rejection;
- deterministic generation of the contract and 1k balanced corpora;
- oracle and mutation transitions;
- probe build and result-schema validation;
- one invocation per fdu job with no numeric speed assertion;
- report rendering from a committed synthetic result fixture.

Hosted CI may use generous catastrophic timeouts, but their values are not baselines.

#### Scheduled Regression

A protected, documented runner class executes the 100k key scenarios at a fixed cadence
and on explicit performance-sensitive pull requests.
It compares only compatible baselines, uploads raw artifacts, opens or updates a
tracking issue on a material regression, and never rewrites the baseline automatically.

#### Release Evidence

A maintainer-triggered run on a dedicated host executes the full scale/state/comparator
matrix, controlled cold-cache protocol, profiles selected dominant paths, and renders a
reviewable report. Publishing and README performance claims depend on approval of this
artifact, not merely on workflow success.

### Profiling and Optimization Loop

Profiles are triggered by a stable material regression, a failed Phase 1 target, or a
dominant phase identified by the component matrix.

Every optimization bead records:

1. compatible before result and profile;
2. named bottleneck and causal hypothesis;
3. red correctness or regression test where applicable;
4. focused implementation with no semantic reduction;
5. normal `make check` result;
6. compatible after result and profile;
7. complexity and memory trade-off.

An instruction-count or syscall reduction without end-to-end improvement is useful
diagnosis but not a product win.
Conversely, a wall-time change without a causal profile is not enough to justify a
complex architecture change.

### API and Packaging Changes

No stable library or CLI API change is required to establish the first harness.

- The probe uses existing public `scan`, index, snapshot, open, and query boundaries.
- First-output timing is measured by the parent process reading stdout.
- Additional stage events, if later necessary, begin as an unstable probe-only sink or
  crate-internal helper, not a hidden production environment variable.
- `perf_probe.rs` and benchmark scripts are excluded from the published crate and wheel.
- Any new external tool or dependency must pass the 14-day cool-off and supply-chain
  policy before adoption.
  Optional system collectors are not build dependencies.

If lazy snapshot access makes first-listing impossible to observe through the public
surface, the snapshot-format bead may add the smallest supported query API needed by a
real consumer. Benchmarking alone is not a reason to stabilize an abstraction.

## Implementation Plan

### Phase 1: Establish the Evidence Contract

- [x] `fdu-rq5m`: implement deterministic contract, scale, topology, metadata, and churn
  recipes in unique safe run directories
- [x] `fdu-rq5m`: implement the independent observed-manifest verifier, mutation
  transitions, and semantic digest
- [x] `fdu-d8kq`: commit strict scenario, corpus-manifest, and result schemas with
  valid, unknown-field, truncated, and incompatible-version fixtures
- [x] `fdu-oj25`: implement probe modes that detect deliberately wrong counts, digests,
  cache sources, and snapshot postconditions
- [x] `fdu-d8kq`: document terminology, host capabilities, safe cleanup, and why smoke
  results support no performance claim

### Phase 2: Measure the Engine and Comparators

- [x] `fdu-d8kq`: implement the direct-argv state machine, timeouts, process cleanup,
  paired ordering, immutable results, validation, statistics, and report rendering
- [x] `fdu-oj25`: implement external timing plus capability-negotiated portable resource
  collectors
- [ ] `fdu-bmhr`: add opt-in dedicated-host profile and byte-I/O/syscall collectors
- [ ] `fdu-849g`: pin strict claim-grade build and anonymous host manifests
- [ ] `fdu-k5t5`: complete reviewed dut/gdu adapters and the job-capability matrix
- [ ] `fdu-p2i1` and `fdu-1vd0`: execute the revalidation and snapshot-candidate spikes
  before freezing their Phase 1 designs
- [ ] `fdu-6wu0`: establish repeated large trials from safely cloned, independently
  verified base corpora instead of regenerating 500k-1M entries for every invocation
- [ ] `fdu-ywu0`: add memory, scale, thread-count, traversal-order, output, Python, and
  contention scenarios as their engine surfaces become available
- [ ] `fdu-atqk`, `fdu-aky1`, `fdu-1gbl`, `fdu-a6dz`, `fdu-xihx`, and `fdu-wbis`:
  profile failed targets and land optimizations only with before/after correctness and
  end-to-end evidence

### Phase 3: Govern Regressions and Claims

- [ ] `fdu-8z5l`: add the tiny harness correctness suite to the handoff gate after
  measuring its cost
- [ ] `fdu-8z5l`: establish a protected scheduled runner, compatible baselines, noise
  policy, artifact retention, and regression triage
- [ ] `fdu-ywu0`: run the dedicated-host Phase 1 matrix and generate the reviewed report
- [ ] `fdu-ywu0`: update README claims only from that report and link its reproduction
  manifest
- [ ] `fdu-9cf0`: require the reviewed report for publishing without treating shared CI
  timing as a release oracle

## Testing Strategy

The benchmark system is tested like production code because a harness bug can create a
false engineering direction.

- Golden fixtures cover recipe, corpus-manifest, raw-result, and generated-report
  schemas.
- Generator tests prove same recipe/seed produces the same normalized manifest and that
  another seed changes the intended fields.
- Destructive-cleanup tests use only marked temporary directories and reject root,
  workspace, unresolved, symlinked, and missing-marker targets.
- State-machine tests use fake executables to cover success, nonzero exit, signal,
  timeout, partial output, invalid JSON, wrong digest, and process-child cleanup.
- Environment tests prove an unrelated sentinel variable cannot leak into a child and
  that every declared variable is normalized in the persisted result.
- Mutation tests prove every transition updates only the declared paths and yields the
  expected next manifest.
- Collector tests parse committed platform samples and distinguish unsupported,
  unavailable, permission-denied, malformed, and zero values.
- Statistical tests use fixed raw samples to lock medians, MAD, p95 eligibility,
  variation warnings, paired ratios, incompatible-baseline rejection, and invalid-trial
  handling.
- Adapter tests assert exact argument arrays and parse representative tool outputs; no
  release adapter may silently skip a missing binary or unsupported flag.
- The contract corpus is cross-checked with the existing focused Rust and CLI golden
  tests.
- A deliberate no-op or narrowed fdu probe must fail the oracle even if it is faster.
- Smoke tests run on Linux, macOS, and Windows where their declared capabilities exist;
  controlled-cold and comparator claims remain on the dedicated supported host.

The implemented corpus suite additionally locks root-inclusive counts, the committed
contract fixture, seed determinism, all recipe families, 1k bounded-manifest behavior,
subsecond mtime detection, ordered local and distributed transitions, operation-path
declarations, precondition tampering, manifest hashes and shapes, JSON size limits,
atomic operation exclusion, failed-lock release, safe cleanup, and the Python 3.9
surface. It remains a correctness gate, not a performance baseline.

The implemented evidence suite locks unknown and incompatible schemas, non-finite JSON,
schedule reconstruction, immutable hashes, minimal environments, bounded pipe capture,
pipe-versus-file output semantics, first-output timing, process-group timeouts,
per-sample snapshot and cache preparation, churn ordering, corpus postconditions,
invalid-sample retention, baseline compatibility, all declared statistics, review
triggers, structured CLI errors, and byte-for-byte report regeneration.

`make check` remains the handoff gate for code changes.
Full benchmarks run separately because creating one million filesystem entries is not a
reasonable correctness prerequisite for every commit.

## Rollout Plan

1. Land schemas, recipes, generator, oracle, and smoke fixtures without publishing a
   timing number.
2. Land the runner and probe; exercise them against the portable scaffold only to find
   harness defects.
3. Use the common system for the foundational revalidation and snapshot spikes.
4. Let the syscall, parallelism, packed-record, revalidation, and block-snapshot beads
   consume component results while their APIs are still pre-release.
5. Establish the stable scheduled runner and collect enough history to set
   scenario-specific regression thresholds.
6. Execute and review the complete Phase 1 report after the implementation blockers
   land.
7. Only then update README performance language and allow publishing to rely on it.

Old raw results remain readable by their schema-specific renderer or are migrated by an
explicit checked converter.
A schema change never silently compares incompatible baselines.

## Acceptance Criteria

The performance-testing workstream is complete when:

1. a clean checkout can generate and validate every declared corpus from committed
   recipes without network access;
2. every accepted sample records and passes semantic, output, exit, cache-source, and
   filesystem-state postconditions;
3. raw results include all required provenance and regenerate the committed report
   byte-for-byte;
4. snapshot state and filesystem-cache state are explicit in every scenario and table;
5. fdu, dut, and gdu adapters are pinned, reviewed, non-skipping, and accompanied by the
   completed capability matrix;
6. the 500k revalidation, snapshot UX, 25-32-byte record, scale, output, and contention
   evidence is linked to the beads that own those decisions;
7. pull-request smoke tests prove the harness without imposing a numeric speed gate;
8. scheduled regressions use a stable runner and reject incompatible baselines;
9. the full Phase 1 report satisfies the ten performance gates above or records an
   explicit design decision to revise a target;
10. every README performance claim links to that report and reproduction manifest.

## Open Questions

- **Dedicated Linux host (`fdu-8z5l`, `fdu-ywu0`)**: which runner and filesystem become
  the first benchmark host class?
  The result schema and local harness do not depend on the answer, but numeric baselines
  do.
- **Controlled-cold protocol (`fdu-8z5l`, `fdu-ywu0`)**: privileged cache eviction,
  disposable VM/block device, or a corpus larger than available cache?
  The full report must document the chosen method and its limits.
- **Comparator postconditions (`fdu-k5t5`)**: can dut and gdu expose strong enough
  machine-readable evidence, or must adapters add external traversal/count validation?
- **Snapshot first-listing surface (`fdu-1vd0`, `fdu-xihx`)**: does the optimized
  snapshot expose this through a real query, or is a narrow API needed?
- **Dedicated-host collectors (`fdu-oj25`, `fdu-8z5l`)**: which pinned `perf` and
  byte-I/O protocol becomes the Linux release collector?
  Portable POSIX runs use per-child `wait4` for CPU, peak RSS, faults, block operations,
  and context switches. Byte I/O, retained RSS, and syscall counts remain explicitly
  unavailable until the dedicated-host protocol supplies them; Windows smoke records
  the whole rusage set as unavailable rather than inventing zeros.

None of these questions blocks Phase 1 of the harness.
They are settled before the first corresponding numeric baseline or comparator claim.

## Design Review Ledger

These findings were identified and resolved while reviewing the plan.
The planning bead retains the same stable IDs.

| ID | Severity | Finding | Resolution |
| --- | --- | --- | --- |
| PEV-01 | High | Sorting or hashing every record inside a component timer would measure the oracle as much as the engine | Exact semantic validation is untimed; timed probes emit only compact job postconditions, while complete CLI output remains part of its end-to-end job |
| PEV-02 | High | Establishing corpus, snapshot, or filesystem-cache state once per run set lets later samples inherit earlier tool state | Every warmup and timed invocation re-establishes and records its declared state immediately before timing |
| PEV-03 | Medium | Inheriting a developer shell makes locale, cache, thread, allocator, or tool configuration an invisible benchmark input | The runner passes and records a minimal normalized environment and strips unrelated variables |
| PEV-04 | Medium | Peak RSS cannot by itself prove a 25-32-byte record layout | Record/arena accounting owns the layout gate; peak and retained RSS report whole-process memory separately |
| PEV-05 | Low | TOML recipes would require a parser not present in the Python standard library on every supported interpreter | Scenarios, recipes, schemas, and adapters use strict versioned JSON |
| PEV-06 | High | Whole-second corpus mtimes would miss changes that alter fdu’s nanosecond cache fingerprint | Semantic records and deterministic mutation timestamps retain nanoseconds; a subsecond-only tamper test must fail verification |
| PEV-07 | High | Concurrent mutate, verify, or cleanup processes could race one run’s manifest and filesystem state | Every stateful operation acquires the same atomic run-directory lock; active and abandoned locks fail closed |
| PEV-08 | Medium | Add, remove, and rename transitions could accidentally change corpus size, shape, extension mix, or fan-out and confound revalidation evidence | Every transition preserves and rechecks those aggregate invariants while chaining exact changed paths and semantic components |
| PEV-09 | Medium | Long hashed directory segments made the nominally platform-safe deep corpus exceed narrower path budgets | Deep recipes use compact segments under an explicit tested relative-path budget while other seeded fields retain recipe variation |
| PEV-10 | High | The first runner draft called a temporary stdout file `output-digest`, so it measured filesystem writes and hashed bytes after the timer | `output-digest` now drains and hashes a pipe during the timed job with only bounded compact-JSON retention; `output-file` alone measures real writes and hashes the completed file afterward |
| PEV-11 | High | Requiring identical executable checksums would reject every regression comparison whose subject binary legitimately changed | Compatibility now requires the same adapter command shape while recording checksum changes separately; harness, host, corpus, scenario, and collector contracts must still match |
| PEV-12 | High | A self-consistent result hash did not prove that the stored invocation list contained every declared warmup and trial in the seeded order | Result validation reconstructs the complete schedule and cross-checks every trial against its scenario, corpus, environment, state, process, output, and timing contract |
| PEV-13 | High | Any successful preparation command could label a portable run `controlled-cold` without evidence that the operating-system cache was evicted | The portable runner rejects that label; only the dedicated-host protocol under `fdu-8z5l` may authorize it, while local preparation can establish `verified-warm` |
| PEV-14 | Medium | Binary hashes alone omitted the exact Python and harness implementation that generated timings and validity decisions | Results now record Python identity plus hashes of the corpus, runner, schema, and report implementations, and compatibility requires an exact harness match |
| PEV-15 | High | Process-wide cumulative `getrusage(RUSAGE_CHILDREN)` counters would attribute earlier setup and trials to later samples | A single owner thread reaps each POSIX child with `wait4`; every resource value is per invocation, while unsupported platforms and metrics retain exact null reasons |
| PEV-16 | High | Reporting only process wall time for component probes hid the target operation behind setup, exact validation, and JSON emission | Results and reports preserve external wall and explicit internal component duration separately; only product jobs may use external latency as the user-facing number |
| PEV-17 | High | The exact contract corpus showed that retained symlinks and special nodes incremented regular-file counts and bytes despite the `RollUp` contract | `fdu-6x07` splits non-file contributions from regular-file roll-ups and adds kind-transition regression coverage before any sample is accepted |
| PEV-18 | Medium | A portable semantic digest omits inode, ctime, device, and allocated size, so it cannot prove the exact fingerprint-sensitive index scanned in one trial | Manifests now carry a second per-run engine digest over pinned binary records; probe output must match it while cross-run compatibility continues to use the portable semantic digest |
| PEV-19 | Low | Increasing observation batches looked like an easy way to reduce index and reconciliation overhead | A six-point 10k sweep from 64 through 65,536 ops showed no stable improvement, so the 1,024 default remains unchanged |
| PEV-20 | Medium | The Unix portable walker constructed an absolute path for every successful metadata lookup even though the equivalent `DirEntry` API was available | Unix `DirEntry::metadata()` preserves non-following semantics and improved alternating same-corpus 100k paired medians by 6.84-8.24% across producer, full-index, and revalidation jobs; the focused evidence and limits are recorded in the linked research note |
| PEV-21 | High | “Unchanged directory mtime skips re-listing” can be misread as permission to trust a whole subtree, which misses in-place file edits because they do not change the parent directory mtime | A matching cached directory fingerprint may skip only `read_dir` name-set discovery; revalidation must still stat every known child and recurse into known directories, while a changed directory fingerprint triggers re-listing for membership changes |
| PEV-22 | Medium | Known-child expectation capture reconstructed each path and performed repeated root-to-leaf lookups; exclusive unchanged reconciliation then allocated and arbitrated guaranteed no-op upserts | Capture present-child state and identity directly from coherent child iteration, and elide exact no-op upserts only for `&mut Index`; nine exact-oracle 100k pairs improved by a paired median 18.15%, while shared ABA arbitration remains unchanged |
| PEV-23 | Medium | The first 1M invocation spent more than twelve minutes in serial Python corpus setup before any probe child launched, making fresh generation per sample impractical for scheduled evidence | `fdu-6wu0` adds a keyed immutable base-corpus cache with capability-proven clone/reflink and safe copy fallback; trials still verify their own exact fingerprint-sensitive precondition and never hardlink mutable corpus files |
| PEV-24 | High | Windows `DirEntry::metadata()` reuses directory-enumeration attributes that the platform permits to be non-current, producing a spurious warm-path update in both native and wheel CI | `fdu-k9zq` retains the measured `DirEntry` path on Unix, performs a fresh non-following stat on non-Unix platforms, and locks the boundary down with mutation-after-enumeration coverage |
| PEV-25 | High | Python exposes `os.utime(..., follow_symlinks=False)` on Windows but raises `NotImplementedError` because that operation is unavailable, so every generated performance trial failed during setup | `fdu-tqz1` checks `os.supports_follow_symlinks`, keeps non-following writes where available, verifies that fallback targets are not symlinks, and tests the unsupported-capability path on every development platform |

## Beads

Epic: **fdu-d5e1** — Build reproducible end-to-end performance evidence for fdu.
It is a child of the Phase 1 epic `fdu-qfz6` and depends on final PR approval under
`fdu-sn43`.

Planning record: **fdu-f0or** — Synthesize the pinned Flowmark lessons, write the
research and plan, assemble this graph, and validate it through CI.

| Bead | Priority | Work | Direct blockers |
| --- | --- | --- | --- |
| `fdu-rq5m` | P1 | Deterministic corpus recipes, safe generator, observed manifests, mutation transitions, and semantic oracle | `fdu-sn43` |
| `fdu-d8kq` | P1 | Strict scenario/result schemas, direct-argv runner, immutable trials, statistics, and report renderer | `fdu-rq5m` |
| `fdu-oj25` | P1 | fdu component probe, first-output timing, and portable per-child resource collectors | `fdu-rq5m`, `fdu-d8kq` |
| `fdu-6x07` | P1 | Exclude symlinks and special nodes from documented regular-file roll-ups | discovered by `fdu-oj25` |
| `fdu-s23t` | P1 | Use `DirEntry` metadata on Unix, with paired exact-oracle evidence | discovered by `fdu-oj25` |
| `fdu-k9zq` | P0 | Keep Unix metadata speedups without trusting cached Windows enumeration attributes | discovered by cross-platform CI |
| `fdu-tqz1` | P0 | Set deterministic corpus timestamps without unsupported Windows flags | discovered by cross-platform CI |
| `fdu-pkyu` | P1 | Elide redundant path lookups and guaranteed no-op applies during reconciliation | discovered by `fdu-p2i1` |
| `fdu-6wu0` | P1 | Reuse safely cloned and independently verified base corpora for large repeated trials | discovered by `fdu-p2i1` |
| `fdu-849g` | P1 | Strict claim-grade build and anonymous host provenance manifests | `fdu-oj25` |
| `fdu-bmhr` | P2 | Opt-in dedicated Linux byte-I/O, syscall, perf-stat, and profile collectors | `fdu-oj25` |
| `fdu-k5t5` | P1 | Pinned dut/gdu adapters, parsers, postconditions, and capability matrix | `fdu-rq5m`, `fdu-d8kq`, `fdu-ad45` |
| `fdu-8z5l` | P2 | Pull-request smoke, stable scheduled baselines, regression triage, artifact retention, and claim governance | `fdu-d8kq`, `fdu-k5t5`, `fdu-zga3`, `fdu-849g`, `fdu-bmhr`, `fdu-6wu0` |
| `fdu-ywu0` | P1 | Execute the complete Phase 1 matrix and publish the generated evidence report | all implementation/proof beads plus the existing engine blockers |

Cross-workstream dependencies make the existing decision beads consume the common
infrastructure:

- Closed beads `fdu-rq5m`, `fdu-d8kq`, and `fdu-oj25` supply the common foundation for
  the now-unblocked 500k revalidation spike (`fdu-p2i1`), snapshot candidate spike
  (`fdu-1vd0`), packed-record memory gate (`fdu-1gbl`), and concurrency measurement
  (`fdu-r27g`).
- `fdu-849g` and `fdu-bmhr` block stable release evidence and claims, not exploratory
  local engine decisions.
- The comparator adapters and scheduled numeric baselines have cleared the completed
  executable-dependency (`fdu-ad45`) and pinned-toolchain (`fdu-zga3`) prerequisites;
  their remaining graph edges still apply.
- The syscall walk (`fdu-atqk`), parallel scheduler (`fdu-aky1`), reducer registry
  (`fdu-a6dz`), block snapshot (`fdu-xihx`), optimized revalidation (`fdu-wbis`),
  contention proof (`fdu-r27g`), and pinned toolchain (`fdu-zga3`) also block `fdu-ywu0`
  through the Phase 1 graph.

The implementation tasks remain open after this planning record closes.

## References

- [Performance-evidence research](../../research/research-2026-08-09-end-to-end-performance-evidence.md)
- [Unix `DirEntry` metadata evidence](../../research/research-2026-08-09-portable-direntry-metadata.md)
- [Reconciliation index fast-path evidence](../../research/research-2026-08-09-reconciliation-index-fast-path.md)
- [fdu Phase 1 plan](plan-2026-08-08-fdu-phase-1.md)
- [fdu file-roll-up engine research](../../research/research-2026-08-06-file-rollup-engine.md)
- [fdu CLI golden-test plan](../done/plan-2026-08-09-fdu-cli-golden-tests.md)
- [fdu Rust engineering quality plan](plan-2026-08-09-fdu-rust-engineering-quality.md)
- [Flowmark performance report at the reviewed revision](https://github.com/jlevy/flowmark-rs/blob/015f23989af3e5cfb3f8b58dfc72822c534df25a/benchmarks/REPORT.md)
- [Flowmark benchmark comparison runner](https://github.com/jlevy/flowmark-rs/blob/015f23989af3e5cfb3f8b58dfc72822c534df25a/benchmarks/run_comparison.sh)
- [Flowmark performance and profiling plan](https://github.com/jlevy/flowmark-rs/blob/015f23989af3e5cfb3f8b58dfc72822c534df25a/docs/project/specs/done/plan-2026-02-26-perf-comparison-profiling.md)
- [Flowmark parallel-processing plan](https://github.com/jlevy/flowmark-rs/blob/015f23989af3e5cfb3f8b58dfc72822c534df25a/docs/project/specs/done/plan-2026-02-27-parallel-file-processing.md)
- [Flowmark cache and performance roadmap](https://github.com/jlevy/flowmark-rs/blob/015f23989af3e5cfb3f8b58dfc72822c534df25a/docs/project/specs/done/plan-2026-02-27-incremental-cache-and-performance-roadmap.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
