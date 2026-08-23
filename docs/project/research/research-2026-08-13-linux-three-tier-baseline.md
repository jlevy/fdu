# Research: The Linux Three-Tier Baseline — Aggregate, Index, and Content

**Date:** 2026-08-13

**Author:** fdu project, with Claude Code review assistance

**Status:** Proposed

## Overview

fdu now answers at three retention tiers, and only one of them has ever been measured
across platforms. This note establishes a Linux baseline for all three on one host,
immediately after the content-metrics stack landed, and reports what the numbers say
about where Linux work should go.

The three tiers, in the vocabulary the engine already uses:

1. **Aggregate** — `RetainedState::Summary`: a complete scan reduced to five exact
   tallies, no path or hierarchy records retained.
   This is the tier that competes with `diskus` and `du -s`.
2. **Index** — `RetainedState::FullIndex`: the reusable parent-pointer tree with
   pre-computed roll-ups, optionally persisted as a snapshot.
   This is the tier that competes with `dut`, `pdu`, and `dust`, and the only one a
   cache or a second view can consume.
3. **Content** — an analysis profile over retained entries, with its metrics persisted
   in a sidecar keyed to the metadata snapshot.
   This is the tier that competes with `scc` and `tokei`.

Three findings order the work:

1. **The tiers are separated by more than an order of magnitude, and the separation is
   the design working.** On one 14,542-file tree the aggregate tier answers in 24 ms
   where code analysis takes 1,105 ms.
   Nothing here argues for collapsing them.
2. **The metadata cache is a net loss on Linux and the content sidecar is a net win**,
   through the same warm-open machinery.
   Warm metadata open runs +70% against a cold scan at both 14.5k and 450k entries,
   while the content sidecar returns −51% on the code profile.
   The difference is not the cache; it is that analysis is expensive enough to outrun a
   fixed warm-open tax that metadata-only work cannot.
3. **Snapshot load costs as much as re-walking the tree.** At 450,463 entries, loading a
   snapshot takes 1,305 ms against a 1,326 ms cold scan of the same view.
   Every warm run pays that before it does anything else.

## Environment and Method

- **Host:** 4-vCPU Intel Xeon @ 2.8 GHz (KVM, virtio disk), 15 GiB RAM, kernel 6.18.5,
  ext4, running as root.
  This is a **virtualized-warm** regime in the sense of
  [the platform tuning guide](../guides/platform-tuning.md): warm measurements are
  representative of the common deployment case, while cold measurements on this host
  order strategies without measuring device latency.
- **Subjects:** two nominated trees, both outside the repository, neither written to
  during measurement.
  - `meta450k` — the committed `explorations/benchmarks/spikes/gen_tree.py` generator at
    450,000, yielding 450,463 entries (28,629 directories, 421,690 files, 143 symlinks),
    3.00 GiB apparent, 748 MB allocated.
    Chosen to be directly comparable with
    [the 2026-08-13 first Linux measurements](research-2026-08-13-linux-first-measurements.md).
  - `content` — 14,542 real text files in 2,558 directories, 366 MB, assembled from npm
    `node_modules`, Rust crate sources from the Cargo registry, and the CPython standard
    library. **Real text is required here**: both committed corpus generators write
    filler bytes (`corpus.py` repeats a SHA-256 digest, `gen_tree.py` writes
    `b"x" * size` and sparse-truncates the remainder), so a generated tree measures the
    binary gate and a single pathological line rather than language and prose structure.
- **Method:** `explorations/benchmarks/spikes/paired_runner.py`, adjacent paired trials
  with alternating order, 10 pairs per matchup, two full-tree warmups per tool, wall via
  monotonic clock around spawn+wait, rusage via `wait4`, paired medians with a
  4,000-resample bootstrap 95% interval.
- **Binary:** `fdu` release build at the content-metrics head (`2abdb11`), all features.

Nothing in this note is a ledger experiment.
It is a baseline against which candidates are measured, and per the reproduction rule it
names the regime rather than claiming a portable number.

## The three tiers, side by side

On the 14,542-file `content` tree, so that all three tiers answer over the same subject:

