---
type: is
id: is-01m10nseqmb4w4n271gyre0xnp
title: Gate the thin adapter against contract, structure, concurrency, and clean installs
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies: []
parent_id: is-01m0y1sjbfs5h264xhme2vqymg
created_at: 2026-08-27T03:56:31.347Z
updated_at: 2026-08-27T03:56:31.347Z
---
Run the revised provider registry against fdu, add structural tests forbidding concrete-provider imports and adapter-owned inventory state, prove one bounded native read per application query, and test iterator cancellation, concurrent reads, reset after backpressure, root replacement, and joined close from an exact-revision installed wheel.
