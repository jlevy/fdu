---
type: is
id: is-01m0y1sdsamq2hrrexxcvx98g3
title: Add coherent bounded reads and maintained projections
kind: feature
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
delegate: codex@spud10.local
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sf2nph021wtx28p8ahxh
  - type: blocks
    target: is-01m0yhq8268z0qrza1fnwrddfm
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
hold: null
hold_until: null
created_at: 2026-08-26T03:28:29.993Z
updated_at: 2026-08-26T13:52:06.207Z
started_at: 2026-08-26T12:41:14.531Z
closed_at: 2026-08-26T13:52:06.206Z
close_reason: Coherent bounded reads, paging, maintained projections, work accounting, portable diagnostics, and regression coverage implemented; full handoff and cross-target gates pass.
resolution: null
duplicate_of: null
---
Implement ReadRequest and ReadResponse under one version/state boundary with lookup, depth-one tree, flat rows, roll-up/report, diagnostics, exact-or-capped counts, work accounting, portable-path issues, and the minimal commit-maintained indexes justified by the current contract.

## Notes

Expose complete production read values to the shared scenario envelope and add no alternate renderer-side result model; fdu-0kv7 composes all projection scenarios after Phase 2.

Implemented one-guard ReadRequest/ReadResponse with version pinning; bounded lookup, depth-one tree, flat paging, scalar rollups, reports, diagnostics, counts, and work accounting; handle-local bounded continuations; maintained portable indexes reconstructed from snapshots; and escaped byte-capped portable-path issue examples.

Precommit review found and fixed: intra-batch maintained-index drift, unbounded rollup maps, unbounded portable examples, uncharged tree path traversal, continuation consumption on query limit or later malformed projections, report-view bounds, arbitrary page defaults, and Rust 1.85 let-chain incompatibility. make check and make cross-lint pass.
