---
type: is
id: is-01m0yhq8268z0qrza1fnwrddfm
title: Complete the opened-root session goldens and contract coverage gate
kind: task
status: in_progress
priority: 1
version: 8
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sf2nph021wtx28p8ahxh
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T08:06:55.813Z
updated_at: 2026-08-27T02:26:56.135Z
closed_at: 2026-08-27T01:48:23.090Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
Compose the test seams added with each Phase 2 capability into one deterministic runner over the real OpenedIndex owner, workers, exact commit pipeline, journal, and five synchronous operations. Record the five bounded transparent-box session goldens; compare every commit with the independent model; derive automatic public-contract coverage from observed outcomes; add named-only update, lint, size, unstable-literal, duplicate, and orphan checks; prove deterministic barriers, recovery, continuation, and joined shutdown without a runtime trace bus or fact injection. This bead blocks the Python surface so the native contract is complete and reviewable first.

## Notes

CI run 33031258054 first exposed host read_dir ordering in cold-progressive-knowledge. Commit 8166cf1 added a per-open test-only deterministic discovery scheduler that sorts real directory entries before production admission while leaving non-test discovery streaming and recording exact public commits. CI run 33032529592 then exposed the same portability class in coherent-projections-and-continuations, showing scenario-specific enablement was insufficient. The golden runner now constructs all TestControls through one deterministic_controls helper, so every session receives the same portable input schedule by construction; only coherent-projections-and-continuations required a new expected root commit. Exact-tree local validation passes: focused all-feature golden test; golden lint (5 sessions/159 records); full make check including all feature combinations, MSRV, Python wheel/sdist smoke, and CLI/Python parity; plus x86_64 Apple/Windows cross-lint. Awaiting replacement GitHub CI before re-closing.
