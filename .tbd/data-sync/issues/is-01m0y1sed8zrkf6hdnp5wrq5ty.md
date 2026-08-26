---
type: is
id: is-01m0y1sed8zrkf6hdnp5wrq5ty
title: Add bounded verified multi-path refresh
kind: feature
status: in_progress
priority: 1
version: 8
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
delegate: codex@spud10.local
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1seqtkvcawjhawny979ry
  - type: blocks
    target: is-01m0y1sf2nph021wtx28p8ahxh
  - type: blocks
    target: is-01m0yhq8268z0qrza1fnwrddfm
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:30.631Z
updated_at: 2026-08-26T17:56:24.762Z
---
Implement refresh(paths) validation, canonicalization, deduplication, ancestor collapse, widening, I/O outside the write guard, conditional exact commits, typed accepted/rejected paths, journal range, state, and work. Reuse the sound algorithms and fixtures from PR #47 d19b0ce.

## Notes

Resume from the four-file uncommitted refresh slice in engine_contract.rs, lib.rs, opened.rs, and scan.rs. Preserve the named after-verification/before-conditional-commit test barrier for fdu-0kv7. Before commit, resolve three review findings: RefreshResult.work must account for verified no-op/stale work even when no Commit is emitted; ResourceBudget rejection must correspond to a resource-stopped/full retained set or shared commit-boundary capacity accounting, not merely the presence of max_files; and the scan.rs candidate-kind predicate must use matches! so clippy passes. Then add the bounded path/ancestor/widening/budget/concurrency regressions, run the focused no-default/all-feature suites, make check, and cross-lint.
