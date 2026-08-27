---
type: is
id: is-01m0yhq8268z0qrza1fnwrddfm
title: Complete the opened-root session goldens and contract coverage gate
kind: task
status: closed
priority: 1
version: 13
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sf2nph021wtx28p8ahxh
  - type: blocks
    target: is-01m10nq2bqxrkqxrtkh7rs668g
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T08:06:55.813Z
updated_at: 2026-08-27T03:55:13.141Z
closed_at: 2026-08-27T03:14:06.650Z
close_reason: "Completed at 86f64eb: the five deterministic opened-root session goldens, independent-model and contract-coverage gates, closed normalization vocabulary, named-only updates, and cross-platform portability fixes are complete. The exact-tree make check passed, and GitHub Actions run 33035356756 passed all 19 checks across macOS, Linux, and Windows."
resolution: null
duplicate_of: null
---
Compose the test seams added with each Phase 2 capability into one deterministic runner over the real OpenedIndex owner, workers, exact commit pipeline, journal, and five synchronous operations. Record the five bounded transparent-box session goldens; compare every commit with the independent model; derive automatic public-contract coverage from observed outcomes; add named-only update, lint, size, unstable-literal, duplicate, and orphan checks; prove deterministic barriers, recovery, continuation, and joined shutdown without a runtime trace bus or fact injection. This bead blocks the Python surface so the native contract is complete and reviewable first.

## Notes

GitHub CI run 33034549905 exposed the last platform-specific Debug value after the LF/path fixes: std::time::SystemTime renders as { intervals: ... } on Windows and { tv_sec, tv_nsec } on Unix. Commit 86f64eb maps both forms to the audited closed token [SYSTEM_TIME], adds a platform-shape unit test, updates the one affected session golden, and extends the token lint. The exact-tree make check passed (including 523 all-feature core tests, all feature-boundary suites, golden/parity, MSRV, Python wheels/sdist smoke, docs, and audits). Awaiting replacement GitHub CI before re-closing.
