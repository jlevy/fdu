# fdu Performance Measurements

This page has the measurements behind the README’s [Speed](../README.md#speed) section:
each run’s engine, date, host, intervals, and memory, what each tool returns, and the
throughput figures the README rounds.
The experiment records under [project/experiments](project/experiments/) hold the raw
evidence, and
[the performance evidence report](project/reports/report-2026-08-20-fdu-performance-evidence.md)
is the full record of every experiment the project has run.

## How the Runs Are Measured

- **One-shot reports.** Each tool runs its own command once per sample.
  fdu runs its default command, `fdu PATH`, which neither reads nor writes its cache, so
  a repeated run costs the same.
  The one exception is the repeated source-line count, which measures that cache.
- **Warm caches.** The filesystem cache holds the tree before timing starts.
  Each cell discards its first runs as warm-ups.
- **Paired and interleaved.** Each tool’s run alternates with an adjacent fdu run.
  A percentage is the median of those paired differences, with its 95% bootstrap
  interval. +25% means 1.25× as long as fdu.
- **Same answer.** The harness checks every run’s answer and counts any tool whose total
  disagrees with fdu’s as a semantic mismatch; every run here had none.
  Source-line counters recognize different languages, so there the harness requires
  instead that each tool give one answer across all its samples.
- **Different work.** The tools do not return the same thing, so each table names what
  each one returns. fdu’s default tree reads every `.gitignore` in the tree to label
  ignored shares, which no other tool here does.

## Throughput

The README rounds these rates.

### Disk Usage

Each rate divides the generated million-entry tree’s 875,000 regular files, or its
2,986,741,760 allocated bytes, by fdu’s median wall time for its default report.
Both are metadata rates: sizing a file reads none of its contents.

| Platform | fdu build | Median | Files/s | GB/s |
| --- | --- | ---: | ---: | ---: |
| Linux | 0.3.0 ([exp-202](project/experiments/exp-202-linux-the-0-3-0-release-end-to-end-the-default-tree-48-faste.md)) | 0.951 s | 920k | 3.1 |
| macOS | pre-0.2.0 (`a5c0ab46`) | 6.4 s | 137k | 0.47 |

The runs behind each row are under [Linux](#linux) and [macOS](#macos).

### Source-Line Counting

On 2026-09-30, on the Linux host below, the published 0.3.0 wheel
(`fdu-0.3.0-cp312-abi3-manylinux_2_17_x86_64.manylinux2014_x86_64.whl`) was installed
with `uv tool install` and run through its `fdu` command, a Python launcher for the
native extension, so each time includes Python’s startup.
The subject was a copy of the Linux v6.12 source without `.git`, 86,618 files and
1,476,498,481 bytes, measured as the
[SLOC tools survey](project/research/research-2026-09-29-sloc-tools-survey.md#methodology)’s
“ignore rules off” arm: 3 warm-ups and 12 adjacent pairs per peer.

| Tool and run | Median | Files/s | GB/s read | CPU time | Peak RSS |
| --- | ---: | ---: | ---: | ---: | ---: |
| **fdu** `--analyze=code --cache=off`, first run | **8.20 s** | **10.6k** | **0.18** | 31.0 s | 110 MiB |
| **fdu** `--analyze=code`, repeated with its cache | **0.55 s** | **158k** | — | 0.72 s | 138 MiB |
| scc 4.1.0 | 1.37 s |  |  | 5.3 s | 233 MiB |
| tokei 15.0.0 | 2.17 s |  |  | 8.4 s | 152 MiB |

Against the adjacent fdu run, scc took 83% less time [−84%, −83%] and tokei 73% less
[−74%, −73%] than fdu’s first run, and 154% more [+148%, +158%] and 297% more
[+285%, +311%] than its repeated run.
A repeated run reads no contents when no file has changed: it revalidates each file’s
metadata and serves the counts it cached, so it has no read rate.
fdu counted 26,312,541 code lines in 61,451 files of the languages it recognizes, one
file and six lines fewer than the survey’s engine, which took a Documentation config
file for C. The survey measured that earlier engine at 7.92 s against scc’s 1.24 s, in
another session. The harness documents are in the evidence directory:
[first run](project/research/evidence/sloc-tool-comparison-2026-09-30-0.3.0-no-ignore.json.gz)
and
[repeated](project/research/evidence/sloc-tool-comparison-2026-09-30-0.3.0-cached.json.gz).

## Linux

The host is a 4-vCPU virtual machine (Firecracker, kernel 6.18) on ext4, otherwise
quiet.

### The 0.3.0 Engine, Three Trees

On 2026-09-30, 20 pairs per tool per tree
([exp-202](project/experiments/exp-202-linux-the-0-3-0-release-end-to-end-the-default-tree-48-faste.md)):

| Tree | fdu 0.3.0 | fdu 0.2.1 | pdu | pdu `--max-depth 2` | diskus |
| --- | ---: | ---: | ---: | ---: | ---: |
| Linux v6.12 source, 92k entries | **0.110 s** | 0.215 s, +94.7% [+85.8%, +112.4%] | 0.124 s, +15.3% [+12.0%, +19.3%] | 0.118 s, +9.7% [+1.8%, +12.2%] | 0.123 s, +16.8% [+3.3%, +20.5%] |
| `node_modules`, 80k entries | **0.106 s** | 0.126 s, +20.5% [+18.3%, +22.4%] | 0.126 s, +18.2% [+15.2%, +23.0%] | 0.119 s, +11.5% [+1.9%, +18.9%] | 0.117 s, +12.1% [+9.3%, +16.5%] |
| generated, 1M entries | **0.951 s** | 1.131 s, +20.6% [+17.9%, +21.7%] | 1.196 s, +25.3% [+20.5%, +27.7%] | 1.125 s, +18.8% [+15.0%, +19.7%] | 1.171 s, +23.9% [+20.4%, +28.4%] |

What each returns:

- **fdu:** its default tree, five levels deep with a 1% share floor, and every
  `.gitignore` read to label ignored shares.
- **pdu:** a ten-level tree with a 1% floor; at `--max-depth 2`, the root and its
  children.
- **diskus:** one total.

The release engine’s default tree holds 57.3 MiB at its peak on the million-entry tree,
where 0.2.1 held 292.7 MiB.

**Host regime.** On the real trees every tool’s median was 29% to 38% above its figure
in exp-201, earlier the same day, and 0.2.1 was level with `pdu --max-depth 2` on the
million-entry tree, where earlier runs had pdu about 11% ahead of it.
So about ten points of fdu’s 19% lead there are probably the session rather than fdu.
On the real trees,
[exp-201](project/experiments/exp-201-linux-the-pdu-track-end-to-end-the-default-tree-3-and-9-fast.md)
measured the leads at 13% and 15% over pdu, 3% and 10% over `pdu --max-depth 2`, and 12%
and 11% over diskus.

### Every Tool, the Million-Entry Tree

On 2026-09-29, the same host and tree, with the engine that preceded the release’s last
performance changes (`ebc06c78`):

| Tool | Work returned | Median | Time vs. fdu | Peak RSS |
| --- | --- | ---: | ---: | ---: |
| **fdu** | default tree | **1.09 s** | baseline | 58 MiB |
| pdu `--max-depth 2` | the root and its children | 1.06 s | −3% | ≤ 54 MiB |
| pdu | ten-level tree, 1% floor | 1.14 s | +4% | 93 MiB |
| diskus | one total | 1.16 s | +7% | ≤ 54 MiB |
| dust | one allocated-byte total | 1.75 s | +62% | 446 MiB |
| gdu | ten largest files | 2.81 s | +158% | 596 MiB |
| GNU `du` 9.4 | one total, serial | 2.85 s | +160% | ≤ 54 MiB |
| ncdu 1.19 | full-tree JSON export to `/dev/null` | 3.00 s | +174% | ≤ 54 MiB |
| dua | the root’s children and a total | 3.65 s | +234% | ≤ 54 MiB |

Every interval excludes zero.
A peak of ≤ 54 MiB is a bound, not a measurement: Linux carries a process’s peak memory
across `exec`, so a tool smaller than the harness reports the harness’s peak.
The README’s Linux column takes pdu and diskus from the 0.3.0 run above and the other
tools from this run.
The [Linux comparison](project/reports/report-2026-09-27-fdu-linux-tool-comparison.md)
has versions, CPU time, and the protocol.

## macOS

On 2026-09-28, an M1 Pro’s internal APFS SSD, the same generated million-entry tree,
with a pre-0.2.0 build of fdu (`a5c0ab46`) under heavy background load.
That build made a reusable index and rendered a ten-row tree; its default `fdu PATH`
measured the same within 0.1%. No macOS comparison has run on the current engine; the
planned cells are exp-203 onward in the
[platform review](project/research/research-2026-09-29-platform-review-of-the-linux-round.md).

| Tool | Work returned | Median | Time vs. fdu | Files/s | GB/s |
| --- | --- | ---: | ---: | ---: | ---: |
| **fdu** | reusable exact index and ten-row tree | **6.4 s** | baseline | **137k** | **0.47** |
| dumac | allocated-byte total | 6.9 s | +9% | 127k | 0.43 |
| diskus 0.9.0 | one total | 9.3 s | +42% | 94k | 0.32 |
| pdu 0.24.0 | the root’s total (`--max-depth 1`) | 9.2 s | +49% | 96k | 0.33 |
| dust 1.2.4 | allocated-byte total | 11.0 s | +57% | 79k | 0.27 |
| dua 2.41.1 | the root’s children and a total | 10.4 s | +61% | 84k | 0.29 |
| gdu 5.36.1 | ten largest files | 10.5 s | +67% | 83k | 0.28 |
| BSD `du` | one total, serial | 55.6 s | +801% | 16k | 0.054 |
| GNU `du` 9.9 | one total, serial | 68.0 s | +960% | 13k | 0.044 |
| ncdu 2.9.2 | full-tree JSON export to `/dev/null` | 67.3 s | +968% | 13k | 0.044 |

Percentages are paired medians, so their order can differ from the medians’.
Rates count regular files and their disk space, not file-content reads; `k` means
thousands and GB is decimal.
The [macOS comparison](project/reports/report-2026-09-26-fdu-live-tool-comparison.md)
has methodology, memory use, confidence intervals, and exact results.

## Windows

Windows builds and passes the test suite in CI, and has no performance measurements.

## Multi-View Reports

Building several reports from one index has its own result.
In an exploratory macOS benchmark on a 137,085-entry tree, building the unfiltered
Types, Families, Languages, and Documents views 100 times from an already line-analyzed
index took 12.0 s, against 29.9 s before shared metric resolution: about 120 ms instead
of 299 ms per report
([exp-159](project/experiments/exp-159-share-content-metric-resolution-across-views.md)).
It applies only to unfiltered requests with several metric views; the default disk-usage
command and single-view analysis are unaffected.
A quiet confirming run failed to qualify on a loaded host, so the result stands as
exploratory; it predates the 0.3.0 engine, and its Linux magnitude is unmeasured.

## When the Cache Pays

Every figure above except the repeated source-line count is a one-shot report.
The
[cache economics brief](project/research/research-2026-09-27-cache-economics-and-default-plans.md)
covers when fdu’s snapshot cache and a retained index pay on each platform.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
