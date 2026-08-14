# fdu Performance Evidence Harness

This directory contains the repository-owned tooling that creates reproducible
performance evidence.
The current implementation generates deterministic filesystem corpora, verifies them
with an implementation independent of fdu, applies exact churn transitions, and runs the
repository-only Rust component probe through a strict evidence state machine.
It separates correctness-gated experiment evidence from external-tool calibration.
Current published measurements and their limitations live in the README and committed
performance reports; raw host-specific runs remain outside the repository.

The full methodology, runner design, and release gates live in the
[end-to-end performance plan](../docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md).

## Quick Start

The generator uses only the Python standard library and does not install a project or
contact the network.
Run it from the repository root:

```shell
uv run --no-project python -m benchmarks.generate create \
  --recipe contract \
  --work-dir benchmarks/corpus
```

The command prints one JSON object.
Save its `run_root`, then verify or remove that exact run:

```shell
uv run --no-project python -m benchmarks.generate verify \
  --run-root benchmarks/corpus/fdu-perf-EXAMPLE

uv run --no-project python -m benchmarks.generate cleanup \
  --run-root benchmarks/corpus/fdu-perf-EXAMPLE
```

Churn recipes declare an ordered state machine.
Each transition verifies the current manifest before changing anything:

```shell
uv run --no-project python -m benchmarks.generate create \
  --recipe churn-local \
  --entries 1000 \
  --work-dir benchmarks/corpus

uv run --no-project python -m benchmarks.generate mutate \
  --run-root benchmarks/corpus/fdu-perf-EXAMPLE \
  --transition one-change
```

Run the corpus correctness suite with:

```shell
make test-performance
```

This builds the excluded `perf_probe` example and runs both the corpus/schema/runner
suite and the real-tree evidence-harness suite, including an actual probe scan checked
against an independently fingerprinted filesystem tree.
The suite has no numeric speed assertion and is included in `make check`. Its component
matrix includes metadata, basic content, code SLOC, normalized text, reader-visible
Markdown, binary gating, content-cache reuse, churn, grouped content-query jobs, and
separate resolved-versus-ambiguous classification probes.
Large-corpus measurements remain separate from that correctness gate.

## Execute and Validate Evidence

`run.py` is the structured entry point for scenario execution and evidence handling.
Every command emits exactly one JSON object, writes diagnostics to stderr, and uses a
nonzero status for an invalid request or incompatible comparison.

The committed [scenario set](scenarios.json) covers scan production, scan plus index,
snapshot save/load, unchanged and changed revalidation, deterministic delta application,
and steady query work.
Build and execute it with:

```shell
cargo build --locked --release -p fdu --example perf_probe --no-default-features

uv run --no-project python -m benchmarks.run execute \
  --scenarios benchmarks/scenarios.json \
  --executable fdu-probe=/absolute/path/to/target/release/examples/perf_probe \
  --work-dir /absolute/scratch/fdu-performance \
  --output-dir /absolute/results/fdu-performance \
  --order-seed documented-seed
```

An executable mapping is a direct argument-vector prefix.
Repeat an adapter name for an interpreter plus script; no command passes through a
shell.

One verified base is generated per effective recipe, seed, and scale within a result
run. Each warmup and timed invocation receives an independent clone or bounded copy of
that base plus freshly prepared snapshot and filesystem-cache state.
APFS `clonefile` and Linux `FICLONE` are used only after a live probe proves
copy-on-write independence; the fallback refuses more than 8 GiB of logical copying.
Internal hardlinks are re-created only within the trial and never connect mutable trial
files to the base. The runner independently refreshes and verifies every trial’s exact
filesystem identity.
Base generation, verification, clone/copy counts, copied bytes, strategy-probe time, and
total setup time are recorded separately and excluded from the measured command.
The runner uses a minimal environment, kills the child process group on a timeout,
validates the corpus again afterward, and retains invalid samples with mechanical
reasons. It writes one exclusive result file; it never replaces an earlier run.