| Tier | Command | Wall | CPU | Peak RSS |
| --- | --- | ---: | ---: | ---: |
| Aggregate | `--cache off --view summary` | **23.9 ms** | 0.08 s | 11.7 MB |
| Index | `--cache off --view tree` | 53.6 ms | 0.11 s | 14.7 MB |
| Content, basic | `--cache off --analyze basic --view types` | 624.5 ms | 1.75 s | 38.8 MB |
| Content, documents | `--cache off --analyze documents --view documents` | 806.1 ms | 2.35 s | 44.6 MB |
| Content, code | `--cache off --analyze code --view languages` | 1,104.5 ms | 3.42 s | 43.2 MB |

Code analysis costs 46× the aggregate tier on the same tree, which is the expected
shape: the aggregate tier reads directory and inode metadata, and the content tier reads
166 MB of file payload and decodes it.
The tiers are doing genuinely different work and the ratio is evidence the planner is
right to keep them apart.

At 450,463 entries the aggregate/index separation widens, because the index is the tier
whose cost scales with entry count rather than byte count:

| Tier | Wall | CPU | Peak RSS |
| --- | ---: | ---: | ---: |
| Aggregate (`--view summary`) | **415.6 ms** | 1.56 s | **22.7 MB** |
| Index (`--view tree`) | 1,332.5 ms | 2.59 s | 274.0 MB |

The index costs +207% wall and 12× peak memory over the aggregate tier for the same
walk. This reproduces the first-measurements finding that the Linux gap is the index
consumer rather than the enumeration layer.

## The warm path is a tax the content tier can afford and metadata cannot

Every cached run performs the same sequence: load a snapshot, reconcile it against the
tree, then clone the index and write it back.
Decomposing that with `--cache only` (load, no reconcile, no save) against
`--cache auto` (the full warm path) separates the two halves.

At 450,463 entries, `--view tree`:

| Stage | Wall | CPU | Peak RSS |
| --- | ---: | ---: | ---: |
| Cold scan, no cache | 1,326.0 ms | 2.60 s | 277.4 MB |
| Snapshot load alone (`--cache only`) | 1,304.5 ms | 1.30 s | 194.5 MB |
| Full warm open (`--cache auto`) | 2,261.1 ms | 3.41 s | 411.6 MB |

Reconcile plus clone plus save is +73.3% [67.6%, 81.5%] on top of load, and the deep
clone accounts for the +217 MB. Loading the snapshot costs 1,305 ms against a 1,326 ms
cold scan of the same view: **the cache costs as much to read as the walk it replaces,
before any of its own overhead.**

The same decomposition on the 14,542-file tree, where load is cheap, isolates the second
half:

| Stage | Wall | CPU | Peak RSS |
| --- | ---: | ---: | ---: |
| Cold scan, no cache | 53.6 ms | 0.11 s | 14.7 MB |
| Snapshot load alone | 44.4 ms | 0.04 s | 11.7 MB |
| Full warm open | 89.2 ms | 0.15 s | 21.5 MB |

Warm open is +70.3% [62.2%, 82.9%] against a cold scan here and +69.1% [61.3%, 87.5%] at
450k. **The inversion is scale-independent**, which rules out an explanation in terms of
snapshot size alone and points at the unconditional work every warm run performs.

Against that fixed tax, the content sidecar still wins decisively, because the analysis
it avoids is much more expensive than the tax it pays:

| Profile | Cold | Warm (sidecar) | Change | 95% CI | Cold CPU | Warm CPU |
| --- | ---: | ---: | ---: | --- | ---: | ---: |
| basic | 624.5 ms | 520.1 ms | −16.3% | [−18.7%, −14.3%] | 1.75 s | 0.61 s |
| documents | 806.1 ms | 516.7 ms | −35.7% | [−36.7%, −33.7%] | 2.35 s | 0.61 s |
| code | 1,136.3 ms | 562.6 ms | **−51.1%** | [−53.0%, −48.0%] | 3.50 s | 0.66 s |

Two things are visible in that table.
The first is that the sidecar works: CPU falls by 2.9× to 5.3× across profiles, and the
`code` profile — the most expensive, and the one users would reach for to count lines —
gains the most. The second is that **all three warm runs converge on the same ~520 ms
floor** regardless of profile.
That floor is not analysis; with the sidecar hit, analysis is nearly free.
It is the warm-open tax, and it is what caps the `basic` profile’s return at −16% when
its CPU fell by 65%.

Decomposing the content warm path the same way shows where the floor lives:

