# fdu Live Uncached Tool Comparison on macOS

**Date:** 2026-09-26, refreshed 2026-09-28 on stack 141

**Status:** Exploratory local performance evidence from an uncontrolled host

## Outcome

The tables below are the 2026-09-28 refresh, which measured `a5c0ab46`: the top of stack
141 ([#137](https://github.com/jlevy/fdu/pull/137) →
[#138](https://github.com/jlevy/fdu/pull/138) →
[#139](https://github.com/jlevy/fdu/pull/139)), whose engine is the post-merge `main`.
The 2026-09-26 run measured `main` at `5d7ac7ba`; its figures are kept under
[First Run](#first-run).

On a reproducible generated tree of 1,000,001 entries, a fresh fdu process with its
persisted cache disabled built a reusable exact index and rendered a depth-one, ten-row
tree in a **6.4-second median**. That is **137k files/s** and **0.47 GB/s** on this
subject, measured under heavier competing load than the first run.

The same matrix measured every available baseline requested by the comparison harness.
Each competitor ran immediately beside fdu with alternating order, so the relative
figures are paired even though the host was not quiet.

| Tool | Work class | Median wall-clock time | Wall time vs. fdu | Files/s | GB/s | 95% interval | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| **fdu** | indexed tree | **6.4 s** | baseline | **137k** | **0.47** | — | 285.7 MiB |
| dumac | total only | 6.9 s | +9% | 127k | 0.43 | +4% to +14% | 29.6 MiB |
| pdu | rendered tree | 9.2 s | +49% | 96k | 0.33 | +41% to +52% | 12.0 MiB |
| diskus | total only | 9.3 s | +42% | 94k | 0.32 | +34% to +48% | 10.9 MiB |
| dua | total only | 10.4 s | +61% | 84k | 0.29 | +49% to +65% | 21.1 MiB |
| gdu | rendered tree | 10.5 s | +67% | 83k | 0.28 | +55% to +74% | 516.5 MiB |
| dust | allocated total | 11.0 s | +57% | 79k | 0.27 | +52% to +86% | 523.1 MiB |
| BSD `du` | total only | 55.6 s | +801% | 16k | 0.054 | +778% to +839% | 1.2 MiB |
| ncdu | indexed tree | 67.3 s | +968% | 13k | 0.044 | +919% to +995% | 2.0 MiB |
| GNU `du` | total only | 68.0 s | +960% | 13k | 0.044 | +908% to +1001% | 5.8 MiB |

Positive percentages mean that the competitor took more wall time than its immediately
adjacent fdu run. The paired column and the medians can order two rows differently, as
pdu and diskus do here: the medians come from each tool’s own samples, the percentage
from its pairs. Displayed values are rounded; raw samples retain full precision.
`k` means thousands.
Files/s divides the subject’s 875,000 regular files by median wall time.
Allocated GB/s divides its 2,986,741,760 allocated bytes by median wall time, using
decimal GB. This is a metadata-coverage rate, not file-content read bandwidth.

These are not equal-output jobs.
fdu and ncdu construct an index; pdu and gdu render a tree; several faster comparators
return one aggregate.
fdu additionally returns file and directory counts, apparent and allocated bytes, newest
file time, per-directory roll-ups for the whole tree, and per-extension tallies, then
retains the index for the next query.
The work-class column keeps that distinction visible; total-only rows are useful lower
bounds, not claims of semantic equivalence.

The retained index also costs memory: fdu peaked at about 286 MiB, versus dumac’s 30
MiB. For totals without an index, use `fdu --no-gitignore --view summary`; the
[separate summary-mode measurement](report-2026-09-16-fdu-live-tool-comparison.md) used
about 15 MiB. That mode trades retained state for memory, not necessarily speed.

Totals have defined semantics: fdu counts a hard-linked file once per path and excludes
symbolic links’ own sizes and directory blocks.
The [peer-agreement report](report-2026-09-25-peer-agreement.md) checks those
differences against other tools.
Metadata-only reruns still revalidate the tree; content analysis can reuse unchanged
file-body results.

### What the default `fdu .` costs

The table measures fdu with `--cache off`, as every published comparison has.
The same matrix also paired the bare default command, `fdu PATH`, beside the same
anchor, with its cache in a benchmark-owned directory that persisted across the run:

| fdu build and command | Work returned | Wall time vs. `--cache off` | 95% interval | Peak RSS |
| --- | --- | ---: | ---: | ---: |
| `a5c0ab46`, `fdu PATH` (`--cache auto`) | default tree, five levels, 1% floor | +0.1% | −4.5% to +1.9% | 285.8 MiB |
| `ea786683` (#138), `fdu PATH` | same, and encodes its snapshot every run | +1.5% | −2.0% to +3.8% | 385.9 MiB |

On the current engine the default one-shot command costs what `--cache off` costs,
because `--cache auto` no longer writes a snapshot that no later one-shot run reads.
The previous default encoded one on every run and wrote it when it changed; on macOS
that cost about 100 MiB of peak memory and a wall difference this host could not
resolve.

### Stack 141 on macOS

The same matrix paired the bottom of the stack, `4e008e78` (#137), with the top on the
table’s own `--cache off` contract.
#137 took 5.6% longer, with a paired interval of −0.1% to +10.3%: the stack did not
regress the published figure on macOS. Peak RSS was unchanged (286.0 against 285.7 MiB).

A stacked probe run isolated the two engine changes the stack adds, pairing the release
probes of the three heads in one interleaved schedule, 12 trials each after 3 warm-ups.

| Change | Job | Wall | User CPU | Peak RSS |
| --- | --- | ---: | ---: | ---: |
| H156, #137 → #138 ([exp-164](../experiments/exp-164-macos-one-shot-index-release-shows-no-wall-change-and-no-reg.md)) | `default-tree` | −1.00% [−5.13%, +6.11%] | −2.43% | −0.05% |
|  | `default-tree-first` | +0.16% [−5.12%, +4.01%] | −2.30% | −0.09% |
|  | `cold-scan-index` (placebo) | +2.09% [−0.72%, +3.62%] | +0.72% | −0.03% |
| H160, #138 → #139 ([exp-165](../experiments/exp-165-macos-auto-cache-policy-cuts-default-tree-peak-rss-26-but-mi.md)) | `default-tree` | −3.09% [−6.75%, +2.00%] | −9.65% | −26.29% |
|  | `default-tree-first` | −2.98% [−7.12%, +5.12%] | −9.22% | −26.29% |
|  | `cold-scan-index` (placebo) | −0.44% [−2.46%, +1.42%] | −0.96% | +0.02% |

Neither change clears the 3% accept rule on macOS, and neither regresses: no median is
worse than +2.1%. Linux measured −3.19% for H156
([exp-160](../experiments/exp-160-linux-one-shot-index-release-off-the-answer-path-clears-3-on.md))
and −13.81% for H160
([exp-163](../experiments/exp-163-linux-auto-cache-policy-stops-one-shot-snapshot-writes-clear.md)).
On macOS each run spends about 37 CPU-seconds in the kernel and 1.3–1.5 in user space,
so both changes remove user-space work that is a few percent of wall at most: the index
free that H156 moves, and the 79 MB snapshot encoding that H160 stops.
Their user-CPU and memory savings show; wall savings of about 0.2 s cannot be resolved
on this host.

Ten-run `hyperfine` screens of the product commands came first, each pair in both
orders. Under this load they could not resolve a few percent: the summary placebo, which
H156 cannot move, differed by 6.8% between the pooled #137 and #138 arms.
Their raw samples are in [the screen results](fdu-macos-screens-result-2026-09-28.json).
Two things they do show:

- `--stale-ok` after one `--cache on` run answered in **0.69 s** (median of 10), against
  6.4–9.5 s medians for every walk series in the same session, 9× or more.
  The seeded snapshot was 79,300,020 bytes.
- At a million entries the cache policy behaves as specified: `--cache auto` on #139
  left no cache file for `fdu PATH` or `--view summary`, `--cache off` left none, and
  `--cache on` and #138’s default each left exactly one.

### H153 Confirmation

The quiet confirming run of the shared metric-resolution pass
([exp-159](../experiments/exp-159-share-content-metric-resolution-across-views.md#quiet-confirmation-attempt-2026-09-28))
was attempted on the same stack top, on the unchanged `metabrowser-clone` subject, and
failed to qualify: the unchanged 25% gate invalidated 20 of 24 timed samples.
It produced no figure, it was not rerun under a weaker regime, and exp-159’s provisional
verdict stands.

## Measurement Protocol

The publishable local comparison has four phases:

1. Finish and test benchmark-harness changes, then make one immutable release binary.
2. Generate and verify the committed `balanced` recipe at its 1,000,000-entry scale
   point on the internal APFS volume.
3. Run one fixed matrix with 3 full-tree warm-ups and 12 timed adjacent pairs per
   competitor. Alternate whether fdu or the competitor runs first within each pair.
4. Verify the independent pre/post tree fingerprints, per-sample semantic answers,
   summary oracle, baseline, and executable identities before publishing the result.

For both runs, the measured corpus, benchmark-owned temporary directory, isolated fdu
cache state, baseline, and raw results all lived on the same internal APFS SSD. Cargo
targets, uv caches, the Python environment, and the copied immutable fdu binaries lived
on an external SSD. No build or dependency work ran during measurement.
This split keeps disposable build traffic off the internal drive without moving the
filesystem work being measured away from it.

The refresh built one release binary per stack head, each from its own worktree and
Cargo target: `4e008e78` (#137), `ea786683` (#138), and `a5c0ab46` (#139). Each answered
`--cache off --depth 1 --limit 10` byte-identically on a small tree before measurement.
The 2026-09-26 corpus had been removed after that run, so the refresh generated a new
copy of the same recipe.
Every timed cell held a timing lock shared with the other agents on this host, so none
of their builds or benchmarks overlapped it.
No RAM disk was used.

One matrix used `fdu`’s indexed-tree contract as its anchor and included every work
class. That is appropriate for the README’s orientation table because every competitor
gets its own adjacent fdu control and the table names the work returned.
Optimization verdicts or like-for-like claims should still use separate matrices with
matching work classes.

## Subject and Validity

- **Subject.** The committed `balanced` recipe, seed `fdu-balanced-v1`: 875,000 regular
  files and 125,001 directories, 2,986,741,760 allocated and 793,658,448 apparent bytes.
  Its semantic digest is `4bbd97c0d3d4e2ad` in both runs, as on Linux.
- **Binaries.** Refresh anchor: `fdu 0.1.0-dev+ga5c0ab46d`, SHA-256
  `f928319e8b3c20b34edd78bc6e90941649173386d14c44791bd9479330112cab`. Paired fdu rows:
  `fdu 0.1.0-dev+gea7866837`,
  `3dab589adc6da66aea4cfeeb5dfc31e89166a9a20768ce4c76d864454763aaff`, and
  `fdu 0.1.0-dev+g4e008e78a`,
  `5d26671522b58a40d72a604c5b8e7368e4eac9c2d72c193bdf2731622c3c244c`. First run:
  `fdu 0.1.0-dev+g5d7ac7bab`, built from commit `5d7ac7ba`, SHA-256
  `0b52855f7c5d2397de15fbb71483191ca162667a2774f71c5334aed578595cb7`.
- **Peers.** All nine executables hashed identically in both runs: dumac, dust 1.2.4,
  diskus 0.9.0, pdu 0.24.0, dua 2.41.1, gdu v5.36.1, ncdu 2.9.2, BSD `du`, and GNU `du`
  9.9.
- **Schedule.** Refresh: nine peers plus three fdu rows, 3 warm-ups per tool, and 12
  timed adjacent pairs per competitor: 360 tool processes under a fixed-N stopping rule,
  in 105 minutes. First run: nine competitors, 270 processes.
- **Correctness.** In both runs, zero invalid timed samples, semantic mismatches, or
  independent summary-oracle mismatches; no baseline drift and no tree mutation.
- **Host.** Apple M1 Pro, 8 performance and 2 efficiency cores, 32 GiB, bare-metal
  Darwin 25.5.0, AC power, and normal thermal pressure at every observation.
- **Cache state.** Warm-steady after one complete independent fingerprint and at least
  three full-tree warm-ups per tool.
  This means repeated-workload steady state, not guaranteed residency of every APFS
  metadata object.

Exact commands, versions, executable hashes, host and tree facts, raw paired samples,
resource measurements, validity results, and confidence intervals are in the refresh’s
[comparison result](fdu-live-tool-comparison-result-2026-09-28.json) and the first run’s
[comparison result](fdu-live-tool-comparison-result-2026-09-26.json).
The stacked probe run is committed with
[exp-164](../experiments/evidence/exp-164/run.json).

## Host Regime and Interpretation

The refresh ran on a host busy with unrelated work that the timing lock does not cover.
Between screen series the harness’s own one-second observation read 49–100% CPU busy,
with a 1-minute load average of 10 to 40 on ten cores.
Across the sample boundaries it read 17–100% (median 46%) during the probe run and
12–100% (median 30%) during the tool matrix.
The quiet gate refused the probe run at 31.8% CPU busy and the tool matrix at 47.4%, so
both kept the 25% gate unchanged and declared their regime `uncontrolled`. The matrix’s
CPU busy was 47.75% at the start and 32.34% at the end.

Pairing limits the effect on within-row comparisons because the two processes run
seconds apart and alternate order, but it cannot turn these absolute seconds into a
quiet-host measurement, and it widens every interval.
The 6.371-second median should not be read as a regression from 5.991 seconds on
2026-09-26: the paired #137 row, measured beside the same anchor, says the stack made
the command slightly faster, not slower.

The result answers the narrower question the matrix can support: under the observed
load, fdu remained faster in paired wall time than every measured comparator while
returning its reusable index and tree, as it did on 2026-09-26. The closest comparator
was again dumac, at +8.5% with a paired 95% interval of +3.7% to +13.6%, despite dumac
returning only one allocated-byte total.
Each fdu run spent about 36.8 CPU-seconds in the kernel and 1.3 in user space, dumac
24.4 and 0.8: the ranking comes from `getattrlistbulk` and fdu’s wider worker pool, as
the
[cache economics brief](../research/research-2026-09-27-cache-economics-and-default-plans.md#why-the-ranking-differs-between-macos-and-linux)
explains. This comparison says nothing about Linux, where the ranking differs.

### First Run

The 2026-09-26 run measured `main` at `5d7ac7ba` under the same protocol and a lighter
load: the intended quiet regime rejected three starts, one earlier attempt was stopped
during warm-up, and CPU busy was 55.44% at the start and 33.37% at the end.

| Tool | Work class | Median wall-clock time | Wall time vs. fdu | 95% interval | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: |
| **fdu** | indexed tree | **6.0 s** | baseline | — | 285.9 MiB |
| dumac | total only | 6.3 s | +8% | +5% to +16% | 29.6 MiB |
| diskus | total only | 8.7 s | +45% | +32% to +49% | 10.8 MiB |
| pdu | rendered tree | 9.2 s | +53% | +49% to +60% | 12.0 MiB |
| dust | allocated total | 9.6 s | +59% | +55% to +67% | 545.9 MiB |
| dua | total only | 9.7 s | +63% | +51% to +71% | 21.2 MiB |
| gdu | rendered tree | 10.4 s | +64% | +55% to +83% | 510.4 MiB |
| BSD `du` | total only | 49.3 s | +717% | +611% to +742% | 1.2 MiB |
| ncdu | indexed tree | 60.6 s | +910% | +873% to +951% | 2.0 MiB |
| GNU `du` | total only | 62.1 s | +955% | +891% to +1004% | 5.8 MiB |

That run’s 5.991-second median was not a regression from 5.206 seconds on 2026-09-16
either: the recipe matched, but the competing system load did not.

The previous macOS comparison is the
[2026-09-16 report](report-2026-09-16-fdu-live-tool-comparison.md).
The benchmark entry point and storage rules are in the
[benchmark README](../../../explorations/benchmarks/README.md), and the broader method
is in the [performance loop](../guides/performance-loop.md).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
