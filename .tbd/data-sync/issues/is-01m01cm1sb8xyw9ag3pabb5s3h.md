---
type: is
id: is-01m01cm1sb8xyw9ag3pabb5s3h
title: Stabilize adaptive scan scaling on heterogeneous macOS trees
kind: bug
status: open
priority: 1
version: 14
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - macos
  - fix
  - performance
dependencies:
  - type: blocks
    target: is-01m01ed61j7yty2bqp0zw8v0xc
  - type: blocks
    target: is-01kzy1w2vbam0mr1z5we4y6fy0
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:19:49.615Z
updated_at: 2026-08-15T01:17:12.271Z
---
Resolve the production defect conditionally from fdu-9x4o. If an independently confirmed controller exists, implement only that design; do not retune the 30-microsecond constant or replace automatic mode with a fixed CPU count without its evidence. If no candidate qualifies or profiling falsifies the diagnosis, make no production behavior change and close this bead with the no-change evidence and any narrowly scoped follow-up.

Acceptance for an implementation: the legacy completion-order model demonstrates the old violation and the selected policy passes invariant tests; explicit thread counts retain exact documented semantics; hardware remains a safe bound; queue shutdown, panic, consumer disconnect, partial results, macOS bulk/fallback, one-filesystem, and traversal-order behavior remain correct; the held-out topology/host matrix passes the pre-registered stability, resource, and +3% noninferiority/non-regression gates; rejected prototypes are absent; make check and make cross-lint pass. Claims are limited to measured Apple Silicon/local APFS regimes, with Intel Mac and non-APFS behavior identified as inherited or unproven unless separately measured.

## Notes

Observed defect: automatic mode discards calibration after one completion-ordered 16,384-entry window, producing materially different effective concurrency and latency on a heterogeneous, partial Application Support diagnostic. Existing post-bulk evidence covered one M1 Pro/APFS warm-steady regime and did not record policy history, so it cannot establish a universal fix. Follow fdu-5rpt’s refined graph and decision rules; implementation is contingent on independent confirmation, and a documented no-change resolution is acceptable.
