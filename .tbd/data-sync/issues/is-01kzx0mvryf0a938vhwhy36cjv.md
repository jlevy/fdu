---
type: is
id: is-01kzx0mvryf0a938vhwhy36cjv
title: "H65: Retune worker depth for reduction-only scans"
kind: task
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01kzwk20kyaxajq254tee8apts
created_at: 2026-08-13T07:33:35.645Z
updated_at: 2026-08-13T10:44:43.914Z
closed_at: 2026-08-13T10:44:43.913Z
close_reason: Measured and rejected as exp-043; the promising eight-worker screen did not reproduce and resource costs regressed.
---
After worker-local reduction removes the single index consumer, profile and measure automatic scan depth at 6, 8, 10, 12, and 16 workers on the frozen million-entry APFS tree. The indexed path remains at its accepted six-worker policy. Add a plan-specific policy only if paired wall improves >=3% with bounded CPU/RSS and exact semantic hashes; otherwise record and reject.

## Notes

Rejected as exp-043. A 5-pair 901,963-entry curve made fixed eight workers look 5.2% faster; ten neutral, twelve +3.9% slower. A 16-pair confirmation left t8's interval crossing zero and t16 slower. Independent 720,805-entry 20-pair decision run: t8 wall +0.669% CI [-1.562%, +3.994%], CPU +40.66%, system CPU +42.03%, RSS +3.39%, involuntary context switches +76.66%; exact semantics and no invalid/drift/mutation. Compile-time hook removed; automatic/six retained.
