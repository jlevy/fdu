---
type: is
id: is-01m0y1sed8zrkf6hdnp5wrq5ty
title: Add bounded verified multi-path refresh
kind: feature
status: closed
priority: 1
version: 10
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
updated_at: 2026-08-26T19:32:50.286Z
closed_at: 2026-08-26T19:32:50.285Z
close_reason: Implemented in d5d9151. The complete local handoff gate, cross-platform lint, no-default Clippy, and all 19 GitHub checks in run 33005096442 pass.
resolution: null
duplicate_of: null
---
Implement refresh(paths) validation, canonicalization, deduplication, ancestor collapse, widening, I/O outside the write guard, conditional exact commits, typed accepted/rejected paths, journal range, state, and work. Reuse the sound algorithms and fixtures from PR #47 d19b0ce.

## Notes

Implementation complete pending pushed CI. Added OpenedIndex::refresh for bounded canonical multi-path verification, typed accepted/rejected inputs, ancestor collapse and widening, one safe (after, version] journal interval, bounded issues, and transparent filesystem/commit work. Reused PR #47's d19b0ce reconciliation structure but replaced its per-path bound refusal, direct resource accounting, and delta receipt with the rewrite's exact commit, lifecycle, and shared-budget boundaries. Discovery and refresh now arbitrate max_files atomically under the index write lock; first refusal publishes the stopped/budget state in the same commit, retained-file progress stays exact after removals, and later stopped refreshes allow only proven non-expanding work. Close cancels and waits for active refreshes. Tests cover no-op work, duplicate/ancestor collapse, invalid/hidden paths, ancestor kind replacement, exact interval impact including concurrent producers, stale arbitration, discovery/refresh budget races, oversize preflight, stopped probes, close races, and .gitignore create/edit/delete. Precommit review fixed a Rust 1.85-incompatible let-chain, concurrent-interval impact loss, invisible no-op and resource-probe work, duplicate retained budget issues, and direct hidden-control deletion. Validation is green: focused all-feature refresh regression; complete make check including all feature matrices, Rust 1.85 tests, goldens, Python wheel/sdist/parity, docs and audits; make cross-lint for x86_64 macOS and Windows; and no-default-features Clippy with warnings denied.