Validate, render, or compare an existing result without executing a benchmark:

```shell
uv run --no-project python -m benchmarks.run validate \
  --kind result --path /absolute/results/run-EXAMPLE.json

uv run --no-project python -m benchmarks.run render \
  --result /absolute/results/run-EXAMPLE.json \
  --output /absolute/results/report.md

uv run --no-project python -m benchmarks.run compare \
  --current /absolute/results/current.json \
  --baseline /absolute/results/baseline.json
```

Comparison requires the same host capabilities, harness and Python runtime, scenario
contracts, observed corpora, executable command shapes, and collector availability.
Executable checksums are recorded but may differ: comparing two code revisions is the
normal regression use case.
An exact checksum change is reported separately.

The portable runner deliberately rejects `controlled-cold`. That label requires the
dedicated-host eviction protocol and supporting collector evidence, which are owned by
`fdu-8z5l`. A successful cache-preparation command can establish `verified-warm`; normal
developer and hosted-CI runs remain `uncontrolled`.

### Standard Local Near-Million-Entry Comparison

The repository’s `benchmarks/` subtree is the standard self-contained large local
testbed. Its 2026-08-13 fingerprint contained 901,963 entries: the ignored generated
corpus, benchmark environment, harness, schemas, and prior result artifacts.
It is large and heterogeneous enough to expose real directory topology while excluding
volatile Git and Rust build state.
Generated state moves the count, so fingerprint every run and treat “near-million scale”
as the designation rather than assuming an exact size across machines.

Finish every change under `benchmarks/` before measuring.
Copy immutable binaries outside the subtree, and do not run the benchmark test suite,
update its environment, or write result artifacts there until the post-run fingerprint
finishes.
The Make defaults put baselines, scratch snapshots, profiles, and results under
`/tmp/fdu-realtree`; the CLI also rejects explicit output or scratch paths inside the
measured root.

Run tree and scalar work classes as separate matrices so the headline does not present a
one-number total as equivalent to a reusable index and rendered tree:

```shell
make perf-compare-tools \
  PERF_TREE=benchmarks PERF_LABEL=benchmarks-self-contained \
  PERF_TOOL_CONTROL=/tmp/fdu-tool-comparison/bin/fdu \
  TOOL_ARGS='--tool dust=/path/to/dust --tool gdu=/path/to/gdu-go --tool pdu=/path/to/pdu --tool ncdu=/path/to/ncdu' \
  STORAGE='local APFS SSD' NAME=tree-900k

PYTHONDONTWRITEBYTECODE=1 uv run --project benchmarks --frozen --group dev \
  python -m benchmarks.realtree.compare_tools \
  --root benchmarks --label benchmarks-self-contained \
  --anchor fdu-transient-summary=/tmp/fdu-tool-comparison/bin/fdu \
  --tool dumac=/path/to/dumac --tool diskus=/path/to/diskus \
  --tool dua=/path/to/dua --tool bsd-du=/usr/bin/du \
  --tool gnu-du=/path/to/gnu-du --trials 12 --warmups 3 \
  --storage 'local APFS SSD' --name scalar-900k
```

Homebrew installs the Go disk analyzer as `gdu-go` when GNU coreutils also owns `gdu`.
Resolve and hash the actual executable rather than assuming the command name.
Each competitor runs immediately beside fdu with alternating order.
The v3 fingerprint records redacted counts, depth, bytes, newest file time, and in-tree
hard-link duplication; any baseline drift, pre/post mutation, timeout, or nonzero exit
makes the run non-publishable.
The v3 summary oracle additionally checks files, descendant directories, apparent bytes,
allocated bytes, and newest regular-file mtime on every fdu sample.
Partial, stale, cached, or error-bearing output is invalid.
Reports label indexed-tree, rendered-tree, transient-summary, and total-only work
classes because those jobs are not semantically identical.

