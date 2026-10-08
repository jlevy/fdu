---
type: is
id: is-01m4d4r46bedn0er8e3v9ycqe0
title: "0.4.0 release standing: paired non-regression of the report path against v0.3.0"
kind: task
status: open
priority: 2
version: 2
labels:
  - release
  - performance
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
created_at: 2026-10-08T06:55:55.082Z
updated_at: 2026-10-08T16:19:42.698Z
---
0.4.0 changes report construction (query_report.rs +588, query_request.rs +933 lines since v0.3.0) and makes content views imply analysis (--view documents now runs words), and none of #174/#177/#179 timed it. Walker/scan code is unchanged. 0.3.0 had a release standing cell (exp-202). Before tagging 0.4.0, run a paired non-regression screen of the release candidate against v0.3.0 on default-tree, aggregate-summary, and the code/documents views (cold and warm content cache) on a nominated real tree, and record it. Relates to fdu-mvnp (content-query re-baseline).

## Notes

Diagnostic only (2026-10-08, not a harness cell, not recorded): native v0.3.0 (built from the tag) vs claude/release-0.4.0-prep 237696eb, both release profile, on the Linux kernel 7.3-rc6 shallow tree (95,938 files) on the internal SSD, 6 interleaved pairs in alternating order, uncontrolled host (load 7-14). Paired median change of 0.4.0: cold code+documents analysis +1.1% (range -23.9..+22.0), cached -1.4% (-13.9..+11.7), default tree -10.2% (-37.3..+24.2). No sign of a large regression; a recorded screen still needs make perf-compare on default-tree, aggregate-summary and the content jobs. Script: scratchpad pair_diag.py (session 7d380cd7).
