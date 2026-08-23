---
type: is
id: is-01m0py29ngg76h33a3y0v9ef5r
title: "A macOS floor instrument: parallel getattrlistbulk walk with the tallies oracle"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies:
  - type: blocks
    target: is-01m0p4ya63vm7c49hw49pt4xaw
parent_id: is-01m0py28fhtj20wwfc05s9e148
created_at: 2026-08-23T09:08:45.359Z
updated_at: 2026-08-23T09:09:11.560Z
---
fdu-33ri's named obstacle: parfloor.c is Linux-only (SYS_getdents64, statx), so no x-floor can be computed on this Mac and none of the campaign's thresholds (1.25x aggregate, 1.4x index) or its termination rule can be evaluated here. The floor report says getattrlistbulk 'changes the interface floor itself', so the macOS floor is a parallel getattrlistbulk walk that retains nothing and produces the five tallies -- checked by the tallies oracle PR #45 landed, because a floor that skips work is a fake denominator. Deliverable: explorations/benchmarks/spikes/parfloor_darwin (Rust or C), a worker-count sweep on the three deciding subjects, the x-floor row for aggregate-summary and cold-scan-index, and the regime difference recorded in the floor report. Decides whether fdu-33ri ships as one scoreboard or two.
