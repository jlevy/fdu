---
type: is
id: is-01m0yhq8268z0qrza1fnwrddfm
title: Complete the opened-root session goldens and contract coverage gate
kind: task
status: in_progress
priority: 1
version: 11
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sf2nph021wtx28p8ahxh
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T08:06:55.813Z
updated_at: 2026-08-27T03:06:37.788Z
closed_at: 2026-08-27T01:48:23.090Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
Compose the test seams added with each Phase 2 capability into one deterministic runner over the real OpenedIndex owner, workers, exact commit pipeline, journal, and five synchronous operations. Record the five bounded transparent-box session goldens; compare every commit with the independent model; derive automatic public-contract coverage from observed outcomes; add named-only update, lint, size, unstable-literal, duplicate, and orphan checks; prove deterministic barriers, recovery, continuation, and joined shutdown without a runtime trace bus or fact injection. This bead blocks the Python surface so the native contract is complete and reviewable first.

## Notes

GitHub CI run 33034549905 exposed the last platform-specific Debug value after the LF/path fixes: std::time::SystemTime renders as { intervals: ... } on Windows and { tv_sec, tv_nsec } on Unix. Commit 86f64eb maps both forms to the audited closed token [SYSTEM_TIME], adds a platform-shape unit test, updates the one affected session golden, and extends the token lint. The exact-tree make check passed (including 523 all-feature core tests, all feature-boundary suites, golden/parity, MSRV, Python wheels/sdist smoke, docs, and audits). Awaiting replacement GitHub CI before re-closing.
