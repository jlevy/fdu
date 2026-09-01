---
type: is
id: is-01m1dtr3hap1kqbkfcap66paq8
title: Remove duplicate path ownership from exact impact and journal publication
kind: task
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - performance
  - design
dependencies:
  - type: blocks
    target: is-01m1dtr903vj783j9ajaxfnczf
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-01T06:33:17.609Z
updated_at: 2026-09-01T10:43:54.375Z
---
Move scanner-owned paths once, accumulate impact with flags and bounded IDs rather than a full PathBuf set, stop at all_dirty, and compute journal retention cost before cloning. Introduce shared Commit ownership only if a post-cleanup profile still names retained cloning.

## Notes

Starting from def29fd after exp-079 and full local gates. First step is a fresh post-proof profile and counter snapshot for opened discovery and exact large/batched commits; only then will impact or journal ownership change. Keep shared Commit ownership out unless retained cloning remains material.
