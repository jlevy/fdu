---
type: is
id: is-01kzg4ajxc0pvgcmj834gahcgt
title: "Block snapshot format: compressed blocks, tail index, lazy directory listing"
kind: feature
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg4c6h9v2dzand7t090p278
  - type: blocks
    target: is-01kzg4d2fb96erw3h1b5k0c6xy
  - type: blocks
    target: is-01kzg4d256qmchmtyvttnpvn4y
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:27:45.580Z
updated_at: 2026-08-09T20:37:10.361Z
---
Replace the flat, uncompressed bootstrap format v2. Its lifecycle invariants—engine-fingerprint invalidation, semantic scope, payload integrity verification, exclusive temporary-file reservation plus atomic rename, corrupt-equals-empty behavior, complete-only persistence, and bounded allocation—must survive unchanged.

Modeled on ncdu 2 binary export, the strongest design found in the survey:
- Magic, version, and endianness header. Pin any enum whose numeric value reaches the format.
- zstd-compressed data blocks, then an index block at the tail holding one offset/length pair per block plus the root reference. Opening is O(1): read only the tail.
- Blocks decompress on demand into a small LRU cache (ncdu 2 uses 8 slots), addressed by item reference.
- Item references as (block << k) | offset, delta-encoded for same-block references.
- Adapt block size upward (64 KiB to 2 MiB) as the file grows, to bound index size.
- Write sibling groups contiguously so one directory listing costs one block decompression.
- Front-code names against the previous sorted sibling.
- Use u32 parent indices rather than pointers; rebuild on load.
- Store pre-computed roll-ups per directory so a query never re-aggregates.
- Consider optional pre-sorted permutation arrays for instant sort-order switching.

Hard constraint: no row-per-file relational store on the hot path. gdu SQLite measured 10–17x slower than its in-memory path.
