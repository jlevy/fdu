---
type: is
id: is-01kzscyrm1fhnd1psjbsxmmv3m
title: "Two-tier scheduler experiment: shallow-FIFO orientation tier over region-affine depth-first workers"
kind: task
status: open
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-08-11T21:51:45.280Z
updated_at: 2026-08-11T22:40:23.188Z
---
Design from first principles with jlevy (PR#6 thread); supersedes the open question in fdu-bfxh and depends on the attribution bead. Depth <= D (D~2): claims drain FIFO, strictly preferred. Below D: work grouped by depth-D ancestor (region); LIFO within region for locality and bounded memory; workers keep region affinity while their region has work; free workers claim from the least-served region. No barrier anywhere - if only deep work exists, take it (work-conserving). Predicted: global-FIFO frontier RSS cost disappears (pending = top levels + per-worker spines), deep spurs occupy ~1 worker while others spread horizontally, orientation becomes stronger than today's preference because workers are spread across distinct top subtrees by construction. Gate through the accept rule (wall/RSS/CPU) AND the product metric (subtrees-started-at-halfway under default workers, currently 7-8 for both orders). Prior art: rayon/dust and ignore use local-LIFO steal-FIFO work stealing; jwalk pays throughput for strict ordered streaming and is unmaintained - we want the preference, not the strict order.

## Notes

Landed in 37c0da0 as exp-013. Region buckets keyed by depth-1 ancestor, round-robin ready ring, LIFO within region, unbounded worker affinity, no barrier. RSS -3.89% [-4.90,-3.15] on cold-scan-index; wall unchanged; deep-spur share 0-4% vs DFS 4-23%. REMAINING: region granularity is fixed at depth 1, so a tree whose content sits under one top-level directory collapses to a single region (correct, no orientation benefit) - adaptive granularity unmeasured. Affinity is unbounded, so regions in flight are bounded by worker count. Ring untested at home-folder region counts.
