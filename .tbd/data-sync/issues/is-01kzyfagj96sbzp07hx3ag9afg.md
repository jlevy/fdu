---
type: is
id: is-01kzyfagj96sbzp07hx3ag9afg
title: "H78: zero-copy snapshot format so warm open is reconcile-bound"
kind: task
status: open
priority: 2
version: 1
labels:
  - performance
dependencies: []
created_at: 2026-08-13T21:09:19.561Z
updated_at: 2026-08-13T21:09:19.561Z
---
H10's remaining half. After fdu-91ts makes snapshot load O(N) rather than O(N*D), the residue is still parse-and-rebuild: every record allocates and inserts. A format whose on-disk layout can be used directly — mmap plus offset-addressed records, with the roll-up state persisted rather than recomputed — would make warm open latency-bound on the reconcile walk instead of the load. That is the only route to warm-beats-cold on Linux, which has no persistent change journal to scope the sweep (FSEvents does that job on macOS). Constraints: exact snapshot semantics, a completeness boundary in the format version, endianness and alignment discipline, and fail-closed rejection of corrupt images without allocating from untrusted counts.
