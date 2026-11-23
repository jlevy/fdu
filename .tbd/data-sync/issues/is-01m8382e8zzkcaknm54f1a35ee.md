---
type: is
id: is-01m8382e8zzkcaknm54f1a35ee
title: Time v0.4.0, v0.5.0 and v0.6.0 in the performance index's history cells
kind: task
status: open
priority: 2
version: 1
labels:
  - performance
dependencies: []
created_at: 2026-11-23T03:44:21.785Z
updated_at: 2026-11-23T03:44:21.785Z
---
The macOS history cells (performance-evidence/history/, PR #176) end at v0.3.0, timed 2026-10-05/06, so the page's score and its release history stop three releases short of what has shipped. Add the three release builds to every component cell with the in-repo driver (benchmarks.realtree.history), interleaved against the reference build, on the internal SSD with nothing else running (the runbook's unattended rules). Measured cost at Phase 1 rounds: about 5.3 h for 13 builds across 13 cells, so about 6.5 h with 16 builds, longer at 20 rounds. Run it together with fdu-bkj2 (quiet, 20 rounds, a non-exploratory stage) so the cells are re-timed once. Decide first whether the reference build stays v0.3.0 (the spec's fixed reference) or moves to the latest release; moving it changes every published ratio. Known content-answer differences between builds (0.4.0's documents shares, 0.6.0's change-time default) need index-known-answers.json entries before a cell will accept them. Then make perf-report and update the evidence report's macOS sections.
