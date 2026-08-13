---
type: is
id: is-01kzx0mvryf0a938vhwhy36cjv
title: "H65: Retune worker depth for reduction-only scans"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01kzwk20kyaxajq254tee8apts
created_at: 2026-08-13T07:33:35.645Z
updated_at: 2026-08-13T07:33:35.645Z
---
After worker-local reduction removes the single index consumer, profile and measure automatic scan depth at 6, 8, 10, 12, and 16 workers on the frozen million-entry APFS tree. The indexed path remains at its accepted six-worker policy. Add a plan-specific policy only if paired wall improves >=3% with bounded CPU/RSS and exact semantic hashes; otherwise record and reject.
