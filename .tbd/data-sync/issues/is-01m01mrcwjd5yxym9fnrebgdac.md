---
type: is
id: is-01m01mrcwjd5yxym9fnrebgdac
title: "H90: bootstrap journals every batch then clears it; re-screen exp-003 residue"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/research/research-2026-08-15-consumer-structural-headroom.md
labels:
  - perf
dependencies: []
created_at: 2026-08-15T02:42:00.721Z
updated_at: 2026-08-15T02:42:00.721Z
---
apply_baseline: apply() clones effective ops (PathBufs included) into the journal, establish_baseline() clears it - one cloned-and-freed path per entry per cold scan. exp-003 rejected the invasive twin-loop form on macOS/60k before the consumer was the bottleneck; residue is now a one-flag change. Pre-registered signal: cold-scan-index minor_faults and user_cpu_ns down; wall change may be under the bar alone - expected to ride with H86.
