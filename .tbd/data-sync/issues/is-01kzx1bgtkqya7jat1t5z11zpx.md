---
type: is
id: is-01kzx1bgtkqya7jat1t5z11zpx
title: "Phase 3a: Decide the common-language SLOC engine"
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1bhaasx9mn1q2nh4r7408
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:45:58.093Z
updated_at: 2026-08-13T07:45:58.601Z
---
Build feature-gated per-buffer Tokei and narrow native SCC-style prototypes behind content/code.rs. Compare semantics, dependencies, licenses, cool-off status, artifact and compile size, bounded parallelism, cancellation, RSS, large-file behavior, cache reuse, per-file latency, and pinned SCC/Tokei outputs. Record the production-engine decision before adding a dependency.