The reviewed M1/APFS result and exact manifest are in the
[live tool comparison](../docs/project/reports/report-2026-08-13-fdu-live-tool-comparison.md).
The architecture-level synthesis is the
[performance white paper](../docs/project/reports/report-2026-08-12-fdu-performance-architecture.md).

The tool runner’s `warm-steady` label is deliberately narrower than “everything fits in
RAM.” Before timing, the independent fingerprint walks every entry, and every tool then
receives explicit full-tree warmups; future JSON records retain both facts.
The label means repeated-workload steady state under whatever metadata-cache pressure
the subject creates.
It never means an fdu snapshot was used, and it never implies that all dentries, inodes,
vnodes, or APFS metadata blocks remained resident.

### Future Linux warm and cold comparison

The
[current diskus benchmark](https://github.com/sharkdp/diskus/blob/90196e950017d25b2940e8e0fda51a321ca66e1a/README.md#benchmark)
provides one practice the macOS run cannot reproduce without different platform
controls: it uses Hyperfine with five warmups for the warm regime, and runs `sync` plus
`/proc/sys/vm/drop_caches` before every timed Linux cold-cache sample.
It also uses a parameter scan before selecting a comparator’s worker count.

The future Linux matrix adopts that per-sample cache preparation for its
`controlled-cold` regime and reports a separate `verified-warm` regime.
It retains the stronger fdu evidence contract: adjacent paired scheduling, exact
binaries and host facts, immutable pre/post fingerprints, correctness oracles, work
classes, output digests, resource metrics, raw samples, and bootstrap confidence
intervals.
The runner must record every preparation command and failure; a label alone is
not proof that the kernel cache was evicted.

macOS `/usr/sbin/purge` is useful only as a separately labeled approximation: its own
manual says it approximates initial-boot buffer-cache conditions, while a full metadata
residency guarantee is unavailable.
A publishable macOS cold matrix therefore needs a quiet dedicated host and preferably a
disposable APFS volume remounted between samples.
Running on a corpus larger than `kern.maxvnodes` is valuable cache-pressure evidence,
but size alone does not establish a controlled-cold state.

`output-digest` drains stdout through a pipe and hashes it without timing filesystem
writes. Compact JSON postconditions are retained in a bounded 16 MiB buffer for untimed
validation.
Use `output-file` for complete or potentially large JSON; its writes are part
of the measured product job.
Both modes record byte count, digest, first-output latency, and completion latency.

External wall time includes process startup, setup performed by the command, compact
JSON emission, and pipe draining.
Component probes additionally record their explicit `component_ns` boundary; reports
show the two timings separately and never substitute a component duration for product
latency. On POSIX, per-child `wait4` evidence records user/system CPU, peak RSS, page
faults, block operations, and context switches.
Metrics that `rusage` cannot establish—byte I/O, retained RSS, and syscall count—remain
`null` with a reason.
A scenario may require named metrics; collector unavailability then invalidates that
scenario only.

The strict contracts are versioned under `schema/`. Runtime validation rejects unknown
fields and cross-checks the declared schedule, scenario states, environment, corpus,
process outcome, output, and self-checking result hash.
The result also records hashes for the executable components and the corpus, runner, and
schema and report implementations.

## Corpus Families

`corpora.json` is the strict, versioned source of recipe defaults.
`--entries` changes the required descendant count for parametric recipes; the root and
supported optional link cases are additional observed entries.

| Recipe | Purpose | Declared scale points |
| --- | --- | --- |
| `contract` | Small exact semantic contract, including empty files, extensions, nested paths, spaces, and optional links | 14 required descendants |
| `wide` | High sibling and directory-fan-out pressure | 10k, 100k, 500k, 1M |
| `deep` | Platform-safe chains plus leaves | 10k, 100k |
| `balanced` | General scale and throughput | 1k, 10k, 100k, 500k, 1M |
| `mixed-metadata` | Mixed and sparse sizes plus optional hardlinks and symlinks | 100k, 500k |
| `churn-local` | Ordered changes concentrated in one directory | 100k, 500k |
| `churn-distributed` | The same ordered change classes distributed across directories | 100k, 500k |
| `partial` | Capability-marked resilience input, never a ranking corpus | small, platform-specific |

The committed `expected/contract-v1.json` independently locks the contract paths, kinds,
sizes, extension totals, and optional-link semantics.
Unsupported symlink, hardlink, allocation, or permission cases are explicit capability
records; the generator never substitutes another case silently.

## Manifest Contract

Each run has three siblings:

```text
fdu-perf-UNIQUE/
├── .fdu-perf-run-v1
├── corpus/
└── observed-corpus.json
```

The observed manifest is root-inclusive and records:

- the effective recipe, seed, and canonical recipe hash;
- counts by kind and hardlink group;
- entry and unique apparent sizes plus allocated sizes where `st_blocks` exists;
- extension counts and apparent bytes;
- maximum depth and observed directory fan-out;
- platform capability decisions;
- the mutation state and exact changed paths;
- a normalized semantic digest and its independently reproducible components;
- an exact per-run engine digest over the fields retained by fdu;
- complete normalized records for corpora of at most 512 entries.

The semantic record includes the relative POSIX path, kind, apparent file size,
deterministic nanosecond mtime, symlink target, and canonical in-corpus hardlink source.
It deliberately excludes inode, ctime, absolute paths, and allocated blocks because
those values change across valid regenerations.
Timestamp writes request non-following behavior only where Python reports it available.
On platforms such as Windows that reject that flag, the generator verifies that each
owned timestamp target is not a symlink before using the ordinary path operation.
The verifier likewise keeps directory-entry metadata only where it carries authoritative
identity. On Windows it performs a fresh non-following path stat because Python’s cached
directory record deliberately zeros device, inode, and hardlink-count fields.
The separate engine digest includes allocated size, ctime, inode, and device identity.
It intentionally changes across valid corpus regeneration and is compared only to the
probe that scanned that exact invocation.
The portable semantic digest remains the cross-run corpus identity.

`sha256-multiset-v1` combines a SHA-256 leaf for each normalized record through count,
XOR, and modular-sum accumulators and hashes those components once more.
This is stable regardless of filesystem enumeration order and uses constant digest
memory. Generation builds the expected components from creation operations; a separate
`scandir` walk with authoritative non-following metadata must reproduce them before the
manifest is accepted.

## Mutation Contract

Churn recipes apply these transitions exactly once and in order:

1. `one-change` updates one deterministic file;
2. `modify-1pct` updates one percent of eligible files;
3. `mixed-1pct` performs a deterministic mix of modify, remove/add replacement, and
   same-directory rename operations.

Every phase preserves kind counts, apparent sizes, extension totals, and topology.
The manifest chains each transition to the previous manifest hash and records every old,
new, modified, and directory-mtime path.
A pre-existing mismatch stops the transition.

## Safety and Claim Policy

Generation occurs only below a newly reserved `fdu-perf-*` directory.
Cleanup rejects a nonexistent path, symlink, wrong name, absent or mismatched marker,
repository root, and every ancestor of the repository.
It never accepts an unmarked corpus path directly.

Create, verify, mutate, and cleanup acquire the same atomic per-run operation lock.
Concurrent access fails closed; an abandoned lock is never guessed to be stale.
Inspect the run before manually removing such a lock after an interrupted process.

Generated corpora and current results are ignored by Git.
Snapshot and output files belong beside `corpus/`, not inside the scanned root.
Do not commit large corpora or use smoke runtime as a performance baseline.

No duration is valid until the runner proves its precondition and postcondition against
the manifest. Numeric claims additionally require the state, host, collectors, raw
trials, comparator capabilities, and dedicated-host protocol specified by the active
plan.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
