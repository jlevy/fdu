---
type: is
id: is-01kzsbja5qc5496wzbde67w6ee
title: Breadth-first advantage is unproven under the default worker count
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:27:28.694Z
updated_at: 2026-08-11T21:27:28.694Z
---
Measured while addressing PR#6 C3/D4: on the branching_tree fixture, at the halfway point of a scan both orders start 7-8 of 12 top-level subtrees under the default worker count (run to run), while with threads=1 breadth-first starts 7 against depth-first's 6 deterministically. Emission order under parallelism is dominated by worker scheduling rather than queue order, so the product justification for the BFS default -- a meaningful mid-scan top-level ranking -- is demonstrated only serially. Options: level-aware scheduling (depth barrier or a minimum-outstanding-depth rule), or accept and document that progressive consumers should read roll-ups rather than rely on emission order. Needs a real-tree measurement (time to stable top-level ranking) before choosing.
