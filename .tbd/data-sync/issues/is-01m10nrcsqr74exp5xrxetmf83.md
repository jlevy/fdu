---
type: is
id: is-01m10nrcsqr74exp5xrxetmf83
title: Implement bounded handle-local continuation authority
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nrd50t3xfevcxx7j98x5h
parent_id: is-01m0y1shykye8sc7h7e9rkk6kh
created_at: 2026-08-27T03:55:56.598Z
updated_at: 2026-08-27T10:11:10.574Z
closed_at: 2026-08-27T10:11:10.563Z
close_reason: "Completed in 27aeed0: bounded handle-local continuation authority, atomic 64 KiB record cap, exact query/version ownership, proportional resume, restoration after projection errors, cleanup, full make check, and all 19 PR CI jobs passed."
resolution: null
duplicate_of: null
---
Implement opened/continuation.rs create/resume/evict/clear over version, normalized fdu-native query identity, and structural traversal position. Public IDs remain opaque; enforce table/record bounds, stale/query-mismatch/foreign/evicted/closed outcomes, proportional resume work, and close cleanup without historical images or signing.

## Notes

Retained the simple Phase 2 authority after design audit: handle-local opaque IDs, version/query-owned records, structural resume keys, 128-record oldest-first cap, single use, stale/foreign/evicted handling, and clear-on-close. Added a 64 KiB exact structural-footprint cap per cloned record (<=8 MiB table payload excluding map/allocator overhead), rejected atomically before ordinal advance or eviction, restored tokens after any projection error, and proved normalized filtered-query ownership with one-row resume work. Full make check passes: 530 all-feature core tests (529 passed, 1 ignored), 125 unchanged CLI goldens, opened-root goldens, Python/type/concurrency/wheel/sdist/parity, no-default feature matrices, MSRV, docs, audits, and release tests.
