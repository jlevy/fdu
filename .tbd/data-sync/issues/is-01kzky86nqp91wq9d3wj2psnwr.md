---
type: is
id: is-01kzky86nqp91wq9d3wj2psnwr
title: Exercise snapshot parsing and commit failures as a state machine
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - testing
  - filesystem
  - correctness
dependencies:
  - type: blocks
    target: is-01kzg4ajxc0pvgcmj834gahcgt
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T18:58:33.782Z
updated_at: 2026-08-09T21:05:12.706Z
---
Extend the strong targeted snapshot tests with reusable parser and commit-failure state evidence before format v3. Mutate and truncate valid seeds across headers, counts, names, records, checksums, and trailers while proving bounded fail-closed behavior with no panic or allocation from unchecked lengths. Add a narrow commit seam that injects temporary creation, write, flush, sync, rename, and cleanup failures; prove the prior complete snapshot remains authoritative before commit and temporary state is bounded and recoverable. The cross-thread old-or-new reader visibility proof and shared-handle lock-release contract are owned by fdu-gd6n and fdu-s7wr; reuse their fixture rather than duplicating it here. State whether the persistence contract is atomic visibility or crash durability. Consolidate the duplicated FNV-style hashing into one named stable primitive, correct the nonstandard multiplier with an intentional cache miss or format bump, and add known vectors.
