# Research: First Linux Measurements — Syscall Convergence, the Exposed Consumer, and the Warm-Open Inversion

**Date:** 2026-08-13

**Author:** fdu project, with Claude Code review assistance

**Status:** Proposed

## Overview

Every published fdu product number is from one M1 Pro on APFS. The
[architecture white paper](../reports/report-2026-08-12-fdu-performance-architecture.md)
says Linux “may produce a different ranking”; this note reports the first paired Linux
measurements and the ranking is indeed different, in both directions.
It was produced during the senior review of PR #8, on a disposable Linux VM, so these
are **scouting results under the caveats below**, not release evidence.
They exist to order the Linux work, not to be quoted as product numbers.

Three findings matter more than the individual timings:

1. **The Linux syscall layer is already converged, across tools and inside fdu.** Strace
   shows fdu, dut, and diskus issue an essentially identical mix — one `statx` per
   entry, ~2 `getdents64` per directory, one `openat`+`close` per directory — and a
   single-threaded C harness shows raw `getdents64`, `statx` with narrow masks, and
   glibc `readdir`+`fstatat` are all within noise of each other warm.
   Rust’s standard library already emits `statx(dirfd, name)` and `getdents64` on Linux,
   so the “replace `read_dir` with `getdents64`+`statx`” goal in the design principles
   is **already true in the syscalls** — there is no dut-style enumeration advantage
   left to take on the warm path.
2. **What Linux exposes is the index consumer, not the producer.** With identical kernel
   work and no bulk-metadata API to hide behind, fdu’s full-index tree run was ~3.3×
   slower than dut and pdu and ~1.5× slower than dust on the same tree, with ~20× dut’s
   user CPU per entry and ~25× its peak RSS. The queued consumer work (H60 worker-local
   subtrees, H19–H22 compact entries, H66 directory-only transient tree) is therefore
   the *primary* Linux program, not a refinement.
   The transient summary plan, which skips the index, **tied diskus** — evidence the
   architecture is right and the index construction cost is the whole Linux gap.
3. **The verified warm open is inverted on Linux: the cache made fdu slower.** Warm open
   measured **+72% versus a cold scan** at 450k entries.
   Load rebuilds the index through the full apply path (O(N·D) roll-up merges, one
   `PathBuf` per record), reconciliation re-stats every entry at the same price as a
   cold producer pass, and `spawn_save` then deep-clones the index and rewrites a
   byte-equivalent snapshot even when nothing changed.
   H9 — closed on macOS by bulk metadata and reconciliation waves — is alive on Linux.

## Environment and Method

All caveats up front, per the reporting rules in
[the performance loop](../guides/performance-loop.md):

- **Host:** 4-vCPU Intel Xeon VM (2.8 GHz), 15 GiB RAM, kernel 6.18.5, ext4 on a virtio
  disk, running as root.
  Virtualized storage means guest-cold reads may still hit host caches; cold numbers
  order strategies but do not measure real device latency.
  Root means permission-denied paths never fail (which also surfaced two fixture bugs;
  see PR notes).
- **Subject:** generated heterogeneous tree, 450,462 entries (28,629 directories,
  421,690 files, 143 symlinks), node_modules-heavy with src trees, sparse pack files,
  and assets; ~3.0 GiB apparent, ~748 MB allocated (file blocks).
  Byte semantics across tools reconcile exactly: du-class totals exceed fdu’s files-only
  totals by 28,629 × 4 KiB of directory inode blocks.
- **Method:** adjacent paired trials with alternating order, 10 pairs warm and 6 pairs
  cold per matchup, wall via monotonic clock around spawn+wait, rusage via `wait4`,
  paired medians with 4,000-resample bootstrap 95% intervals.
  Warm runs followed explicit full-tree warmups; each cold sample ran `sync` and wrote
  `3` to `/proc/sys/vm/drop_caches` (the controlled-cold preparation the Linux plan
  specifies). Every walker variant reproduced identical file/dir/byte tallies before
  timing.
- **Binaries:** fdu from PR #8 head (release, all features); dut `68d4ba2` built locally
  with `-O3` (the exact revision the frontier research audited); dust, pdu v0.24.0, dua
  v2.41.1, diskus from crates.io release builds.
  dut defaults to a hardcoded 4 threads, which happens to equal this host’s cores.

The single-file C harness (`benchmarks/spikes/walkspike.c`) isolates the enumeration and
metadata layer with interchangeable strategies over one BFS walk: glibc
`readdir`+`fstatat`; raw 256 KiB `getdents64` with `fstatat`, `statx`
(`STATX_BASIC_STATS`), and a narrow mask; a files-only tier that skips `statx` for
`d_type`-known directories and symlinks; an inode-sorted `statx` order; and a
hand-rolled io_uring `IORING_OP_STATX` ring at queue depth 128.

## Warm results

### The enumeration layer has no headroom warm

Single-threaded variants against the raw-`getdents64`+`statx` baseline (positive =
variant slower):

