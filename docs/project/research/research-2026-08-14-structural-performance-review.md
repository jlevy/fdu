# Research: What Thirty Hypotheses Have in Common, and What They Missed

**Date:** 2026-08-14

**Author:** fdu project, with Claude Code review assistance

**Status:** Analysis current; the ordering in [Order, and why](#order-and-why) is
superseded by
[the campaign-2 plan](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md),
which runs S1–S4 as one structural experiment (H86)

## Overview

Thirty hypotheses have been measured.
Ten were confirmed, eleven refuted, nine remain open.
That record is unusually good — the refutations are recorded as carefully as the wins,
and the queue is honest about what it does not know.

This note asks a different question: **what do those thirty have in common, and what
does that shared shape hide?**

The answer is that almost every hypothesis optimizes *within* one architecture — a
parallel producer emits path-addressed observations, a single consumer arbitrates them
through the delta contract and merges roll-ups upward — and tunes one of its parts: more
workers, fewer syscalls, cheaper keys, better layout, a different allocator.
Very few question the shape itself.

That is not a criticism of the campaign; tuning inside a working architecture is how it
got twice as fast. But it means the untested space is not a list of smaller constants.
It is a small number of structural facts that every tier pays for, and this note names
seven of them with the evidence that motivates each.

One of the seven is not speculative at all.
It is a direct transplant of a change that was measured at −51.9% four commits ago, into
a path that still has the defect it removed.

## The unifying observation

Nearly every remaining cost is the engine **re-deriving something the caller already
held**.

The snapshot loader was the clearest case: it had the parent’s `EntryId` in a local
variable and then spent a `PathBuf` join, a `normalize` vector, and a descent from the
root through one `BTreeMap` lookup per level to rediscover it.
Removing that re-derivation took snapshot load down 51.9% and warm open down 41.9%.

The same pattern appears in at least three other places, and the sections below are
ordered by how much evidence there is that each one matters.

## S1. The cold scan re-derives the parent exactly as the loader did

**This is the same defect, in the other producer, still unfixed.**

`scan.rs` emits `Op::Upsert { path: rel_path.clone(), kind, attrs }` for every entry.
`Index::apply_upsert` then calls `normalize(path)` to split it into components and
`ensure_dir_chain(ancestors)` to walk down from the root, one map lookup per level — to
arrive at a directory the worker was standing in when it produced the record.

A callgrind profile of a single-threaded `scan-index` over a 17,100-entry tree, with the
probe’s own oracle digest backed out, attributes:

| Cost | Share of engine work |
| --- | ---: |
| Allocator (`malloc`/`free`/`realloc` family) | **~35%** |
| `std::path::Components::next` | **~13%** |
| `memcmp` (name comparison) | ~4% |
| `apply_validated` itself | ~4% |
| `memcpy` | ~4% |
| `merge_upward` | ~2% |

The caller tree, not the flat view, is what identifies the driver: `apply_validated`
accounts for **426,818** path-component comparisons across roughly 17,100 entries, about
**25 per entry**. That is the descent, paid once per file.

For comparison, the snapshot loader before its fix profiled at 27.5% allocator and 15%
path iteration — the same shape, and it yielded 51.9%.

**The change.** Give the observation contract a parent-relative form —
`Op::UpsertUnder { parent: EntryId, name, kind, attrs }` — and have the walker emit it
for every entry below a directory it has already established.
This does not bypass the delta contract, and that distinction matters: the index still
arbitrates, conditional generation and revision guards still apply, and the observation
becomes *more* precise rather than less.
Reconciliation already carries parent-relative expectations, so the concept is present
in the engine; the cold producer simply does not use it.

**Predicted:** `cold-scan-index` wall down at least 15%, with `user_cpu_ns` and
`minor_faults` down substantially; `cold-scan-producer` unchanged, because this is
entirely consumer-side work.
**Tier:** index. The aggregate tier retains no path records and does not pay this.

## S2. Every name is allocated twice and stored twice

An entry owns `name: OsString`; its parent’s `children: BTreeMap<OsString, EntryId>`
owns the same bytes again.
Two heap allocations and two copies per entry, for one name.

H19–H22 mentions removing duplicate name storage as one arm of a compaction experiment,
but it has never been measured, and the stronger form is not in the registry at all: a
**single name arena**, one growable byte buffer for the whole index, with entries
holding `(offset: u32, len: u16)`. That is what `fsearch` does, and it takes per-entry
name allocation from two to zero while making sibling names contiguous in memory.

The allocator is ~35% of cold-scan engine work and was 27.5% of snapshot-load work, so
this is aimed at the largest single line in both profiles.
It also composes with S1 rather than competing: S1 removes the *path* allocations, S2
removes the *name* allocations.

**Predicted:** million-entry RSS down at least 20%; cold indexed wall down at least 3%.
**Tier:** index and content, which both retain entries.

## S3. Children want a sorted arena slice, not a per-node map

Every directory allocates its own `BTreeMap`, which is several nodes’ worth of
allocation, for a set of children that arrived together in one `getdents64` batch and is
then read in sorted order.

A contiguous sorted slice in an arena gives one allocation per directory instead of
several, binary-search lookup, and locality for the roll-up merge that walks it.
It composes with S2: the slice holds `(name_offset, name_len, EntryId)` and the names
sit adjacent in the same arena.

H7 — swap the `BTreeMap` for a hash map with a cheap hasher — has been in the registry
since the beginning with status “—”, never tested.
It is the weak form of this idea: it changes the lookup constant but keeps the per-node
allocation and loses the sorted iteration order that snapshots and goldens depend on.
The arena form keeps ordering, removes the allocation, and improves locality, so H7
should be superseded rather than scheduled.

**Predicted:** cold indexed wall down at least 5%; RSS down; sorted iteration order and
snapshot bytes unchanged.
**Tier:** index and content.

## S4. A cold bootstrap does not need arbitration at all

The single mutation authority exists so that snapshots, queries, and change feeds cannot
diverge, and so that concurrent producers cannot race.
Both concerns are real — for a **warm** path.

A cold scan has no prior state to arbitrate against, no concurrent readers, and no
present-state ABA to reject: it is constructing a tree from nothing.
Yet every entry still crosses a channel as an allocated observation, is applied by one
serialized consumer, and merges its contribution to the root.

The Linux scouting measured the consequence directly: fdu’s index consumer costs about
**2.3 µs/entry of user CPU against dut’s ~0.1 µs**, roughly twenty times, and that gap
is the whole tree-class deficit on Linux.
No syscall change addresses it, which the enumeration measurements already established.

H60 (worker-local subtree arenas) is queued and points at this, but frames it as a
construction optimization.
The sharper framing is that the cold path can drop arbitration entirely: workers build
disjoint subtrees in local arenas with no coordination, splice them at region
boundaries, and **one bottom-up pass** computes every roll-up.
That removes the channel, the per-entry observation allocation, the consumer
serialization, and `merge_upward`’s O(N·D) in a single change.

This is the largest and riskiest item here, and it should be attempted only after S1–S3,
both because they are cheaper and because they will shrink it: S1 removes the path work
the consumer does, S2 and S3 remove the allocation it does, and what remains is the
serialization itself, measured against a much lower floor.

**Predicted:** cold indexed wall down at least 20% on Linux; parity proven by the
existing `assert_same_image` differential harness at every worker count.
**Tier:** index.

## S5. Roll-ups are merged per entry rather than computed once

`merge_upward` walks to the root for every entry, so a depth-20 tree merges twenty times
per file.
H13 proposed accumulating consecutive same-parent contributions and was refuted
at −2.5% — but that was measured *after* H18’s interning had already taken the expensive
part of each merge, and the two were competing for one cost.

The profile now puts `merge_upward` at about 2% of cold-scan engine work, so as a
standalone change it is not worth doing, and this note does not propose it as one.
It is listed because **S4 makes it free**: a bottom-up pass computes each directory’s
roll-up once from its children, and the O(N·D) disappears as a consequence rather than
as a target. That is worth stating so nobody schedules it separately and measures noise.

## S6. Per-directory extension tallies are retained everywhere and read almost nowhere

Every directory carries `by_ext`, a map from interned extension id to tally, and
`merge_upward` merges that *map* at every ancestor of every file.

Most invocations read extension tallies for one directory — the root — or for the
handful a `--view types` renders.
Retaining a map per directory and merging it per entry per level is a large memory and
merge cost for a projection that is almost never read at depth.

Two variants worth screening independently:

- **Compact representation:** a sorted `(ExtId, ExtTally)` slice instead of a
  `BTreeMap`, which most directories can hold in a handful of entries.
- **Deferred computation:** retain tallies only at the root and compute a directory’s on
  demand by subtree traversal when a query actually asks.
  This trades a rare query’s cost for every scan’s cost, and needs the query surface
  checked for who reads `rollup_of(...).by_ext` at depth.

Neither is in the registry.
RSS at million scale is the clearest remaining defect, and this is retained state that
scales with directories × distinct extensions.

**Predicted:** million-entry RSS down at least 15%; cold indexed wall down at least 3%.
**Tier:** index.

## S7. Classification runs over every file on every content open

Already filed as `fdu-926e` and repeated here because it is the largest known target on
any platform: `Index::analysis_candidates` calls `classify_path` for every file each
time it enumerates candidates, including on a `--cache only` run whose sidecar already
stores the classification that then replaces it.

About 34% of a warm content open, reached through roughly 96 comparisons per file — the
signature of a linear scan over the type-rules table.
The index already interns each file’s extension as an `ExtId`; classification could key
off that instead of re-deriving from the path.

**Tier:** content.

## What this list is not

It is not a plan to change what fdu computes.
Every item above preserves the index, the roll-ups, the provenance, the snapshot bytes,
and the query surface.
S1 makes an observation more precise rather than weaker; S2 and S3 change representation
behind accessors; S4 changes when arbitration happens, not whether the result is
arbitrated; S6 is the only one that touches what is retained, and its deferred variant
must prove no reader regresses.

It is also not a claim that all seven will pay.
This campaign has twice found a landed change consuming a queued hypothesis’s headroom —
H13 lost to H18, and H74 lost to `fdu-91ts` on a path a profile had put it at 27.5% of.
S1 through S4 all attack overlapping costs on the same path, so **each must be
re-screened after the one before it lands**, and the later ones should be expected to
shrink.

## Order, and why

1. **S1**, because it is a transplant of a measured −51.9% into a path with the same
   defect, and because it shrinks S4 before S4 is attempted.
2. **S7**, because it is the largest known content-tier cost and independent of the
   rest.
3. **S3 then S2**, because they compose, are behind accessors, and attack the allocator
   line that dominates both profiles.
4. **S6**, screened as two independent variants.
5. **S4**, last, measured against whatever floor S1–S3 leave.

Two harness gaps gate the evidence rather than the work.
`fdu-tyjx` — the aggregate tier has no probe job, so nothing above can be measured on
the tier where fdu competes with `diskus` — and `fdu-fz3j`, the profiler runs on macOS
only, so two of three supported platforms cannot take the first step of the loop.
Both multiply what every later experiment can settle.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
