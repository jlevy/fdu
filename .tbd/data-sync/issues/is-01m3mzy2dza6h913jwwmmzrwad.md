---
type: is
id: is-01m3mzy2dza6h913jwwmmzrwad
title: "H162: match .gitignore rules without allocating"
kind: task
status: in_progress
priority: 1
version: 2
delegate: claude-code@vm
labels:
  - performance
  - linux
dependencies: []
parent_id: is-01m3mvdz2891yheyemx49gzm6j
hold: null
hold_until: null
created_at: 2026-09-28T21:50:00.638Z
updated_at: 2026-09-28T21:50:01.755Z
started_at: 2026-09-28T21:50:01.755Z
---
Pre-registered 2026-09-28 before any harness timing (screens only so far). On linux-v6.12 (358 .gitignore, 1,593 rules) the default summary spends ~0.5 s of single-thread user CPU classifying 92k entries, 7.5x the --no-controls walk, with 7.2M allocations (74 per entry): Gitignore::matches collects components into a Vec, and segment_path_matches allocates two Vec<bool> rows per segment for every anchored/path pattern. Change (892cef40): stack-buffered components, fixed-length fast path for patterns without **, stack DP rows. Cell: control H159 probe aa58a6b1, candidate 892cef40 probe; jobs aggregate-summary (controls on) and default-tree; 12 quiet pairs; linux-v6.12 deciding; accept: wall down >=3% with the interval below zero on aggregate-summary, peak RSS non-inferior (upper bound <= +5%); placebos: both arms --no-controls on aggregate-summary (includes zero) and linux-balanced-1m default-tree (no .gitignore; includes zero). Id exp-173.
