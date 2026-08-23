# The Metadata Walk Floor: What the Machine Costs, and Where fdu Sits Against It

**Date:** 2026-08-23

**Author:** fdu project, with Claude Code review assistance

**Status:** Current

## Overview

Every experiment in [the ledger](report-2026-08-10-fdu-performance-experiments.md)
answers the same shape of question: is this change faster than the code it came from.
Sixty-four of them now say yes or no with intervals.
None of them answers the question that decides whether to run a sixty-fifth: **how much
is left.**

A relative loop cannot answer that, because it has no denominator.
This note supplies one.
It measures what the machine costs for the work fdu cannot avoid — enumerate every
entry, read every entry’s metadata, add up five integers — and reads every tier of fdu,
every peer walker, and every queued mechanism against it.

Three results carry the note:

1. **The aggregate tier is close to done, and how close depends on the tree.** fdu’s
   `--view summary` runs at **1.20× a hand-written parallel syscall floor on the primary
   synthetic subject and 1.59× on `/usr`**, so the remaining prize on that tier, in this
   regime, is 17–37% — and `arena_spike`, the consumer redesign this project has already
   prototyped, measures **1.06×**. There is no third act.
2. **fdu leads ripgrep’s walker on every generated tree and loses on the real one.**
   Against [`ignore`](https://docs.rs/ignore) — ripgrep’s walker — doing the identical
   job, fdu is **12–26% faster on four synthetic subjects, tied on a tree carrying real
   filenames, and 11.8% slower on `/usr`**. Both halves have one mechanism: `ignore` and
   `walkdir` stat each entry by full absolute path where fdu stats relative to the
   directory descriptor, worth **+37%** to fdu — and a real tree’s names and widths cost
   fdu about that much back, while costing `ignore` nothing.
3. **Batching syscalls is the wrong lever, measured twice.** io_uring-batched `statx`
   cuts syscalls 21× and runs **5.9–7.6× slower** than plain `statx`, each ring against
   its own single-threaded control, and never beats plain threads at any thread count in
   either cache state. The warm floor is VFS work, not the syscall boundary, which bounds
   what any bulk interface could ever buy here at about **9%**.

Three queue items change priority as a result, and one methodological defect surfaces
that affects how every generated-corpus number in this project should be read.
Part 4 states each verdict; where they landed in the actual work order is
[the campaign-2 plan](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md),
which this report’s denominators are the basis for.

Everything here is one virtualized host, warm page cache unless stated, and is scouting
evidence under [the loop’s](../guides/performance-loop.md) rules: it orders the queue
and must not be quoted as product numbers.

## The rig, the subjects, and the oracle

4-vCPU Intel Xeon @ 2.80 GHz KVM guest, 15 GiB RAM, kernel 6.18.44, ext4 on virtio,
root; rustc 1.97.1; release profile.
This is the same class of rig as
[the consumer structural-headroom review](../research/research-2026-08-15-consumer-structural-headroom.md),
so its numbers are directly comparable.

Five subjects, because one tree cannot separate size from shape from content:

| Subject | Entries | Dirs | Entries/dir | What it isolates |
| --- | ---: | ---: | ---: | --- |
| `wide` | 401,999 | 2,000 | 201.0 | per-directory cost amortized away |
| `tree` | 419,999 | 20,000 | 21.0 | the primary subject |
| `narrow` | 399,999 | 80,000 | 5.0 | per-directory cost dominant |
| `usrshape` | 85,799 | 7,800 | 11.0 | `/usr`’s size and width, uniform names |
| `usrnolnk` | 84,423 | 7,842 | 10.8 | `/usr`’s real names and widths, no symlinks |
| `/usr` | 84,535 | 7,843 | 10.8 | a real tree |

`usrshape` and `usrnolnk` are matched controls, built for this note: the first shares
`/usr`’s entry count and directory-width distribution but uses generated names, and the
second reproduces `/usr`’s actual directory tree and filenames with every symlink
replaced by an ordinary file.
Between the three, size, width, names, and symlinks each vary alone.

Two binaries were measured, either side of the workspace split that moved the engine to
`fdu-core` and put the CLI on its public API. The split touches exactly the layer the
aggregate tier’s one-shot plan runs through, so it could not be assumed to carry.
It did, on the three subjects re-run against it: 1.20× the floor against 1.17× before,
`usrnolnk` at +1.7% against `ignore` where it was +1.5%, `/usr` at +12.4% where it was
+11.8%, and every tally identical.

The full sweeps below — the sixteen-program floor table and the six-subject peer
comparison — are the pre-split binary, because re-running both in full would move every
number by less than the drift between sittings.
Where a post-split value exists it is given beside the original.
Nothing in the record depends on the difference.

Every instrument reports the same five tallies, and on the primary subject all of them
agree exactly — 19,999 directories, 419,999 entries, 1,199,792,066 apparent bytes,
2,158,034,944 allocated bytes — across `parfloor` at three variants and four thread
counts, `walkspike`, `arena_spike`, `peerwalk`’s five walkers, and fdu’s own summary.
That agreement is the oracle: a walker that is fast and disagrees is broken, not fast.
Timing is paired and interleaved in alternating order with medians of 15 trials after 3
warmups, which is [the loop’s protocol](../guides/performance-loop.md#the-loop) applied
to peer tools rather than to two fdu builds.

### What had to be built, and what already existed

`walkspike.c` already isolates this layer, including an io_uring variant, and
`arena_spike.rs` already measures the consumer ceiling.
Both were re-run here rather than re-derived, and both reproduced.
Two instruments were genuinely missing and are added by this note:

- **`explorations/benchmarks/spikes/parfloor.c`** — the *parallel* syscall floor.
  `walkspike` is single-threaded, which ranks syscall strategies correctly and cannot
  serve as a lower bound for a four-worker walker: a one-thread floor sits above fdu,
  not below it. `parfloor` also carries the `abspath` variant that prices absolute-path
  statting on its own.
- **`explorations/benchmarks/spikes/peerwalk.rs`** — the ecosystem anchor: `ignore`,
  `walkdir`, and `jwalk` over one tree on one job.
  Nothing in this repository previously compared fdu against the walker a Rust program
  would otherwise reach for.

## Part 1 — What the machine costs

Five floors stack. Only the first is a theorem; the rest are measurements of this rig.

### The information floor: Ω(N), and the cache cannot dodge it

An exact total over N entries requires observing N sizes, and each size lives in a
distinct inode. Nothing beneath the filesystem aggregates them, so no algorithm answers
exactly in fewer than N observations unless something else already did the aggregation —
a journal, a quota subsystem, an FSEvents stream.

This is the same argument the README makes for the cache’s honesty contract, and it is
worth restating as a limit rather than a policy: **a directory fingerprint proves only
that no entry was added, removed, or renamed.** An in-place edit changes no directory’s
mtime. So a warm revalidation stats every entry, and a snapshot cannot beat the walk it
still has to do. `plan_report`’s decision not to read the snapshot for a one-shot
metadata query is that theorem expressed in code, and the measurement below confirms the
sign.

### The interface floor: what Linux offers

Per entry, exactly one metadata call.
Linux has no bulk-attribute call — no `getattrlistbulk`, no `readdirplus` for local
filesystems — so `statx` per entry is not an implementation choice, it is the whole
surface. Per directory: one `openat`, one `close`, and **two** `getdents64`, the second
returning zero to say the directory is exhausted.

The syscall census on `/usr/lib` (17,721 entries, 1,705 directories) puts each
implementation against that floor:

|  | `statx` | `getdents64` | `openat` | `close` | `fstat` | total | vs floor |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `parfloor` | 17,720 | 3,402 | 1,705 | 1,705 | 4 | 24,536 | — |
| fdu | 17,725 | 3,403 | 1,709 | 1,709 | 1,705 | 26,255 | +7.0% |
| `ignore` | 16,025 | 3,403 | 1,709 | 1,709 | 1,705 | 24,551 | +0.1% |

Two things to read off it.
fdu’s 7% excess is **one `fstat` per directory**, which is glibc’s `opendir` inside
`std::fs::read_dir` measuring the directory’s block size; `ignore` pays it too, through
the same code. `ignore`’s 1,695 fewer `statx` calls are not an efficiency — it skips
metadata on directories, and fdu cannot, because directory mtime is precisely what its
incremental revalidation is fingerprinting.
That is semantics, not waste, and it is the right way round: the cheaper tool is buying
its discount with a capability.

The doubled `getdents64` is confirmed independently here at exactly 2.00 per directory.
What is new is that it is now priced — see Part 4.

### The warm kernel floor: measured, ~1 µs per entry

With the page cache warm, `parfloor stat` costs **1.411 µs/entry** single-threaded, and
`parfloor enum` — the same walk with the metadata call removed — costs 0.420 µs/entry.
The metadata call is therefore worth about **0.99 µs/entry**.

A raw syscall entry and exit on this host, measured with a `getppid` loop, is **95 ns**.
So the syscall boundary is **9%** of what a warm `statx` costs and the VFS is the other
91%: dentry hash lookup, inode resolution, permission check, and copying a 256-byte
`statx` buffer to user space.
At 2.8 GHz that is roughly 2,800 cycles to answer “how big is this file” about an object
already in memory.

**That 9% is the ceiling on every batching idea.** It bounds io_uring, it bounds a
hypothetical Linux bulk-stat, and it is why the macOS result does not transfer:
`getattrlistbulk` wins on APFS because it removes the *lookup*, not because it removes
the syscall.

### The parallel floor: near-linear, so the wall floor is N·t/cores

`parfloor stat` scales 592.7 ms → 184.3 ms on 4 cores, **3.22×**, with the `abspath`
variant at 3.87× and fdu’s aggregate tier at 3.83×. Warm metadata work is CPU-bound in
the kernel and the VFS parallelizes cleanly at this scale, so the wall floor is simply
per-entry cost divided by cores.
On this rig that is **0.44 µs/entry**, or about 440 ms for a million entries on four
cores.

Nothing here is I/O bound.
That is the single most important thing to hold onto about the warm regime, and it is
why the entire optimization loop has been a contest over user CPU and kernel CPU rather
than over waiting.

### The cold floor: a different problem, and a lever this rig cannot settle

Guest-cold — `sync` plus `drop_caches` before every sample, host page cache still
beneath, so device latency is understated and only ordering strategies can be read:

| Variant | Cold wall | vs warm |
| --- | ---: | ---: |
| `parfloor stat` j1 | 3,567 ms | 6.0× |
| `parfloor stat` j4 | 1,142 ms | 6.2× |
| `parfloor stat` j16 | 780 ms | — |
| io_uring j4 | 2,253 ms | — |
| io_uring j16 | 1,002 ms | — |
| io_uring j32 | 923 ms | — |
| `ignore-stat` j4 | 1,175 ms | 4.2× |
| **fdu `--view summary`** | **1,044 ms** | **4.8×** |
| GNU `du -s` | 3,069 ms | 4.7× |

Two readings. Cold work is not merely warm work plus waiting: at one thread, CPU rises
from 598 ms to 2,793 ms, so most of the cold penalty is the kernel rebuilding dentries
and inodes, not the disk.
And **threads keep paying past core count** — j16 beats j4 by 32% on four cores, which
cannot be compute and must be queue depth.
That is the mechanism behind H73, H76 and the adaptive worker expansion, and it is
visible on Linux here for the first time.
It also remains unsettleable on this rig for exactly the reason
[the platform guide](../guides/platform-tuning.md#host) gives: the host’s cache sits
under the guest’s, so the *size* of the effect on real storage is not measured, only its
sign.

## Part 2 — Where fdu sits

One interleaved sitting, 420k-entry subject, warm, 15 trials after 3 warmups.
Peak RSS has the harness’s 11.4 MiB copy-on-write baseline subtracted.

| Program | Work returned | Wall | ×floor | CPU | Par | RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `parfloor enum` j4 | entry names and types | 60.3 ms | 0.33 | 218 ms | 3.62 | ~0 |
| `peerwalk ignore-nostat` j4 | ripgrep’s job | 118.9 ms | 0.65 | 407 ms | 3.43 | ~0 |
| `peerwalk ignore-default` j4 | ripgrep’s job, rg’s filters | 184.6 ms | 1.00 | 624 ms | 3.38 | ~0 |
| **`parfloor stat` j4** | **five tallies — the floor** | **184.3 ms** | **1.00** | **695 ms** | **3.77** | **~0** |
| `arena_spike` j4 | index-shaped records | 194.6 ms | 1.06 | 727 ms | 3.74 | 11 MiB |
| **fdu `--view summary`** | **five exact tallies** | **216.5 ms** | **1.17** | **829 ms** | **3.83** | **~0** |
| `parfloor abspath` j4 | five tallies, abs-path stat | 252.2 ms | 1.37 | 975 ms | 3.87 | ~0 |
| **`peerwalk ignore-stat` j4** | **five tallies** | **278.0 ms** | **1.51** | **1,088 ms** | **3.91** | **~0** |
| fdu `--cache only` | full index, from snapshot | 369.5 ms | 2.00 | 378 ms | 1.02 | 169 MiB |
| fdu default tree | full index + rendered tree | 493.4 ms | 2.68 | 1,218 ms | 2.47 | 195 MiB |
| `parfloor stat` j1 | five tallies, one thread | 592.7 ms | 3.22 | 598 ms | 1.01 | ~0 |
| `walkspike statx` j1 | five tallies, one thread | 595.3 ms | 3.23 | 582 ms | 0.98 | ~0 |
| GNU `du -s` | one total | 646.7 ms | 3.51 | 643 ms | 0.99 | ~0 |
| `peerwalk jwalk` j4 | five tallies | 864.0 ms | 4.69 | 1,211 ms | 1.40 | 87 MiB |
| `peerwalk walkdir` j1 | five tallies | 892.7 ms | 4.84 | 903 ms | 1.01 | ~0 |
| `walkspike uring` j1 | five tallies | 3,534.2 ms | 19.18 | 4,341 ms | 1.23 | ~0 |

Two things about that ratio have to be stated rather than hidden, because a single
number for it would be wrong in both directions.

**The floor implementation matters at the margin.** `parfloor` keeps a queue of
directory *paths*, as `walkspike` does, and pays a `strdup` and a mutex per directory.
A variant holding directory descriptors instead measured 173.8–177.3 ms across sittings.
On this subject fdu is 1.17–1.20× the committed floor and 1.32–1.35× the tightest one —
a residual smaller than the disagreement between two reasonable definitions of “floor”.

**And the tree matters much more than the implementation does.** The ratio is not a
constant of the program; it is a property of the program *and* the tree, and it moves by
more than a third across subjects:

| Subject | fdu | floor | ×floor |
| --- | ---: | ---: | ---: |
| `narrow` — 400k synthetic, 5 entries/dir | 312.2 ms | 269.6 ms | **1.16** |
| `tree` — 420k synthetic, 21 entries/dir | 197.6 ms | 164.3 ms | **1.20** |
| `usrshape` — 86k synthetic, generated names | 51.2 ms | 37.9 ms | **1.35** |
| `usrnolnk` — 84k, `/usr`’s real names | 57.6 ms | 40.6 ms | **1.42** |
| `wide` — 402k synthetic, 201 entries/dir | 193.6 ms | 128.7 ms | **1.50** |
| `/usr` — 85k real tree | 70.8 ms | 44.7 ms | **1.59** |

Quoting the primary subject alone would claim 17% of headroom where the real tree says
37%. Part 4 explains why the real-tree rows sit highest, and it is the same effect that
decides the peer comparison in Part 3.

### The aggregate tier is nearly finished

At 1.20× on the primary subject and 1.59× on `/usr`, the remaining prize on
`--view summary` in this regime is 17–37%, and `arena_spike` at 1.06× shows what claims
it: the representation change.
Reading that against
[the structural-headroom review’s](../research/research-2026-08-15-consumer-structural-headroom.md)
conclusion sharpens it usefully.
That review measured the redesign as worth ~4× on the *tree* view.
Against the floor it is worth something stronger and more final: it lands essentially
**on** the floor, which means S1–S4, H19–H22, H60 and `fdu-2ubt` are not the next win in
a series. They are the last one on this tier.

### The index tier is where the money is, and it has a tail

The index tier costs 493 ms against the aggregate tier’s 217 ms and the floor’s 184 ms —
**2.68× the floor** — and its parallelism drops from 3.83 to 2.47. With 1,218 ms of CPU
and four cores available, perfect scaling would finish in 305 ms; it takes 493 ms, so
**about 38% of the elapsed time is serialization**, which is the single-writer
arbitration the design deliberately buys.

It is also the only tier here with an unstable wall time.
Fifteen consecutive samples on a quiet host:

```
560 503 567 989 1489 474 547 526 523 476 534 531 456 522 495
```

Median 526 ms, minimum 456 ms, maximum 1,489 ms — a **3.27× spread**, with 55,117 minor
faults against the aggregate tier’s 3,430. The aggregate tier over the same fifteen
samples spread 1.04×. The tail is an allocation and page-fault phenomenon, it is
user-visible latency, and it is a second reason to want the arena representation beyond
its median. It is also a hazard for the loop itself: a 3% accept gate on a distribution
with a 3× right tail needs the paired median it already uses, and would be meaningless
on a mean or a small N.

### The snapshot barely pays, and the code already knew

`--cache only` answers from the snapshot without touching the filesystem in 369.5 ms —
only **25% faster than walking the tree cold and building the same index** (493.4 ms),
and single-threaded while doing it.
Deserializing 420k records costs 0.88 µs/entry; walking and indexing them costs 1.18
µs/entry. Reading your own index back is nearly as expensive as re-deriving it from the
kernel.

This is the measurement `plan_report`’s doc comment already asserts, reproduced at a
different scale on a different tree, and it is worth stating as a general result rather
than a local one: **on a warm page cache, a serialized index is not a shortcut past a
metadata walk.** A snapshot earns its keep where it avoids work of a different kind —
file bodies for content analysis, or a genuinely cold filesystem — and nowhere else.

## Part 3 — The ripgrep anchor

Four questions, answered from primary sources and measurement.

**Does ripgrep use the `ignore` crate?** Yes.
ripgrep 15.2.0’s crates.io index entry declares `ignore ^0.4.29` as a normal dependency,
and the `ignore` crate is published from the ripgrep repository itself.
`ignore` is in turn built on `walkdir`, by the same author, for its single-threaded
path, and has its own parallel walker for the rest.

**Does fdu use it?** No — and not `walkdir` either.
fdu’s walker is `std::fs::read_dir` plus a hand-written bounded parallel scheduler, with
an audited `getattrlistbulk` backend on macOS. `ignore` does not appear in `Cargo.lock`
at all; `walkdir` appears only transitively, through `notify`, behind the optional
`watch` feature.

**How fast is it?** Doing ripgrep’s own job — enumerate and classify by `d_type`, no
metadata call — `ignore` runs the 420k tree in **118.9 ms**, or 0.283 µs/entry, at 1.97×
the pure-enumeration floor; with ripgrep’s default gitignore and hidden-file filters on,
184.6 ms. Doing du’s job it runs **278.0 ms**, 1.51× the floor.
`walkdir` alone is 892.7 ms and `jwalk` 864.0 ms, so `ignore`’s parallel walker is worth
3.2× over the crate it is built on and `jwalk` is not competitive with either.

**How does fdu compare?** It depends on the tree, and the dependence is the finding.
Thirteen paired trials per subject after three warmups, alternating order, reporting the
median of the paired differences:

| Subject | Entries/dir | fdu | `ignore` | Paired |
| --- | ---: | ---: | ---: | ---: |
| `tree` — synthetic | 21.0 | 218.4 ms | 300.9 ms | **−26.2%** |
| `usrshape` — synthetic, generated names | 11.0 | 59.0 ms | 72.6 ms | **−21.3%** |
| `wide` — synthetic | 201.0 | 200.2 ms | 242.4 ms | **−16.2%** |
| `narrow` — synthetic | 5.0 | 332.4 ms | 379.1 ms | **−12.6%** |
| `usrnolnk` — `/usr`’s real names | 10.8 | 69.8 ms | 68.7 ms | **+1.5%** |
| `/usr` — real tree | 10.8 | 70.8 ms | 63.3 ms | **+11.8%** |

fdu leads by 12–26% on every generated subject regardless of shape, ties the moment real
filenames are introduced, and loses by 11.8% on the one real tree.

An earlier draft of this note reported the first row alone as “22% faster, and that
result held in every sitting”.
It did hold — on that subject.
It was read off the primary synthetic tree and generalised without checking it against
the real-tree row printed in this document’s own matched-control table, which already
disagreed. The correction is recorded rather than quietly fixed because the mistake is
the one this whole section is about: a uniform corpus flatters fdu specifically, and the
person most likely to be fooled by that is the one who just measured it.

The two halves of the comparison have a single mechanism, pulling opposite ways.

### Why fdu leads, and why the lead does not survive a real tree

Not scheduling, not batching, and not the parallel walker: `ignore` achieves marginally
*better* parallelism here (3.91 against 3.83). It is one line.

Both `ignore` and `walkdir` implement per-entry metadata as
`fs::symlink_metadata(&self.path)` — a stat on the full absolute path, so the kernel
resolves every component from the root, for every entry.
`std::fs::DirEntry::metadata()`, which fdu uses, issues a dirfd-relative `statx`
resolving a single component.

`parfloor`’s `abspath` variant prices exactly that, changing nothing else:

|  | Wall j4 | CPU | Penalty |
| --- | ---: | ---: | ---: |
| `parfloor stat` (dirfd-relative) | 184.3 ms | 695 ms | — |
| `parfloor abspath` (absolute path) | 252.2 ms | 975 ms | **+37% wall, +40% CPU** |

The absolute-path penalty is 67.9 ms; the fdu-to-`ignore` gap on the primary synthetic
subject is 61.5 ms. One choice more than accounts for the whole lead, and it is a choice
fdu did not make deliberately — it falls out of using `DirEntry::metadata()` rather than
re-statting a joined path.

Redundant path resolution is therefore the largest single avoidable cost in this
workload, **0.16 µs/entry** on this rig, and it is invisible in a syscall census because
the call counts are identical.
That is a real finding about the ecosystem’s walkers and it stands on its own.

What it does not do is survive contact with `/usr`. Going from `usrshape` to `usrnolnk`
— same entry count, same directory widths, no symlinks in either, only the filenames
differ — moves fdu by 22.8 points against `ignore` while moving `ignore` itself barely
at all. That is the same effect measured against the floor in Part 4, and it is large
enough to eat a 21-point lead exactly.
So fdu banks a structural advantage in how it stats and spends it back in how it handles
names and paths; on a generated corpus only the first is visible.

The last half of the comparison favours ripgrep outright.
On *ripgrep’s* job fdu cannot compete at all, because that job is 118.9 ms and fdu’s
floor is 184.3 ms: a search tool learns what it needs from `d_type` and never makes a
metadata call, while a disk-usage tool must make one per entry.
**The interesting difference between these two tools is not implementation quality, it
is that one of them is allowed to skip 91% of the kernel work.**

## Part 4 — What this changes

### `fdu-jnuo` (the doubled `getdents64`) should be demoted

It is real: 2.00 `getdents64` per directory, confirmed independently here, and
`walkspike`’s `elide` variant removes exactly half of them.
But the floor now prices the whole enumeration layer — every `openat`, both
`getdents64`, every `close`, and the name copies — at **0.144 µs/entry**, or 60.3 ms of
the aggregate tier’s 216.5 ms wall.
The terminating call is a fraction of that, and it returns no data.
Bounding it generously at half the per-directory syscall cost puts it under **1% of
aggregate-tier wall** on a 21-wide tree and under **4%** on the 5-wide `narrow` subject.

That is below the accept gate before any of the safety analysis the item is blocked on —
the `statfs` `f_type` allowlist, because FUSE and network filesystems may legally return
a short buffer mid-stream.
It should be composed into a dir-heavy or cold campaign if it is done at all, never run
as its own experiment.

### io_uring is settled for the warm regime, and still open for the cold one

Two independent implementations agree.
`walkspike`’s hand-rolled ring: 3,534 ms against 595 ms for plain `statx`, **5.9×
slower**, single-threaded.
A separately written Rust ring built for this note, measured beside its own control in
one sitting: 3,639 ms against 481 ms, **7.6× slower**, with syscalls cut from 419,999
`statx` to 20,000 `io_uring_enter`. A 21× reduction in kernel transitions bought a 7.6×
slowdown.

The mechanism is that `IORING_OP_STATX` is dispatched to io-wq kernel worker threads
rather than executed inline, so each entry buys a thread handoff to save a 95 ns syscall
boundary. At four threads it narrows but never wins — 312 ms against the same harness’s
174 ms floor in one sitting, **+80%** — and cold it never beats plain threads at any
depth tested up to 32.

The general form of this result is the useful part: **syscall count is not the warm
metadata walk’s cost, and any hypothesis whose mechanism is “fewer kernel transitions”
is bounded at 9% before it starts.** Queue depth on genuinely cold storage is a separate
claim with a different mechanism, it is unrefuted, and it still needs bare metal.

### Generated corpora understate real-tree cost, on this tier, by about 15 points

This is the methodological finding, and it affects how existing numbers should be read.

| Subject | Floor | `ignore-stat` | fdu summary | fdu ×floor |
| --- | ---: | ---: | ---: | ---: |
| `usrshape` — matched size and width, generated names | 36.1 ms | 60.7 ms | 47.5 ms | **1.32** |
| `usrnolnk` — `/usr`’s real names and widths, no symlinks | 34.0 ms | 56.5 ms | 51.6 ms | **1.52** |
| `/usr` — real | 39.5 ms | 57.3 ms | 58.9 ms | **1.49** |

The controls do their job.
`usrshape` and `usrnolnk` have the same entry count and the same directory-width
distribution, and neither contains a symlink, so the 1.32 → 1.52 step isolates the
effect of a real tree’s **names and width distribution** alone.
Adding `/usr`’s 8,447 real symlinks then moves nothing (1.52 → 1.49), which rules out
symlink handling; hard links are ruled out too, since `/usr` has only 11 multiply-linked
files.
Per-entry allocation counts are within 3% across the pair, so it is not allocation
volume either.

Three consequences.
The cost lands on fdu and not on `ignore` (60.7 → 56.5 → 57.3, flat),
which points at fdu’s per-entry name and path handling — `fdu-2ubt`’s `PathBuf` clone
per entry is the obvious candidate and this raises its value.

**A uniform corpus hides roughly 15 percentage points of fdu’s distance from the
floor.** [The loop](../guides/performance-loop.md#the-reference-tree) already requires a
real nominated tree for exactly this reason; this quantifies the requirement on the
aggregate tier, and it means any figure taken against `gen_tree.py` should be read as a
lower bound on real-tree cost, not as an estimate of it.

And it is not only a bound on fdu’s own numbers — **it can invert a comparison.** The
peer result in Part 3 is the demonstration: on generated trees fdu leads `ignore` by
12–26% at every shape tested, and on `/usr` it trails by 11.8%, because the effect this
table isolates is exactly large enough to consume the lead.
A ranking established on a generated corpus is not evidence of a ranking, and this
document produced the wrong one before the controls were run.

### The one `fstat` per directory is free to remove and worth little

fdu’s 7% syscall excess over the floor is glibc’s `opendir` calling `fstat` to size its
buffer, reached through `std::fs::read_dir`. Going to `openat` plus raw `getdents64`
removes it and the `elide` call together.
By the same pricing as above it is worth well under 1% of aggregate-tier wall, so it is
only interesting as a side effect of a producer rewrite that is happening anyway — which
is the same conclusion the enumeration floor forces on every syscall-shaped hypothesis
on this tier.

## What this evidence does not support

**One host, and virtualized.** Every number is one 4-vCPU KVM guest on ext4. Warm
results describe the environment most fdu runs happen in and are ordinary evidence about
it; cold results here are guest-cold, so device latency is understated and only the sign
of an ordering effect can be read, never its size.

**No macOS.** The floor is a Linux floor.
`getattrlistbulk` changes the interface floor itself, so none of Part 1’s per-entry
costs transfer to APFS, and the claim that batching is bounded at 9% is a claim about
Linux only — on macOS the equivalent bound is what makes the bulk reader worth having.

**Peer versions are current, not pinned.** `ignore` 0.4.33, `walkdir` 2.5.0, `jwalk`
0.8.1, resolved at the time of this note.
ripgrep’s dependency declaration was read from the crates.io index at version 15.2.0. A
pinned installation attestation of the kind
[the tool comparison](report-2026-08-13-fdu-live-tool-comparison.md) carries would be
needed before any of this is published as a product comparison.

**`peerwalk` measures the walkers, not the tools.** It is a harness calling `ignore` the
way a disk-usage tool would, not ripgrep.
It says what the walker costs; it says nothing about ripgrep’s search performance, which
is a different program.

**One real tree is not a population.** The peer comparison inverts between generated
trees and `/usr`, and `/usr` is the only real tree measured here.
That is enough to retire the generated-corpus ranking; it is not enough to establish the
real-tree one. A claim in either direction needs several real trees on more than one
host, with the pinned binaries and installation attestation the
[macOS tool comparison](report-2026-08-13-fdu-live-tool-comparison.md) carries.

**The floor is a floor for this product.** `parfloor` retains nothing, reports no
errors, handles no partial results, and has no delta contract.
That is what makes it a bound rather than a design.
The distance between it and fdu is not all waste — some of it is the product.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
