---
type: is
id: is-01kzx0mvryf0a938vhwhy36cjv
title: "H65: Retune worker depth for reduction-only scans"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01kzwk20kyaxajq254tee8apts
created_at: 2026-08-13T07:33:35.645Z
updated_at: 2026-08-13T10:25:01.627Z
closed_at: 2026-08-13T10:25:01.626Z
close_reason: Superseded because rejected H62 prerequisite did not land; indexed worker depth is already resolved.
---
After worker-local reduction removes the single index consumer, profile and measure automatic scan depth at 6, 8, 10, 12, and 16 workers on the frozen million-entry APFS tree. The indexed path remains at its accepted six-worker policy. Add a plan-specific policy only if paired wall improves >=3% with bounded CPU/RSS and exact semantic hashes; otherwise record and reject.

## Notes

Superseded without a measurement: H65 required the H62 reduction-only walker, which exp-041/042 rejected and reverted. The indexed worker-depth curve remains resolved by H57/exp-036; reopen only if a materially different reducer lands.
