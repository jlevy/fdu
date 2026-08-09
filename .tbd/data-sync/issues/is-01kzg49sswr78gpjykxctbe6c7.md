---
type: is
id: is-01kzg49sswr78gpjykxctbe6c7
title: "Reducer registry: make metrics registrations, not engine changes"
kind: feature
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:27:19.867Z
updated_at: 2026-08-09T18:28:33.966Z
---
Today RollUp is a fixed struct. Goal 6 requires new roll-up dimensions to be registrations against a stable interface.

A metric declares: id, version, input tier (stat-only vs content), a commutative merge, and — critically — whether it is INVERTIBLE. That flag is not cosmetic; it decides update cost:
- Invertible (sums, counts, count-by-key): differential apply in O(depth).
- Non-invertible (max/min mtime, top-k): absorb additions in O(depth), but removal may need the directory re-merged from its direct children.

Built-in stat tier: total bytes, allocated bytes, file/dir counts, newest/oldest mtime, mtime-recency histogram, size histogram, count-and-bytes by type, top-k largest, top-k most recent. Per-thread top-K heaps merged at the end, with early rejection against the heap minimum BEFORE allocating (dut).

Content tier is deferred but its place in the registry must be reserved now, because it shapes the snapshot format.

## Notes

Before the reducer registry becomes a stable API, define aggregate overflow policy. Counts and byte sums are u64 and currently use ordinary addition; a theoretical aggregate above u64 can wrap in optimized builds. Evaluate checked/wider accumulators versus explicit saturation together with invertibility and snapshot encoding.