| Variant vs `statx` baseline | Wall change | 95% interval | Note |
| --- | ---: | --- | --- |
| glibc `readdir` + `fstatat` | −0.4% | [−3.3%, +2.7%] | std’s exact shape; no penalty |
| raw `getdents64` + `fstatat` | +1.6% | [−3.3%, +4.9%] | buffer size irrelevant at this fan-out |
| narrow `statx` mask | +1.2% | [−0.7%, +4.8%] | masks don’t skip local-fs work |
| files-only `statx` (summary tier) | −1.4% | [−2.4%, +1.9%] | saves 6.4% of stats here; larger on dir-heavy trees |
| inode-sorted `statx` | **+6.8%** | [+2.8%, +13.7%] | sort cost, no warm benefit; its claim is cold-only |
| io_uring `statx`, QD 128 | **+327%** | [+309%, +345%] | 4.4× CPU; io-wq punting; decisively refuted warm |

The baseline itself runs ~1.5 µs/entry single-threaded, ~91% system CPU: the warm Linux
floor is per-entry `statx` kernel time, and no enumeration rearrangement moves it.
This independently reproduces dut’s own “~2% getdents64 vs readdir” history and extends
the macOS exp-041/exp-044 conclusion to Linux: the floor is kernel entry metadata, and
only *doing fewer stats* (graded tiers) or *hiding latency* (threads) changes it.

### Syscall census (strace -c, full tree)

| Tool | statx | getdents64 | openat | close | notable extras |
| --- | ---: | ---: | ---: | ---: | --- |
| fdu `--cache off` (tree) | 450,471 | 57,260 | 28,641 | 28,641 | 11.9k futex |
| fdu `--cache off --view summary` | 450,471 | 57,260 | 28,641 | 28,641 | 9.2k futex |
| dut | 450,462 | 57,260 | 28,632 | 28,632 | 27 futex |
| diskus | 450,467 | 57,260 | 28,638 | 28,638 | **27,033 sched_yield** |

One `statx` per entry, everywhere.
fdu’s std-based portable walker is already syscall-optimal on Linux; there is no missing
“getdents64+statx layer.”

### Product comparisons

| Matchup (A vs B) | A wall | B wall | B vs A | 95% CI | A CPU / B CPU | A RSS / B RSS |
| --- | ---: | ---: | ---: | --- | --- | --- |
| fdu-tree vs dut | 1,340 ms | **408 ms** | −69.3% | [−75.7%, −67.5%] | 1.93 s / 0.64 s | 285 / 11.5 MiB |
| fdu-tree vs pdu | 1,010 ms | **307 ms** | −69.7% | [−70.1%, −69.1%] | 1.93 s / 1.19 s | 279 / 11.6 MiB |
| fdu-tree vs dust | 1,289 ms | **863 ms** | −29.1% | [−42.1%, −12.3%] | 1.96 s / 1.65 s | 282 / 175 MiB |
| fdu-summary vs diskus | 302 ms | 296 ms | −1.8% | [−4.6%, −0.5%] | 1.10 s / 1.17 s | 50 / 11.6 MiB |
| fdu-summary vs dua | 329 ms | 886 ms | **+178%** | [+162%, +189%] | 1.19 s / 2.19 s | 46 / 11.6 MiB |
| fdu-tree vs fdu-warm | 1,020 ms | 1,761 ms | **+71.8%** | [+62.5%, +79.4%] | 1.93 s / 2.60 s | 286 / **411 MiB** |
| fdu-summary vs 1-thread statx floor | 328 ms | 685 ms | +104% | [+87%, +119%] | — | — |

Work classes differ exactly as the comparison methodology says they do — fdu-tree
retains a complete reusable index while dut/pdu/dust retain a rendered tree — but on
Linux the user waits for that difference: the index consumer costs ~2.3 µs/entry of user
CPU against dut’s ~0.1 µs/entry.
The summary plan (exp-040’s planner tier, no index) is already at the front of the
scalar class: a statistical tie with diskus (interval within the 3% bar) and 2.7× faster
than dua, while returning five exact tallies instead of one total.

The `fdu-summary` line versus the single-threaded floor also confirms the parallel
producer works on Linux: 4 default workers turned a 685 ms floor into 302 ms.

## Cold results

Controlled-cold per the Linux plan’s preparation (`sync` then
`echo 3 > /proc/sys/vm/drop_caches` before every sample, six pairs per matchup), with
the virtualization caveat repeated: guest-cold reads can still be host-cached, so this
regime orders strategies but understates real device latency.

