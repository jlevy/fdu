---
type: is
id: is-01kzwxffnkwm7kkmrdpgn5rbsn
title: Compare cache-off FDU summary with dumac
kind: task
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - benchmark
  - dumac
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-13T06:38:13.682Z
updated_at: 2026-08-13T11:58:29.767Z
---
Run a claim-grade adjacent paired comparison of fdu --cache off --view summary against dumac on the canonical million-scale workspace. Distinguish no persisted snapshot from no retained in-memory index, validate output semantics and hard-link accounting, publish the result, and decide whether H59 should include a true transient summary-only library path.

## Notes

Claim-grade current comparison now additionally requires every FDU summary sample to agree with an independent Python fingerprint on files, descendant directories, apparent bytes, and allocated bytes. Harness unit suite: 64 passing.
