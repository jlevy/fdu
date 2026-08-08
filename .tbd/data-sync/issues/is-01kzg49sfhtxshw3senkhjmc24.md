---
type: is
id: is-01kzg49sfhtxshw3senkhjmc24
title: "Packed entry records: hit the 25-32 bytes per file budget"
kind: feature
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg4ajxc0pvgcmj834gahcgt
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:27:19.537Z
updated_at: 2026-08-08T07:27:45.580Z
---
The current Entry uses String names and a BTreeMap of children per directory, which is nowhere near the memory target. ncdu 2 reaches 25 bytes per regular file and 56-64 per directory; a full root-filesystem scan dropped 429 MB -> 162 MB.

Techniques:
- Parent-pointer tree, name-only storage, paths reconstructed on demand (already the shape; keep it).
- Single allocation for record + variable-length name, from a per-thread arena never individually freed.
- Optional attributes packed contiguously behind a flags word, offsets computed from the flags — pay only for requested fields (fsearch). ncdu 2 places the optional block BEFORE the record so the canonical pointer never moves.
- Steal bits from a wide counter rather than adding a flags byte: ncdu 2 packs a 3-bit type, a presence bit, and a 60-bit block count into one u64.
- Intern device IDs into a small global table, store a narrow index, not a raw 64-bit st_dev.
- Zero padding: fully packed byte-aligned records, accepting slightly worse codegen for materially better memory.
- Chunked arrays rather than one monolithic vector, to avoid realloc pressure during live updates.

fsearch is GPL: ideas only.
