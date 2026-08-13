---
type: is
id: is-01kzx0mvryf0a938vhwhy36cjv
title: "H65: Retune worker depth for reduction-only scans"
kind: task
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01kzwk20kyaxajq254tee8apts
created_at: 2026-08-13T07:33:35.645Z
updated_at: 2026-08-13T10:28:29.082Z
closed_at: 2026-08-13T10:25:01.626Z
close_reason: Superseded because rejected H62 prerequisite did not land; indexed worker depth is already resolved.
---
After worker-local reduction removes the single index consumer, profile and measure automatic scan depth at 6, 8, 10, 12, and 16 workers on the frozen million-entry APFS tree. The indexed path remains at its accepted six-worker policy. Add a plan-specific policy only if paired wall improves >=3% with bounded CPU/RSS and exact semantic hashes; otherwise record and reject.

## Notes

Reopened before H64: H59 itself already removes index construction, so exact-summary worker depth can differ even though H62 did not land. Screen fixed 8/10/12 summary-only pools against committed H59 automatic/six, then run a paired >=16-trial confirmation only if an arm plausibly clears 3%. Indexed policy remains untouched.