| Stage | Wall | CPU | Peak RSS |
| --- | ---: | ---: | ---: |
| Load metadata snapshot only | 44.4 ms | 0.04 s | 11.7 MB |
| Load metadata + content sidecar (`--cache only`) | 415.2 ms | 0.41 s | 42.6 MB |
| Full warm content open | 510.6 ms | 0.60 s | 70.1 MB |

The **content sidecar load costs about 370 ms for 14,542 files, roughly 25 µs per
file**, against about 3 µs per record for the metadata snapshot.
Loading precomputed metrics is an order of magnitude more expensive per record than
loading the metadata they describe, and it is the dominant cost of every warm content
run. That is the layer-3 finding: the content tier’s remaining Linux cost is in its
sidecar format, not in its analyzers.

## First fix: the byte-identical rewrite

`fdu-maxn` was the cheapest of the three pieces and is now implemented.
A warm open whose reconciliation mutated nothing was cloning the index, encoding it, and
writing it back. The rewrite was verifiably redundant: three consecutive warm runs over
an unchanged tree produced the same SHA-256 over the same 1,168,133 bytes, so the write
reproduced the file it had just read.
`ApplyStats::mutated()` distinguishes a pass that changed indexed state from one that
only confirmed it, and metadata and content sidecars are judged separately because they
are invalidated separately — a run that adds content metrics still writes the sidecar
without rewriting the metadata snapshot, and a load that saw stale sidecar records still
rewrites to drop them.
Skipping the write also skips the clone, which is where the memory comes from.

Paired against an immutable control binary built from the same tree, 12 pairs, warm:

| Subject | Job | Control | Candidate | Change | 95% CI | Peak RSS |
| --- | --- | ---: | ---: | ---: | --- | --- |
| meta450k | warm tree | 2,177.1 ms | 1,750.6 ms | **−20.6%** | [−21.2%, −16.6%] | 411.2 → 194.7 MB |
| content | warm tree | 85.3 ms | 66.6 ms | **−22.8%** | [−23.9%, −20.1%] | 21.5 → 12.3 MB |
| content | warm code analysis | 522.3 ms | 463.0 ms | **−11.6%** | [−12.2%, −9.8%] | 70.1 → 43.2 MB |

The content-tier gain is smaller because the sidecar load dominates that path, not
because the fix works less well there; its RSS falls by the same mechanism.

This narrows the inversion without closing it.
Warm open at 450k moves from +64% against a cold scan to +32%, and what remains is
almost entirely the 1,305 ms snapshot load.
`fdu-91ts` owns that, and until it lands the Linux cache is still a net loss for
metadata-only work.

These numbers come from the spike harness in a virtualized-warm regime, which is
evidence about that regime and is not a ledger artifact.
The change is a defect fix rather than a tuning change — the design principles already
hold that “a warm path that loses to a cold scan of the same view is a defect” — so it
does not rest on the 3% accept rule, but a ledger-protocol run on both platforms should
still follow.

## What this changes

1. **The warm-open tax is one defect with three named pieces, and it now has a Linux
   size.** The design principles already classify this as a defect rather than a
   trade-off — “a warm path that loses to a cold scan of the same view is a defect” — so
   it does not need to clear the 3% accept bar; it needs to stop being true.
   `fdu-91ts` (load records through their known parent id with one deferred bottom-up
   roll-up pass) attacks the 1,305 ms; `fdu-niuz` (share the index with the writer
   instead of cloning it) attacks the +217 MB; `fdu-maxn` (skip the save when
   reconciliation applied nothing) attacks the part of the 957 ms that is a byte-
   identical rewrite.
2. **The content sidecar load format is the layer-3 target**, at about 25 µs per file.
   It is the same class of problem as H78 for the metadata snapshot, and probably wants
   the same answer: a layout usable without rebuilding per-record state.
3. **The aggregate tier is in good shape and should be protected by a probe job.** It is
   the tier with no `component_ns` instrumentation, which is why exp-043 and exp-044
   both resolved on diluted wall numbers.

## Reproduction

Both trees are nominated, not committed.
`meta450k` regenerates deterministically from
`explorations/benchmarks/spikes/gen_tree.py`; the `content` tree is assembled from
whatever real source trees the host has and is described by its counts rather than its
paths, so a replication reports its own fingerprint rather than assuming this one.
The matchup definitions are JSON tool tables passed through `SPIKE_TOOLS`, so no
absolute path from this host is baked into the harness.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
