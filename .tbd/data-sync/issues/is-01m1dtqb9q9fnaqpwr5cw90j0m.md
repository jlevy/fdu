---
type: is
id: is-01m1dtqb9q9fnaqpwr5cw90j0m
title: Fix controls-on snapshot reuse for controls-off public opens
kind: bug
status: closed
priority: 0
version: 7
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - correctness
  - cache
dependencies:
  - type: blocks
    target: is-01m1dtqkbyrydtq60902w1sgkr
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-01T06:32:52.790Z
updated_at: 2026-09-13T21:52:05.629Z
closed_at: 2026-09-01T07:43:27.976Z
close_reason: Exact public snapshot ownership and report-only projection are implemented, fully gated, committed, pushed, and green in all 19 GitHub checks.
resolution: null
duplicate_of: null
---
Add red tests for Auto and Only, require exact scope for every returned Index, and confine any directional controls-on to controls-off reuse to a consumed report projection. Public opens must neither fail late with ScanScopeMismatch nor return retained control state under a controls-off scope.

## Notes

Implemented in commit 5d4a6e2 on codex/streaming-performance-parity. Red-green coverage proved public Auto late ScanScopeMismatch and public Only incompatible-index escape. Final design keeps every returned Index exact, treats scanning report mismatches as cold misses, and permits controls-on to controls-off projection only for no-scan reports; the Report is retagged to requested scope. A nine-view JSON parity test matches a cold controls-off report. Focused scope tests (4), watch-persistence integration tests (2), isolated full make check, and all 19 GitHub checks for run 33482895018 passed.

2026-09-13: PR #51 review 5192254822 finding COMMIT-1 (High) is this defect, raised against the PR that introduced it (the directional acceptance in 6d45be9). Ported down to PR #51 as c0729ce (git cherry-pick -x bcb27ca); the code applied unchanged and #52's plan-document edits were left out. The review's proof test is controls_on_snapshot_does_not_serve_controls_off_auto_open (asserts ColdScan). Red on 19c0d73 with ScanScopeMismatch; lib and execution suites green after the port; all 19 CI checks green at 6d2d964.
