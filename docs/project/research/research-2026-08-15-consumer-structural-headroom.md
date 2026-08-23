# Research: A Senior Review of the Performance Campaign — Measured Structural Headroom, and What the Queue Missed

**Date:** 2026-08-15

**Author:** fdu project, with Claude Code review assistance

**Status:** Measurements current; the queue in
[The queue, re-ordered by this evidence](#the-queue-re-ordered-by-this-evidence) is
superseded by
[the campaign-2 plan](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md)
(item 3 has since landed as H87 and H88 with `fdu-6kyn` remaining; H89 was refuted; H90
landed)

## Overview

This note is a deliberate change of altitude: not another turn of the tuning loop, but a
review of the campaign itself — what its 56 experiments have in common, what the open
queue is worth, and which directions never made it into the queue at all.
It follows [the structural review](research-2026-08-14-structural-performance-review.md)
and extends it with new measurements taken for this review on a Linux VM of the same
class as the scouting rig, using the same generator, at the same 450k scale.

Three conclusions, each carried by a measurement in this note:

1. **The consumer redesign is worth ~4× on the Linux tree view, and it should be run as
   one structural experiment, not four gated increments.** A 250-line oracle-checked
   spike (`benchmarks/spikes/arena_spike.rs`) that retains an index-shaped result —
   per-file records, a name arena, per-directory tallies, one bottom-up roll-up pass —
   runs the same tree, same syscalls, same worker count in **~199 ms and ≤ 23 MiB**
   where the fdu CLI tree view spends **~849 ms and ~279 MiB**, beside dut’s ~179 ms.
   S1–S4, H19–H22, H60 and `fdu-2ubt` are one representation change wearing seven
   hypothesis numbers, and measuring them one 3%-gate at a time will thrash: each
   partial form pays conversion costs at the boundary that the end state deletes.
2. **The cold scalar gap to diskus on Linux is a thread-policy constant, nothing else.**
   Guest-cold on this rig, fdu’s summary at its automatic four workers (~1,330 ms)
   *beats* diskus at equal threads (~1,900 ms) and loses only to diskus’s 3×-cores
   default (~1,037 ms).
   H76/H84 are confirmed mechanically and are the cheapest large win in the queue.
3. **The first-run default pays a 56% cache-write tax on the critical path,** and most
   of it is not the file write: `spawn_save` deep-clones the entire index synchronously
   before rendering can proceed, and the snapshot CRC is computed a byte at a time on
   ISAs that have a CRC32C instruction.
   `--cache off` 860 ms vs `--cache refresh` 1,341 ms on the same tree.

Everything here was produced on one virtualized host and is scouting evidence under the
loop’s rules: it orders the queue and must not be quoted as product numbers.
Anything that motivates a production change re-runs under the ledger protocol.

## The rig, and what was reproduced before anything new was claimed

4-vCPU Intel VM, 15 GiB RAM, kernel 6.18.5, ext4 on virtio, root; rustc 1.97.1; release
profile (fat LTO, one codegen unit).
Subject: `gen_tree.py` at 450,463 entries (28,630 dirs, 421,690 files), warm unless
stated. The known results reproduce before any new claim:

- Syscall census matches the scouting doc exactly: 450,471 `statx`, 57,260 `getdents64`
  (2.00 per directory), 28,641 `openat`+`close`.
- Per-layer counters match the documented shape: 15.4 allocations, 10.9 reallocations,
  11.9 roll-up merges per entry; 93.6% parent-memo hit rate; 100% page-cache-served.
- Callgrind on a 40k subtree, with the probe’s oracle digest backed out (it is ~39% of
  instructions — see the harness note below), attributes the engine’s work: **~36%
  allocator**, **~15% `Path::Components` parsing**, ~7% `merge_upward`, ~3% `from_utf8`
  — the same profile shape the ledger records on macOS, now confirmed on Linux.

## The measured ceiling for the consumer program

`arena_spike.rs` walks with `read_dir` + `DirEntry::metadata` (the exact portable
syscall shape), four workers, a shared claim queue — and instead of channel-borne
path-addressed observations arbitrated by one consumer, each worker appends fixed-width
file records and name bytes to worker-local arenas, tallies each directory into a global
slot, and one pass after the walk folds directory tallies bottom-up by depth.

Five warm runs each, medians, this rig:

| Program | Wall | Peak RSS | Retains |
| --- | ---: | ---: | --- |
| dut `-t4` | 179 ms | ≤ 23 MiB | rendered top-N tree |
| **arena_spike, 4 workers** | **199 ms** | **≤ 23 MiB** | per-file records + names + per-dir tallies |
| diskus `-j4` | 296 ms | ~12 MiB | one total |
| fdu `--cache off --view summary` | 371 ms | ~50 MiB | five exact tallies |
| fdu `--cache off` (tree) | 849 ms | ~279 MiB | full index, rendered tree |

The spike’s tallies match fdu’s summary exactly (files 421,690; bytes 3,000,524,491;
allocated 747,966,464), and its roll-up pass over 28,629 directories costs **0.8 ms**
where the live engine performs 5.35 million per-entry ancestor merges.
It is a physics measurement, not a prototype: no arbitration, no progressive
publication, no error provenance, no symlink records.
Those layers cost something real — but dut ships a correct product at the same number,
so the floor is not hypothetical.

**What this changes about the queue.** The structural review already ordered S1–S4 and
warned each shrinks the next.
This measurement sharpens that into a stronger claim: S2 (name arena), S3 (children as
arena slices), S4 (arbitration-free bootstrap), H19–H22 (compact entries), H60
(worker-local subtrees) and `fdu-2ubt` (batch-shaped observations) are **one
representation decision**, and the intermediate states pay conversion costs the end
state deletes — a batch-shaped observation still applied into boxed `BTreeMap` entries
converts twice; an arena entry fed per-op through the current channel still allocates
per op. Run it as one experiment with the differential harness (`assert_same_image` at
every worker count) as the gate, the way exp-022 landed 542 lines of `getattrlistbulk`
as one verdict. The 3% rule gates the *decision*, and a measured ~4× is not a marginal
decision; what it must not do is force the redesign through seven separate doors.

Worker-local arenas also dissolve H85’s cross-thread-free pathology structurally — names
and records are freed (never, until exit) by the thread that allocated them — which is
worth more than recycling batch buffers under the current shape.

## Findings that are in no ledger, queue, or review

### The default first run pays 56% for the cache write, mostly before rendering

Same tree, same warm state, four runs each:

| Invocation | Median wall |
| --- | ---: |
| `fdu --cache off TREE` | 860 ms |
| `fdu --cache refresh TREE` (scan + write, the first-run default shape) | 1,341 ms |
| `fdu TREE` warm (auto, snapshot valid) | 783 ms |

`spawn_save` (lib.rs) runs `Arc::new(index.clone())` — a deep clone of every boxed
entry, both name copies, and every `BTreeMap` — synchronously on the caller before the
save thread spawns, so the clone sits on the render path of every cache-writing run.
`fdu-niuz` already owns this clone but is filed as a changed-warm-path concern; it is
actually a **every-cold-default-run** concern, which raises its priority considerably.
An index behind `Arc` with copy-on-write semantics, or serialization from a borrowed
index before mutation resumes, removes it entirely; the compact-entry redesign above
makes the clone cheap as a side effect.
On the same path, `snapshot.rs` computes CRC-32C **one byte at a time** through a lookup
table, over a 28.7 MB image, twice per warm-write cycle — on x86-64 (SSE 4.2) and
aarch64 (ARMv8 CRC) the polynomial is a hardware instruction, reachable with
`core::arch` intrinsics plus runtime feature detection, no dependency and no unsafe
outside the counters-style confinement.
Both compose with H78’s format work but neither needs to wait for it.

### The extension pipeline allocates and validates per file for a u32 it already had

`derive_ext` builds a fresh `Vec` per file (dhat: ~0.9 allocations/file), runs it
through `String::from_utf8` (callgrind: ~3% of engine instructions), and `intern_ext`
then resolves it against a `BTreeMap<String, ExtId>` by string comparison — all to
re-derive an id that run-length locality makes nearly constant: directory listings
arrive with runs of the same extension.
A one-slot `(last_ext_bytes, ExtId)` memo beside the parent memo, or an interner keyed
on raw bytes with a small inline buffer, removes the allocation, the UTF-8 pass, and
most lookups. Alone it is a 3–5% class change (fold it into the structural experiment);
listed because no queue entry mentions it.

### The bootstrap journals every batch and then throws the journal away

`apply_baseline` calls `apply` — which clones the effective ops, `PathBuf`s included,
into the journal — and then `establish_baseline` clears that journal, per batch, for the
entire cold scan: one cloned-and-freed path per entry.
exp-003 measured “skip journalling on bootstrap” as unmeasurable — on macOS, at 60k, in
2026-08-11’s build, where the consumer was not yet the bottleneck, and via a duplicated
arbitration loop that was rightly rejected on complexity.
The residue is now a one-flag change on a path whose costs have been cut around it three
times since. Re-screen it on Linux; expect small-but-real, and let it ride with the
structural experiment rather than alone.

### The parent memo still parses paths — the contract is the fix

Even at a 93.6% memo hit rate, `Path::Components` iteration is ~15% of engine
instructions, because a memo *hit* is itself a full component-wise path equality.
This is the sharpest form of the structural review’s S1 lesson: the producer knew the
parent; every representation of that knowledge as a path re-derives it at cost.
`fdu-2ubt`’s batch-shaped observation (`parent: EntryId` + names) deletes the compare,
the per-op `PathBuf`, and the memo itself — another reason it belongs inside the one
structural experiment, not as a follow-up.

### The probe’s oracle contaminates what the counters and profiles say

The dhat profile of `scan-index` puts the top three allocation sites — 46% of allocation
events — inside the probe’s own digest oracle (`path_of` per entry plus `PathBuf`
assembly), and callgrind puts the oracle at ~39% of instructions.
The campaign status already warns that harness cost must be subtracted; what is new here
is that `FDU_COUNTERS=1` tallies *include* oracle allocations, so counter-derived
per-entry ratios (15.4 allocations/entry; the “9.7 reallocations per record” that
motivated `fdu-zgxd` on the load path) overstate engine work by a large, job-dependent
factor. Two cheap fixes: a `--no-oracle` probe mode for attribution runs (timing runs
keep the oracle), and scoping the counter guard to engine phases.
`fdu-zgxd` itself is resolved by the dhat runs: the reallocations are `PathBuf`
component-push growth — overwhelmingly the oracle’s, with the engine’s residue in
`Path::join` and `normalize` — not a hidden engine defect.

### The observation channel is unbounded, and nobody has measured its depth

Producers outrun the consumer by ~4× on Linux, and `std::sync::mpsc` imposes no
backpressure, so the queue is an unmeasured store of `Observation` batches at exactly
the moment RSS peaks.
A bounded channel (or the arena handoff above, which replaces the channel with subtree
splicing) caps it; before choosing, add a counter for peak queued observations — it may
explain a measurable slice of the 279 MiB.

## The cold Linux question has a mechanical answer

Guest-cold sweep (`sync; echo 3 > drop_caches` per sample; hypervisor caveat: guest-cold
orders strategies, it does not measure devices), three rounds:

| Scalar tier, guest-cold | Median wall |
| --- | ---: |
| fdu `--view summary` (automatic = 4 workers) | ~1,330 ms |
| diskus `-j 4` (equal threads) | ~1,900 ms |
| diskus `-j 12` (its 3×-cores default) | ~1,037 ms |

At equal concurrency fdu is already the faster engine cold; the entire deficit is that
diskus runs 3× cores and fdu’s adaptive unlock — calibrated between APFS regimes at 30
µs/entry — never fires against a Linux floor of ~1.5 µs warm (H84’s mechanism, observed
exactly). The index tier’s sweep on the same rounds showed 8 workers ≈ 12 workers ≈ 4
within noise cold, and 4 clearly best warm (1,734 / 1,876 / 1,904 ms at 4 / 8 / 12) —
the consumer, not the walk, binds that tier in both states, consistent with everything
above. So: retune `ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY` and the reserve depth per
platform table (`fdu-mjwr`, `fdu-tk1b`), gate on the existing first-chunk service-time
calibration, and expect the scalar-class cold gap to close for a constant.
Bare metal should confirm before the constant ships as evidence rather than inheritance.

## fdu-jnuo, measured and settled to a small number

The `elide` variant added to `walkspike.c` skips the terminating empty `getdents64` when
the previous call left ≥ 512 bytes of slack (a full buffer is the only other reason the
kernel stops early on in-tree filesystems).
On the 450k tree it removes **exactly one call per directory** — 57,260 → 28,630 — with
identical tallies, and the warm wall change is inside noise (~0–1%: ~29k syscalls ≈ 30
ms single-threaded, invisible at four workers).
Verdict to record: the mechanism is real, cheap, and *not worth a production path by
itself*; it needs a `statfs` `f_type` allowlist (FUSE/network filesystems may return
short buffers mid-stream), it only pays composed into a raw-`getdents64` path that has
some *other* reason to exist, and on Linux warm no such reason survives the scouting
measurements. Keep it for a cold or dir-heavy-topology campaign; stop carrying it as an
open “~50% of directory-read syscalls” line, which overstates it.

## The warm end-state: a snapshot the process can adopt instead of replay

Today’s warm numbers on this rig: load component ~410 ms, reconcile ~284 ms, save ~312
ms for a 28.7 MB snapshot of 450k records.
The load rebuilds what the save threw away: roll-ups are recomputed through
`merge_upward` per record, extension ids are re-interned per record, and every record
re-allocates a boxed entry plus two name copies — H78 (`fdu-pdra`) owns this and should
absorb three additions:

- **Persist the roll-ups and the interner.** They are deterministic functions of the
  tree the snapshot already pins; storing them converts load from re-derivation to
  adoption. (exp-008’s “ids are session-local” was a constraint of the current loader,
  not of the format.)
- **Per-block checksums with a tail index** (H35’s shape) so verification can be lazy
  per block while corruption still fails closed — the fail-closed rule requires
  checksum-before-trust, not checksum-before-*everything*.
- **Fixed-width records aligned for direct use**, so a future mmap loader (compose with
  H61’s immutable base + overlay) starts the process at reconcile-only warm cost: on
  this rig that is the difference between ~700 ms and ~300 ms warm opens, before the
  reconcile itself gets faster.

With hardware CRC32C and the `fdu-niuz` clone removal, the whole warm write-read cycle
drops by a large constant without touching the format; with H78+H61 it changes class.

## What transfers to macOS, and what macOS needs that Linux cannot give it

The consumer redesign, the extension memo, the journal-on-bootstrap fix, hardware
CRC32C, the snapshot format, and the save-clone removal are platform-independent — macOS
pays the same consumer and warm-path costs, merely hidden behind a faster enumeration
layer (exp-054 showed the Linux campaign’s consumer wins carried to macOS warm at
−15.7%). The thread-policy retune is Linux-specific; APFS’s measured knees stand.
The `getdents64` elision has no macOS analog (`getattrlistbulk` already batches).

What macOS uniquely needs is under the floor Linux never had: exp-041–046 established
that ~95% of macOS cold worker time is `open`+`getattrlistbulk`, so no user-space change
moves cold macOS much until the per-directory open itself goes.
That elevates **H77 (`searchfs`)** from “speculative, unscreened” to *the* scheduled
macOS spike: it is the only mechanism on the books that reads the catalog without
opening each directory.
It deserves a walkspike-style standalone instrument (subtree scoping via parent-id
reconstruction, permission-semantics audit — `searchfs` can return entries the caller
could not reach by traversal, which the trust rules must treat as a correctness
boundary, not a detail — plus non-UTF-8 names and probe-and-fallback), measured against
`dumac` before any engine work.
Finish H70’s quiet-host confirmation either way; a shared opener pool is worth 4% only
until `searchfs` makes openers unnecessary.

## Method notes, briefly

- **One bare-metal Linux box is the highest-leverage harness purchase available.** Every
  Linux number in the ledger is virtualized; H73 (inode-ordered statting, 4–6×
  literature claim), H76’s constant, and the closed-on-a-VM io_uring verdict are all
  cold-mechanism claims a hypervisor cannot test.
  One NVMe desktop settles all three.
- **Composite structural experiments are legitimate under the accept rule** — the rule
  gates a decision, and exp-022 already set the precedent at 542 lines.
  Write the hypothesis as the representation change, pre-register the differential
  harness as the oracle, and stop re-deriving why four 2% pieces of one 4× change each
  failed a 3% gate.
- **The aggregate tier still has no probe job** (`fdu-tyjx`): the tier where fdu fights
  diskus cannot enter the ledger.
  It is an afternoon of harness work and it gates the thread-policy experiment above; do
  it first.
- **Try PGO once.** The release profile already takes fat LTO and one codegen unit;
  profile-guided optimization on the branchy consumer typically returns another 5–15%
  for zero source change.
  If it clears the bar, wire it into release builds only; a failed experiment costs an
  afternoon and closes a standing unknown.

## The queue, re-ordered by this evidence

| Order | Item | Expected size (this rig) | Blocked on |
| --- | --- | ---: | --- |
| 1 | `fdu-tyjx` aggregate probe job | unblocks two rows below | nothing |
| 2 | Linux cold thread policy (`fdu-tk1b`/`fdu-mjwr`, H76/H84) | ~22% cold scalar | 1 |
| 3 | Save-path: `fdu-niuz` clone off critical path + hardware CRC32C | large slice of the 56% write tax | nothing |
| 4 | **Consumer representation, one experiment** (S1–S4 + H19–22 + H60 + `fdu-2ubt` + ext-memo + journal flag) | ~4× tree view, ~12× RSS | nothing; supersedes piecemeal forms |
| 5 | `fdu-926e` content classification by `ExtId` | ~34% of warm content open | nothing |
| 6 | H78 snapshot format + persisted roll-ups/interner (+H35 blocks, then H61 overlay) | warm open → reconcile-bound | shrinks after 4 |
| 7 | H77 `searchfs` standalone spike (macOS) | the only sub-floor macOS cold lever | instrument + audit |
| 8 | Bare-metal replication: H73, io_uring-cold, cold constants | evidence class, not speed | hardware |
| — | `fdu-jnuo` elision | measured: syscalls −50%, wall ~0 warm | keep for cold campaigns only |

Items 2 and 3 are constants and mechanics; item 4 is the campaign’s next real chapter,
and everything after it should be re-screened once it lands, because it will eat several
queued numbers — which is, by now, this ledger’s most reliably confirmed hypothesis of
all.

## Reproduction

```shell
# Tree (same generator and scale as the scouting doc)
python3 benchmarks/spikes/gen_tree.py /tmp/fdu-spike/tree 450000

# Consumer floor beside the walkers
rustc -O -o arena_spike benchmarks/spikes/arena_spike.rs
./arena_spike /tmp/fdu-spike/tree 4

# getdents64 elision census
gcc -O2 -o walkspike benchmarks/spikes/walkspike.c
./walkspike statx /tmp/fdu-spike/tree && ./walkspike elide /tmp/fdu-spike/tree

# Write-tax comparison
target/release/fdu --cache off /tmp/fdu-spike/tree
target/release/fdu --cache refresh /tmp/fdu-spike/tree

# Attribution with the oracle identified (subtract before quoting)
valgrind --tool=dhat --dhat-out-file=dhat.out \
  target/profiling/examples/perf_probe scan-index --root /tmp/fdu-spike/tree
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
