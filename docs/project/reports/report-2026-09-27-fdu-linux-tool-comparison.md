# fdu Live Uncached Tool Comparison on Linux

**Date:** 2026-09-27

**Status:** Exploratory local performance evidence from a quiet virtualized host

## Outcome

This repeats the
[2026-09-26 macOS comparison](report-2026-09-26-fdu-live-tool-comparison.md) on Linux:
the same generated 1,000,001-entry tree, the same harness, and the same adjacent-pair
schedule. Every Linux peer the harness has an adapter for was measured.
Unlike the macOS run, both matrices held the quiet regime for every sample.

The two fdu jobs rank differently here, which is the main finding.

- **Summary mode is the fastest tool measured.** `fdu --no-gitignore --view summary`
  answered in a **0.97-second median** (907k files/s), 11% ahead of pdu and 16% ahead of
  diskus.
- **The indexed tree is not.** A fresh process building the reusable exact index and a
  ten-row tree took **1.38 seconds** (635k files/s). pdu and diskus were 19% and 18%
  faster. Every other peer was slower, from dust at +25% to dua at +190%.

The indexed-tree gap is in user space, not the filesystem.
fdu, pdu, and diskus issue the same system calls: one `statx` per entry, two
`getdents64` and one `openat` per directory.
[Where the Indexed Gap Comes From](#where-the-indexed-gap-comes-from) locates the cost;
[Changes Made in This Pass](#changes-made-in-this-pass) records one that was kept and
two that were not.

### Indexed tree

| Tool | Work class | Median wall-clock time | Wall time vs. fdu | Files/s | GB/s | 95% interval | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| **fdu** | indexed tree | **1.38 s** | baseline | **635k** | **2.2** | — | 318.6 MiB |
| pdu | rendered tree | 1.12 s | −19% | 783k | 2.7 | −24% to −17% | 3.8 MiB |
| diskus | total only | 1.12 s | −18% | 780k | 2.7 | −20% to −17% | 6.0 MiB |
| dust | allocated total | 1.69 s | +25% | 517k | 1.8 | +23% to +27% | 446.1 MiB |
| gdu | rendered tree | 2.67 s | +92% | 328k | 1.1 | +85% to +94% | 565.8 MiB |
| GNU `du` | total only | 2.67 s | +98% | 328k | 1.1 | +95% to +101% | 1.9 MiB |
| ncdu | indexed tree | 2.92 s | +109% | 300k | 1.0 | +105% to +115% | 2.0 MiB |
| dua | total only | 3.95 s | +190% | 221k | 0.76 | +185% to +196% | 14.1 MiB |

### Summary mode

| Tool | Work class | Median wall-clock time | Wall time vs. fdu | Files/s | GB/s | 95% interval | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| **fdu** | transient summary | **0.97 s** | baseline | **907k** | **3.1** | — | 9.7 MiB |
| pdu | rendered tree | 1.09 s | +11% | 803k | 2.7 | +8% to +13% | 3.8 MiB |
| diskus | total only | 1.10 s | +16% | 795k | 2.7 | +13% to +17% | 6.0 MiB |
| dust | allocated total | 1.70 s | +77% | 516k | 1.8 | +71% to +81% | 446.2 MiB |
| gdu | rendered tree | 2.62 s | +166% | 334k | 1.1 | +166% to +175% | 573.2 MiB |
| GNU `du` | total only | 2.67 s | +174% | 328k | 1.1 | +157% to +186% | 1.9 MiB |
| ncdu | indexed tree | 2.85 s | +200% | 307k | 1.0 | +194% to +203% | 2.0 MiB |
| dua | total only | 3.96 s | +299% | 221k | 0.75 | +290% to +319% | 14.1 MiB |

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
own floor is under 2 MiB, as the median of three runs each.
Values above the floor (fdu’s index, dust, gdu) come from the harness.

fdu’s indexed run holds 319 MiB for a million entries: the retained index is the
product. Summary mode retains nothing and stays under 10 MiB.

## Where the Indexed Gap Comes From

Summary mode and the indexed tree walk the same tree with the same walker, so their
difference is the index.
The indexed run spends 0.4 to 0.5 CPU-seconds more in user time and keeps its four cores
less busy: 3.3 cores on average against 3.8.

**The allocator is most of it.** Running the unchanged release binary under `LD_PRELOAD`
with mimalloc, jemalloc, or tcmalloc closed the whole gap: the indexed tree went from
1.39 to 1.11–1.13 seconds, level with pdu’s 1.10 in the same `hyperfine` session, and
summary mode went from 0.98 to 0.82 seconds.
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
| Release a large one-shot index on a detached thread instead of before the answer returns | [H152](../experiments/exp-158-linux-one-shot-index-release-off-the-answer-path-clears-3-on.md) | `default-tree` −3.19% [−4.88%, −1.79%] | −4.31% [−5.99%, −3.25%] | **Kept** |
| Fold each file straight into its parent’s roll-up and move, not copy, each listed name | [H153](../experiments/exp-159-linux-direct-file-fold-and-owned-names-miss-3-on-cold-scan-i.md) | `cold-scan-index` −2.22% [−4.04%, +0.04%] | −3.71% [−4.71%, −1.97%] | Rejected on its pre-registered probe job |
| Hold leaf-only listings until a batch fills instead of waking the index consumer per chunk | [H154](../experiments/exp-160-linux-detached-leaf-listing-hold-cuts-futex-wakes-but-not-wa.md) | `cold-scan-index` +0.88% [−0.17%, +1.94%] | With H153: −4.30% [−5.52%, −1.38%] against H153 alone at −3.71%; no separable effect | Rejected |

The kept change removes the 95 ms teardown from every large one-shot report and from the
join a default `fdu PATH` waits on; `cold-scan-index`, which frees its index inside the
timed region, was the placebo and did not move.

H153 cut allocations from 7.0 million to 4.3 million and cleared the rule on the product
job, but not on the probe job it named beforehand, so it is not kept; a rerun with the
product job pre-registered is `fdu-o6um`. H154 cut the consumer’s `futex` wakes from
106k to 18k without moving wall time: on four cores the walkers, not the consumer, set
the pace.

The allocator result is the open item, H155 (`fdu-578e`).
[H74](../guides/performance-loop.md#index-and-allocation) found mimalloc neutral on the
index tier before the detached builder existed and declined the dependency, and H85
found that recycling buffers recovered only part of the aggregate tier’s cost.
The screen above reopens the question for the index tier on Linux, and the
context-switch profile names what a dependency-free fix must remove: the consumer
freeing walker-allocated child lists and path keys.

## Measurement Protocol

1. Build one release binary from the merged `main` at `4c4917f`, and copy it outside the
   measured tree.
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
- **Binary.** `fdu 0.1.0-dev+g4c4917f4f.dirty`, SHA-256
  `22572dd84b850ce17bb89c4f48a91a36b9dcc31e76b60891a2afbfdfe6ad45af`. The `.dirty`
  suffix comes from `.tbd/config.yml`, which `tbd` rewrote before the build; no source
  file differed from `4c4917f`.
- **Peers.** diskus 0.9.0, dua 2.41.1, dust 1.2.4, and pdu 0.24.0 built with
  `cargo install --locked`; gdu v5.36.1 built with `go install`; ncdu 1.19 and GNU
  coreutils `du` 9.4 from Ubuntu 24.04. Executable hashes are in the result files.
- **Schedule.** Seven peers per matrix, 3 warm-ups per tool, 12 timed adjacent pairs per
  peer: 210 tool processes per matrix under a fixed-N stopping rule.
- **Correctness.** Zero invalid timed samples, semantic mismatches, or summary-oracle
  mismatches in either matrix; no baseline drift and no tree mutation.
- **Host.** 4-vCPU Intel Xeon at 2.1 GHz, 15.7 GiB, Linux 6.18.44 in a Firecracker KVM
  guest, ext4 on a virtio block device.
- **Regime.** Quiet: every sample was taken with the host under 25% CPU busy, measured
  over one second before and after it.
  Cache state was warm-steady after one complete independent fingerprint and at least
  three full-tree warm-ups per tool.

The exact commands, versions, hashes, host facts, raw paired samples, and confidence
intervals are in the
[indexed-tree result](fdu-linux-tool-comparison-result-2026-09-27-indexed.json) and the
compressed
[summary-mode result](fdu-linux-tool-comparison-result-2026-09-27-summary.json.gz),
whose per-sample scan-policy traces make it 4.5 MB uncompressed.

## Interpretation and Limits

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
