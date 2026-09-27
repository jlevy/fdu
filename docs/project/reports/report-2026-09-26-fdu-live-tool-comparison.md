# fdu Live Uncached Tool Comparison on macOS

**Date:** 2026-09-26

**Status:** Exploratory local performance evidence from an uncontrolled host

## Outcome

On a reproducible generated tree of 1,000,001 entries, a fresh fdu process with its
persisted cache disabled built a reusable exact index and rendered a depth-one, ten-row
tree in a **6.0-second median**. That is **146k files/s** and **0.50 GB/s** on this
subject.

The same matrix measured every available baseline requested by the comparison harness.
Each competitor ran immediately beside fdu with alternating order, so the relative
figures are paired even though the host was not quiet.

| Tool | Work class | Median wall-clock time | Wall time vs. fdu | Files/s | GB/s | 95% interval | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| **fdu** | indexed tree | **6.0 s** | baseline | **146k** | **0.50** | — | 285.9 MiB |
| dumac | total only | 6.3 s | +8% | 138k | 0.47 | +5% to +16% | 29.6 MiB |
| diskus | total only | 8.7 s | +45% | 101k | 0.35 | +32% to +49% | 10.8 MiB |
| pdu | rendered tree | 9.2 s | +53% | 95k | 0.32 | +49% to +60% | 12.0 MiB |
| dust | allocated total | 9.6 s | +59% | 91k | 0.31 | +55% to +67% | 545.9 MiB |
| dua | total only | 9.7 s | +63% | 90k | 0.31 | +51% to +71% | 21.2 MiB |
| gdu | rendered tree | 10.4 s | +64% | 84k | 0.29 | +55% to +83% | 510.4 MiB |
| BSD `du` | total only | 49.3 s | +717% | 18k | 0.061 | +611% to +742% | 1.2 MiB |
| ncdu | indexed tree | 60.6 s | +910% | 14k | 0.049 | +873% to +951% | 2.0 MiB |
| GNU `du` | total only | 62.1 s | +955% | 14k | 0.048 | +891% to +1004% | 5.8 MiB |

Positive percentages mean that the competitor took more wall time than its immediately
adjacent fdu run. Displayed values are rounded; raw samples retain full precision.
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

## Measurement Protocol

The publishable local comparison has four phases:

1. Finish and test benchmark-harness changes, then make one immutable release binary.
2. Generate and verify the committed `balanced` recipe at its 1,000,000-entry scale
   point on the internal APFS volume.
3. Run one fixed matrix with 3 full-tree warm-ups and 12 timed adjacent pairs per
   competitor. Alternate whether fdu or the competitor runs first within each pair.
4. Verify the independent pre/post tree fingerprints, per-sample semantic answers,
   summary oracle, baseline, and executable identities before publishing the result.

For this run, the measured corpus, benchmark-owned temporary directory, isolated fdu
cache state, baseline, and raw results all lived on the same internal APFS SSD. Cargo
targets, uv caches, the Python environment, and the copied immutable fdu binary lived on
an external SSD. No build or dependency work ran during measurement.
This split keeps disposable build traffic off the internal drive without moving the
filesystem work being measured away from it.

One matrix used `fdu`’s indexed-tree contract as its anchor and included every work
class. That is appropriate for the README’s orientation table because every competitor
gets its own adjacent fdu control and the table names the work returned.
Optimization verdicts or like-for-like claims should still use separate matrices with
matching work classes.

## Subject and Validity

- **Subject.** The committed `balanced` recipe, seed `fdu-balanced-v1`: 875,000 regular
  files and 125,001 directories.
  Its semantic digest is `4bbd97c0d3d4e2ad`.
- **Binary.** `fdu 0.1.0-dev+g5d7ac7bab`, built from commit `5d7ac7ba`. Its SHA-256 is
  `0b52855f7c5d2397de15fbb71483191ca162667a2774f71c5334aed578595cb7`.
- **Schedule.** Nine competitors, 3 warm-ups per tool, and 12 timed adjacent pairs per
  competitor: 270 tool processes under a fixed-N stopping rule.
- **Correctness.** Zero invalid timed samples, semantic mismatches, or independent
  summary-oracle mismatches; no baseline drift and no tree mutation.
- **Host.** Apple M1 Pro, 8 performance and 2 efficiency cores, 32 GiB, bare-metal
  Darwin 25.5.0, AC power, and normal thermal pressure.
- **Cache state.** Warm-steady after one complete independent fingerprint and at least
  three full-tree warm-ups per tool.
  This means repeated-workload steady state, not guaranteed residency of every APFS
  metadata object.

Exact commands, versions, executable hashes, host and tree facts, raw paired samples,
resource measurements, validity results, and confidence intervals are in the
[comparison result](fdu-live-tool-comparison-result-2026-09-26.json).

## Host Regime and Interpretation

The intended quiet regime rejected three starts because observed load exceeded the
predeclared limit; one earlier attempt was stopped during warm-up before any timed
sample when the host became busy.
The completed run therefore kept the 25% quiet gate unchanged and declared its regime
`uncontrolled`.

CPU busy was 55.44% at the start and 33.37% at the end.
Pairing limits the effect on within-row comparisons because the two processes run
seconds apart, but it cannot turn these absolute seconds into a quiet-host measurement.
The 5.991-second median should not be read as an absolute regression from the
5.206-second result on 2026-09-16: the subject recipe matches, but the competing system
load does not.

The result does answer the narrower question the matrix can support: under the observed
load, fdu remained faster in paired wall time than every measured comparator while
returning its reusable index and tree.
The closest comparator was dumac at +8.2%, with a paired 95% interval of +5.4% to
+16.3%, despite dumac returning only one allocated-byte total.

The previous macOS comparison is the
[2026-09-16 report](report-2026-09-16-fdu-live-tool-comparison.md).
The benchmark entry point and storage rules are in the
[benchmark README](../../../explorations/benchmarks/README.md), and the broader method
is in the [performance loop](../guides/performance-loop.md).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
