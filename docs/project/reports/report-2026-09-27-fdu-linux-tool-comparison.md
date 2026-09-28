# fdu Live Uncached Tool Comparison on Linux

**Date:** 2026-09-27, refreshed 2026-09-28 on the integrated stack

**Status:** Exploratory local performance evidence from a quiet virtualized host

## Outcome

This repeats the
[2026-09-26 macOS comparison](report-2026-09-26-fdu-live-tool-comparison.md) on Linux:
the same generated 1,000,001-entry tree, the same harness, and the same adjacent-pair
schedule. Every Linux peer the harness has an adapter for was measured.
Unlike the macOS run, both matrices held the quiet regime for every sample.

The tables below are the 2026-09-28 refresh, which measured the integrated stack:
[#137](https://github.com/jlevy/fdu/pull/137)’s multi-view reuse on top of the merged
code analysis and presentation work, plus this branch’s H156, built at `7acface5`. The
first run on 2026-09-27 measured `main` at `4c4917f`; its figures are kept under
[First Run](#first-run) for comparison.

The two fdu jobs rank differently here, which is the main finding, and the refresh did
not change it.

- **Summary mode is the fastest tool measured.** `fdu --no-gitignore --view summary`
  answered in a **0.94-second median** (926k files/s). pdu took 8% longer and diskus 12%
  longer.
- **The indexed tree is not.** A fresh process building the reusable exact index and a
  ten-row tree took **1.25 seconds** (699k files/s). pdu and diskus took 21% and 17%
  less time, so fdu takes about 23% longer than pdu.
  Every other peer was slower, from dust at +32% to dua at +212%.

The indexed-tree gap is in user space, not the filesystem.
fdu, pdu, and diskus issue the same system calls: one `statx` per entry, two
`getdents64` and one `openat` per directory.
[Where the Indexed Gap Comes From](#where-the-indexed-gap-comes-from) locates the cost;
[Changes Made in This Pass](#changes-made-in-this-pass) records one that was kept and
two that were not.

### Indexed tree

| Tool | Work class | Median wall-clock time | Wall time vs. fdu | Files/s | GB/s | 95% interval | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| **fdu** | indexed tree | **1.25 s** | baseline | **699k** | **2.4** | — | 322.0 MiB |
| pdu | rendered tree | 1.02 s | −21% | 860k | 2.9 | −23% to −19% | 3.7 MiB |
| diskus | total only | 1.03 s | −17% | 846k | 2.9 | −20% to −16% | 6.1 MiB |
| dust | allocated total | 1.63 s | +32% | 537k | 1.8 | +27% to +34% | 446.1 MiB |
| GNU `du` | total only | 2.53 s | +105% | 346k | 1.2 | +100% to +107% | 1.9 MiB |
| gdu | rendered tree | 2.57 s | +110% | 341k | 1.2 | +103% to +118% | 565.1 MiB |
| ncdu | indexed tree | 2.72 s | +120% | 321k | 1.1 | +112% to +123% | 2.0 MiB |
| dua | total only | 3.82 s | +212% | 229k | 0.78 | +195% to +220% | 14.4 MiB |

### Summary mode

| Tool | Work class | Median wall-clock time | Wall time vs. fdu | Files/s | GB/s | 95% interval | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| **fdu** | transient summary | **0.94 s** | baseline | **926k** | **3.2** | — | 9.9 MiB |
| pdu | rendered tree | 1.02 s | +8% | 860k | 2.9 | +5% to +14% | 3.7 MiB |
| diskus | total only | 1.04 s | +12% | 840k | 2.9 | +6% to +13% | 6.1 MiB |
| dust | allocated total | 1.67 s | +76% | 523k | 1.8 | +67% to +81% | 446.2 MiB |
| GNU `du` | total only | 2.56 s | +162% | 342k | 1.2 | +160% to +172% | 1.9 MiB |
| gdu | rendered tree | 2.66 s | +185% | 329k | 1.1 | +179% to +187% | 550.1 MiB |
| ncdu | indexed tree | 2.75 s | +183% | 318k | 1.1 | +180% to +206% | 2.0 MiB |
| dua | total only | 3.85 s | +303% | 227k | 0.78 | +281% to +324% | 14.4 MiB |

Positive percentages mean the peer took more wall time than its immediately adjacent fdu
run. Files/s divides the subject’s 875,000 regular files by median wall time; GB/s
divides its 2,986,741,760 allocated file bytes, in decimal GB. Both are
metadata-coverage rates, not file-content read bandwidth.

These are not equal-output jobs, and the work-class column says which is which.
fdu’s indexed run also returns counts, apparent and allocated bytes, newest file time,
per-directory roll-ups for the whole tree, and per-extension tallies, and it keeps the
index for the next query.
pdu renders a size tree; diskus returns one number.

## Peak RSS on Linux

The harness reads peak RSS from `wait4`, and on Linux that value is inherited across
`execve`: a child cannot report less than the high-water mark of the process that
launched it, which here is the roughly 57 MiB Python harness.
The first Linux run reported an identical 57.3 MiB for five different tools.
The harness now withholds a peak at or below that floor and renders it as “≤ 57 MiB” in
its own table. The Peak RSS values above for pdu, diskus, GNU `du`, ncdu, dua, and fdu’s
summary mode were instead measured separately by GNU `time`, a small C launcher whose
own floor is under 2 MiB, as the median of three runs each; the raw values are in
[the screen results](fdu-linux-screens-result-2026-09-28.json).
Values above the floor (fdu’s index, dust, gdu) come from the harness.

fdu’s indexed run holds 322 MiB for a million entries: the retained index is the
product. Summary mode retains nothing and stays under 10 MiB (9.9 MiB, by GNU `time`, on
the refreshed build).

## Where the Indexed Gap Comes From

This analysis was made on the first run’s binary; the refresh did not repeat it.
Summary mode and the indexed tree walk the same tree with the same walker, so their
difference is the index.
The indexed run spends 0.4 to 0.5 CPU-seconds more in user time and keeps its four cores
less busy: 3.3 cores on average against 3.8.

**A screen puts most of it in the allocator.** Running the unchanged release binary
under `LD_PRELOAD` with mimalloc, jemalloc, or tcmalloc closed the whole gap: the
indexed tree went from 1.39 to 1.11–1.13 seconds, level with pdu’s 1.10 in the same
`hyperfine` session, and summary mode went from 0.98 to 0.82 seconds.
Forcing glibc onto a single arena (`glibc.malloc.arena_max=1`) made the indexed run 3.3
seconds, so the cost is contention between arenas, not allocation volume alone.

A context-switch profile names the sites.
The index consumer frees buffers that walker threads allocated — each directory’s child
list, and the path key it retires from its directory map — and glibc returns such a
chunk to the arena that allocated it, under that arena’s lock, while the walker is
allocating from the same arena.
glibc’s per-thread cache can then hand those chunks back out on another thread, which
would explain why even walker-side allocations (`DirEntry` names, joined paths) end up
waiting on each other.
On the macOS run this pattern is invisible: the system allocator there does not lock by
owning arena.

**The rest was teardown.** After the report was rendered, the process freed the
million-entry index one allocation at a time on the main thread before exiting: 95 ms of
the 1.39-second run.

What the gap is not: the syscall mix (identical to pdu and diskus under `strace -c`),
the render (33 µs), or dentry lookup (`__d_lookup_rcu` is 27% of samples, but every tool
pays it for the same million `statx` calls).

## Changes Made in This Pass

Three changes were built and measured one at a time against the change before them,
under [the performance loop](../guides/performance-loop.md)’s accept rule: a median at
least 3% better *and* a 95% interval entirely below zero.

| Change | Hypothesis | Probe job, paired | Product CLI indexed tree, paired | Decision |
| --- | --- | --- | --- | --- |
| Release a large one-shot index on a detached thread instead of before the answer returns | [H156](../experiments/exp-160-linux-one-shot-index-release-off-the-answer-path-clears-3-on.md) | `default-tree` −3.19% [−4.88%, −1.79%] | −4.31% [−5.99%, −3.25%] | **Kept** |
| Fold each file straight into its parent’s roll-up and move, not copy, each listed name | [H157](../experiments/exp-161-linux-direct-file-fold-and-owned-names-miss-3-on-cold-scan-i.md) | `cold-scan-index` −2.22% [−4.04%, +0.04%] | −3.71% [−4.71%, −1.97%] | Rejected on its pre-registered probe job |
| Hold leaf-only listings until a batch fills instead of waking the index consumer per chunk | [H158](../experiments/exp-162-linux-detached-leaf-listing-hold-cuts-futex-wakes-but-not-wa.md) | `cold-scan-index` +0.88% [−0.17%, +1.94%] | With H157: −4.30% [−5.52%, −1.38%] against H157 alone at −3.71%; no separable effect | Rejected |

The kept change removes the 95 ms teardown from every large one-shot report and from the
join a default `fdu PATH` waits on; `cold-scan-index`, which frees its index inside the
timed region, was the placebo and did not move.
On the integrated stack, a paired screen of the builds with and without H156 (#137 at
`326b014b` against `7acface5`, ten runs per arm in both orders) moved the pooled mean of
the indexed tree −6.9% and of the default `fdu . --cache off` −7.2%; summary mode, which
builds no index, moved +0.5%. Each build’s 20 samples are in
[the screen results](fdu-linux-screens-result-2026-09-28.json).

H157 cut allocations from 7.0 million to 4.3 million and cleared the rule on the product
job, but not on the probe job it named beforehand, so it is not kept; a rerun with the
product job pre-registered is `fdu-o6um`. H158 cut the consumer’s `futex` wakes from
106k to 18k without moving wall time: on four cores the walkers, not the consumer, set
the pace.

The allocator result is the open item, H159 (`fdu-578e`).
[H74](../guides/performance-loop.md#index-and-allocation) found mimalloc neutral on the
index tier before the detached builder existed and declined the dependency, and H85
found that recycling buffers recovered only part of the aggregate tier’s cost.
The screen above reopens the question for the index tier on Linux, and the
context-switch profile names what a dependency-free fix must remove: the consumer
freeing walker-allocated child lists and path keys.

## Measurement Protocol

1. Build one release binary, and copy it outside the measured tree: the merged `main` at
   `4c4917f` for the first run, and the integrated stack at `7acface5` for the refresh.
2. Generate the committed `balanced` recipe at its 1,000,000-entry scale point, seed
   `fdu-balanced-v1`, and verify it against its manifest.
3. Run two matrices, one anchored on fdu’s indexed-tree contract and one on its
   transient-summary contract, each with 3 full-tree warm-ups per tool and 12 timed
   adjacent pairs per peer, alternating which process runs first.
4. Check the pre- and post-run fingerprints, every sample’s semantic answer, the
   independent summary oracle, and executable identities before publishing.

The measured tree, the harness’s temporary directory, isolated fdu cache state, and raw
results lived on the same ext4 filesystem.
Builds used a separate target directory, and nothing else ran during measurement.

## Subject and Validity

- **Subject.** The committed `balanced` recipe, seed `fdu-balanced-v1`: 875,000 regular
  files and 125,001 directories, semantic digest `4bbd97c0d3d4e2ad` — the same digest as
  the macOS subject.
- **Binary.** Refresh: `fdu 0.1.0-dev+g7acface52`, SHA-256
  `9e0e140ed6b7f63a00929723ab6777791093192c0d22653d92cf829904c98cf2`. First run:
  `fdu 0.1.0-dev+g4c4917f4f.dirty`, SHA-256
  `22572dd84b850ce17bb89c4f48a91a36b9dcc31e76b60891a2afbfdfe6ad45af`. The `.dirty`
  suffix comes from `.tbd/config.yml`, which `tbd` rewrote before the build; no source
  file differed from `4c4917f`.
- **Peers.** diskus 0.9.0, dua 2.41.1, dust 1.2.4, and pdu 0.24.0 built with
  `cargo install --locked`; gdu v5.36.1 built with `go install`; ncdu 1.19 and GNU
  coreutils `du` 9.4 from Ubuntu 24.04. Executable hashes are in the result files.
- **Schedule.** Seven peers per matrix, 3 warm-ups per tool, 12 timed adjacent pairs per
  peer: 210 tool processes per matrix under a fixed-N stopping rule.
- **Correctness.** Zero invalid timed samples, semantic mismatches, or summary-oracle
  mismatches in any of the four matrices; no baseline drift and no tree mutation.
- **Host.** 4-vCPU Intel Xeon at 2.1 GHz, 15.7 GiB, Linux 6.18.44 in a Firecracker KVM
  guest, ext4 on a virtio block device.
- **Regime.** Quiet: every sample was taken with the host under 25% CPU busy, measured
  over one second before and after it.
  Cache state was warm-steady after one complete independent fingerprint and at least
  three full-tree warm-ups per tool.

The exact commands, versions, hashes, host facts, raw paired samples, and confidence
intervals are in the refresh’s
[indexed-tree result](fdu-linux-tool-comparison-result-2026-09-28-indexed.json) and
compressed
[summary-mode result](fdu-linux-tool-comparison-result-2026-09-28-summary.json.gz), and
the first run’s
[indexed-tree result](fdu-linux-tool-comparison-result-2026-09-27-indexed.json) and
[summary-mode result](fdu-linux-tool-comparison-result-2026-09-27-summary.json.gz).
The summary results carry per-sample scan-policy traces, 4.5 MB uncompressed.

### First Run

The 2026-09-27 run measured `main` at `4c4917f` under the same protocol.
Its peers ran slower than in the refresh, pdu at 1.12 s against 1.02 s, so host state
moved between the two runs as well as fdu’s code; the rankings are the same.

| Job | fdu | pdu | diskus | dust | Slowest peer |
| --- | ---: | ---: | ---: | ---: | ---: |
| Indexed tree | 1.38 s | −19% | −18% | +25% | dua +190% |
| Summary mode | 0.97 s | +11% | +16% | +77% | dua +299% |

## Interpretation and Limits

Both matrices ran fdu with `--cache off`, as the macOS comparison did.
The default `fdu .` on this build also writes a metadata snapshot.
On the refreshed build that added 0.24 s when an earlier run’s snapshot already existed
(1.51 against 1.27 s) and about 0.5 s on a first run into an empty cache directory,
where the kernel must allocate the pages; the
[cache economics brief](../research/research-2026-09-27-cache-economics-and-default-plans.md)
measures that cost, explains the macOS and Linux rankings from their CPU split, and
proposes defaults. Summary mode ran with `--no-gitignore` because reading ignore rules
makes the planner keep the full index; without the flag, a screen of the same tree took
1.25 s with the cache off.

A screen made before the refresh compared the first run’s binary with an integrated
build and suggested the indexed-tree gap to pdu and diskus had narrowed to about 12%.
The refresh measured every peer again in the same matrices and found them faster too, so
the gap held at 21% and 17%.

A virtualized host is the common deployment case for Linux and a valid regime for warm
measurements; it cannot say anything about device latency, so no cold claim is made
here. [The platform tuning guide](../guides/platform-tuning.md#host) explains the
distinction.

This host has four cores, and the ranking may move with core count: fdu starts at most
six walker workers plus one index consumer, while pdu and diskus size their pools to the
machine.
[The macOS run](report-2026-09-26-fdu-live-tool-comparison.md) had ten cores and
an allocator that does not show the contention described above.

Three gaps remain in coverage.
dut, which the [design principles](../architecture/fdu-design-principles.md) name as a
Linux reference, has no verified harness adapter, and its source host was unreachable
from this environment.
ncdu is the 1.x C release from Ubuntu rather than the 2.x release measured on macOS. And
a frozen real tree, not only the generated one, is still needed before generalizing.

The harness needed four fixes to run on Linux at all, all in
`explorations/benchmarks/realtree/`: a `/proc/stat` CPU-occupancy gate in place of load
average, which counted the benchmark’s own previous samples and invalidated all of them;
directory blocks in the dust oracle, since ext4 charges each directory a block and APFS
does not; the `fdu.report/7` envelope for the summary contract; and the peak-RSS floor
above.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