| Matchup (A vs B) | A wall | B wall | B vs A | 95% CI | A CPU / B CPU |
| --- | ---: | ---: | ---: | --- | --- |
| fdu-tree vs dut | 1,784 ms | 1,601 ms | −13.7% | [−18.8%, −7.5%] | 5.32 s / 3.28 s |
| fdu-summary vs diskus | 1,680 ms | **1,346 ms** | −22.8% | [−27.9%, −14.8%] | 4.05 s / 3.90 s |
| spike readdir vs statx | 4,145 ms | 4,189 ms | +1.2% | [−2.7%, +6.0%] | — |
| spike statx vs inosort | 4,300 ms | 4,197 ms | −2.3% | [−3.9%, +0.5%] | — |
| spike statx vs io_uring | 4,133 ms | 7,475 ms | **+77.6%** | [+72.7%, +92.4%] | 2.94 s / 6.78 s |
| fdu-tree vs fdu-warm | 1,714 ms | 2,971 ms | **+72.4%** | [+66.4%, +94.0%] | 4.87 s / 5.47 s |

Readings:

- **The tree-class gap collapses cold** (−69% warm → −14%): once storage latency
  dominates, fdu’s bounded parallel producer holds its own against dut.
  The index consumer is a warm-path tax, which is consistent with the warm analysis.
- **diskus wins the cold scalar class by 23%.** Its stated design (3× cores for cold I/O
  queue depth, 12 threads here) out-hides fdu’s adaptive policy, whose 30 µs/entry
  unlock threshold and 6-worker default were calibrated on APFS. A Linux cold worker
  sweep — the plan already requires one — should retune the calibration constants per
  platform rather than inherit them.
- **Inode-ordered statting did not reproduce prior art’s cold win here** (−2.3%,
  interval crossing zero, against a 4–6× literature claim): expected on a rig whose
  “cold” storage is host-cached.
  Unresolved, not refuted; it needs bare metal.
- **io_uring statx loses cold as well as warm** (+78% wall, 2.3× CPU at QD 128
  per-directory batches).
  With warm at +327%, the io_uring frontier (`fdu-ktka`) should be treated as closed on
  current evidence unless a bare-metal high-latency cold run reopens it; the
  syscall-batching gap that `getattrlistbulk` fills on macOS simply has no profitable
  Linux analog today.
- **The warm-open inversion is cache-state-independent** (+72% both regimes), which
  points the blame at load/reconcile/save construction cost, not at page-cache effects.

## Allocator spike

A local-only build with mimalloc as the global allocator (never committed; the
dependency would need the supply-chain process first):

| Matchup | Stock | mimalloc | Change | 95% CI | CPU | RSS |
| --- | ---: | ---: | ---: | --- | --- | --- |
| summary plan, warm | 344 ms | **242 ms** | **−30.3%** | [−32.9%, −25.6%] | 1.23 → 0.94 s | 31 → 46 MiB |
| full-index tree, warm | 1,079 ms | 1,030 ms | −0.9% | [−13.8%, +9.7%] | 1.99 → 1.93 s | 284 → 248 MiB |

The summary plan’s allocation pattern — producers allocate paths and observation
batches, the consumer thread frees them — is the cross-thread free pattern glibc malloc
handles worst and mimalloc handles best.
A −30% wall change with the interval far below zero would put the summary plan
decisively ahead of diskus warm on this rig (242 vs 296 ms) while returning five exact
tallies to diskus’s one total.
The index path is unaffected: its cost is BTreeMap traversal, not allocation, which
independently corroborates the consumer-layout program (H19–H22/H60). The RSS increase
(+49% on a small base) needs a million-entry check before adoption, and the dependency
itself needs the documented cool-off and audit.

## What this changes

Ordered by expected effect on the Linux ranking:

1. **The consumer program is the Linux program** (existing beads `fdu-weey` H60,
   `fdu-prph` H19–H22, `fdu-sk7v` H66). This note’s numbers say those queued items are
   worth roughly a 3× wall factor on the Linux rendered-tree class, not a memory
   refinement. H66 alone should put the cache-off tree view in dut/pdu territory (pdu
   holds a rendered tree in 11.6 MiB at 450k entries; the physics is available).
2. **Fix the warm-open inversion** (new beads `fdu-maxn` no-op save skip, `fdu-niuz`
   Arc’d save, `fdu-91ts` O(N) snapshot load).
   Until warm open beats cold scan on Linux, the cache is a liability here: today it
   costs +72% wall and +44% RSS on an unchanged tree.
3. **Stop planning syscall-layer work on Linux warm** (H58 downgraded; raw
   getdents64/narrow-statx/io_uring warm variants measured at or below noise, io_uring
   catastrophically negative).
   The remaining syscall lever is *stat elision by tier*: files-only statx for the
   summary plan (~6% fewer stats on this tree, more on dir-heavy trees, `d_type`-gated
   with `DT_UNKNOWN` fallback), and any future attrs-free tier (a pure `--view files`
   listing could skip stats entirely, 8× fewer syscalls, if the planner ever proves no
   size/mtime output is requested).
4. **Cold remains the open Linux question** — see cold section.

## Reproduction

The walker harness and paired runner used here are committed under `benchmarks/spikes/`;
the generated tree, exact commands, and raw JSONL rows are documented in the spike
README. Nothing here entered the experiment ledger: the rig is virtualized and the
subject was generated for this session, so any of these results that motivates a
production change must re-run under the ledger’s protocol on real hardware first.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
