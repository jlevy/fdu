---
type: is
id: is-01kzg4ajxc0pvgcmj834gahcgt
title: "Snapshot format v1: compressed blocks, tail index, lazy directory listing"
kind: feature
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg4c6h9v2dzand7t090p278
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:27:45.580Z
updated_at: 2026-08-08T07:28:38.441Z
---
Replace format v0 (flat, uncompressed, read-it-all). The lifecycle invariants v0 established — engine-fingerprint invalidation, atomic temp+rename, corrupt-equals-empty, bounded allocation from declared counts — must all survive unchanged.

Modeled on ncdu 2's binary export, the strongest design found in the survey:
- Magic + version + endianness header. Pin any enum whose numeric value reaches the format.
- zstd-compressed data blocks, then an index block AT THE TAIL holding one (offset, length) pair per block plus the root reference. Opening is O(1): read only the tail.
- Blocks decompress on demand into a small LRU cache (ncdu 2 uses 8 slots), addressed by item reference.
- Item references as (block << k) | offset, delta-encoded when they point INSIDE the same block. ncdu's comment is blunt: full references compress badly and most references are local.
- Adapt block size upward (64 KiB -> 2 MiB) as the file grows, to bound index size.
- Write sibling groups CONTIGUOUSLY so one directory listing costs one block decompression. This is ncdu 2's documented TODO — do it from the start.
- Front-coded names against the previous sorted sibling (fsearch).
- u32 parent indices instead of pointers; rebuild on load.
- Store pre-computed roll-ups per directory so a query never re-aggregates (duc's lesson).
- Optional pre-sorted permutation arrays for instant sort-order switching, 4 bytes/entry/order.

Hard constraint: no row-per-file relational store on the hot path. gdu's SQLite backend measured 10-17x slower than its in-memory path.
