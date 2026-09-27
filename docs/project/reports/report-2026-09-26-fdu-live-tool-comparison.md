# fdu Live Uncached Tool Comparison on macOS

**Date:** 2026-09-26

**Status:** Exploratory local performance evidence from an uncontrolled host

## Outcome

On a reproducible generated tree of 1,000,001 entries, a fresh fdu process with its
persisted cache disabled built a reusable exact index and rendered a depth-one, ten-row
tree in a **5.991-second median**. That is **146,050 files/s** and **0.499 allocated
GB/s** on this subject.

The same matrix measured every available baseline requested by the comparison harness.
Each competitor ran immediately beside fdu with alternating order, so the relative
figures are paired even though the host was not quiet.

| Tool | Work class | Median wall | Files/s | Allocated GB/s | Versus paired fdu | 95% interval | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| **fdu** | indexed tree | **5.991 s** | **146,050** | **0.499** | baseline | — | 285.9 MiB |
| dumac | total only | 6.333 s | 138,169 | 0.472 | +8.2% | +5.4% to +16.3% | 29.6 MiB |
| diskus | total only | 8.653 s | 101,124 | 0.345 | +45.1% | +31.7% to +48.8% | 10.8 MiB |
| pdu | rendered tree | 9.246 s | 94,640 | 0.323 | +53.1% | +48.9% to +60.4% | 12.0 MiB |
| dust | allocated total | 9.604 s | 91,112 | 0.311 | +59.5% | +55.2% to +67.5% | 545.9 MiB |
| dua | total only | 9.746 s | 89,783 | 0.306 | +63.5% | +50.7% to +70.8% | 21.2 MiB |
| gdu | rendered tree | 10.390 s | 84,217 | 0.287 | +64.1% | +55.3% to +82.6% | 510.4 MiB |
| BSD `du` | total only | 49.341 s | 17,734 | 0.061 | +716.9% | +611.2% to +742.1% | 1.2 MiB |
| ncdu | indexed tree | 60.560 s | 14,448 | 0.049 | +909.7% | +873.4% to +951.4% | 2.0 MiB |
| GNU `du` | total only | 62.118 s | 14,086 | 0.048 | +954.6% | +891.3% to +1003.5% | 5.8 MiB |

Positive percentages mean that the competitor took more wall time than its immediately
adjacent fdu run. Files/s divides the subject’s 875,000 regular files by median wall
time.
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
