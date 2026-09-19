---
type: is
id: is-01m2w71cmphbn0k2pk3w9xytk8
title: "H114: skip type-id String alloc on ContentRollUp::add"
kind: task
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m0py2a8eb90n6r21f4hygyvr
created_at: 2026-09-19T06:53:08.628Z
updated_at: 2026-09-19T07:03:56.624Z
closed_at: 2026-09-19T07:03:56.623Z
close_reason: "exp-111 / H114 rejected: content-cache-hit wall -0.56% [-17.92%, +4.79%]; type-id String alloc trim reverted. Remaining named cut is H115 / fdu-wx15."
resolution: null
duplicate_of: null
---
H83 apply/install increment after H112. ContentRollUp::add allocates a type-id String on every ancestor merge via entry(to_string()) even when the type is already in the map. get_mut-first, the same pattern merge_ancestors already uses for PathBuf. Not parse, not candidates, not install_controls, not H113.

## Notes

2026-09-19 pre-register exp-111 / H114 (H83 apply/install increment).

Claim: ContentRollUp::add allocates a type-id String on every ancestor merge
(entry(to_string())) even when that type is already in the map. get_mut before
entry, the same pattern merge_ancestors already uses for PathBuf, removes that
alloc on the common path. exp-108 counted 1,121,963 roll-up merges on this
subject; allocator was 24.47% of harness samples.

Not parse. Not candidates. Not install_controls. Not H113 file-count shortcut.
Not an instruction-only trim (H103): this is allocate-and-drop of an owned key.

Metric: content-cache-hit wall on nominated metabrowser-clone (same subject as
exp-108/109/110). Direction: down.
Accept: median at least 3% faster and the 95% paired interval entirely below
zero; content digest identical to exp-108 (3b8cfa71…).
Control: current HEAD (c06d09e7), H112 timers in the binary and off.
FDU_COUNTERS unset on the claim-grade pair.
Quiet if the cell holds; else uncontrolled. 25% bar not lowered. No RAM disk.

If reject: revert the engine change. H83 stays open for structural apply
(fdu-jxhk one-pass / EntryId). Do not retry another alloc-trim on this map.
