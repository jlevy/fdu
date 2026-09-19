---
type: is
id: is-01m2w0yms0d21by2wdwf0cqhz0
title: "H111: H86 Linux floor stage"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - campaign-2
dependencies: []
parent_id: is-01m01mqq3cqs8ae87qd2d3rydm
created_at: 2026-09-19T05:06:47.199Z
updated_at: 2026-09-19T05:06:47.199Z
---
H111: H86 remaining gap is the Linux floor and RSS claim, not a restart of the structural rewrite. Darwin has a landed composite (exp-091-102); the Linux floor stage failed (exp-103).

Metric / gates, as pre-registered: on the 450k Linux subject, index at most 1.4x floor, aggregate at most 1.25x on nominated real subjects, RSS at most 3x arena_spike, p95/median spread at most 1.5x.

Why still open: exp-103 on a 4-core virtualized Linux host passed relative gates (default-tree -31.7%, cold-scan-index -18.2%) and failed the floor/RSS gates.

Falsify: a quiet Linux floor scoreboard (`make perf-floor`) meets those gates on the 450k subject, or a current-engine profile names leftover work the Darwin composite left on the table (then a new hypothesis id, not a rewrite of H19-H22/H60/H7).

Do not: restart the H86 structural rewrite. Do not treat a quiet Darwin default-tree vs the pre-H86 control as this claim; that is validation only.

Linux host work. A Darwin agent records that this is not tonight's item.

Protocol: docs/project/guides/performance-loop.md. Pickup: docs/project/guides/performance-loop-runbook.md Current Standing. Parent epic: fdu-xde5.
