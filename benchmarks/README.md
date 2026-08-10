# fdu Performance Evidence Harness

This directory contains the repository-owned tooling that creates reproducible
performance evidence.
The current implementation generates deterministic filesystem corpora, verifies them
with an implementation independent of fdu, and applies exact churn transitions.
It does not contain a performance result and does not show that the current portable
walker is fast.

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

The corpus suite takes well under a second at smoke scale and is included in
`make check`. Numeric timing and large-corpus runs remain separate from that correctness
gate.

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
- complete normalized records for corpora of at most 512 entries.

The semantic record includes the relative POSIX path, kind, apparent file size,
deterministic nanosecond mtime, symlink target, and canonical in-corpus hardlink source.
It deliberately excludes inode, ctime, absolute paths, and allocated blocks because
those values change across valid regenerations.
Later benchmark results may record a separate per-run identity digest for
fingerprint-sensitive jobs.

`sha256-multiset-v1` combines a SHA-256 leaf for each normalized record through count,
XOR, and modular-sum accumulators and hashes those components once more.
This is stable regardless of filesystem enumeration order and uses constant digest
memory. Generation builds the expected components from creation operations; a separate
`scandir` and `lstat` walk must reproduce them before the manifest is accepted.

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
