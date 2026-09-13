---
type: is
id: is-01m2eb2c9yzj0w7yjp5dxk2v10
title: "PR #54 review H86-4: generated linux-450k subject projected as synthetic: false"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eb1cnhke030h15e79fyzvc
created_at: 2026-09-13T21:34:13.308Z
updated_at: 2026-09-13T22:07:31.394Z
closed_at: 2026-09-13T22:07:31.393Z
close_reason: "Fixed in 4dccad8: chose adding linux-450k to SYNTHETIC_SUBJECTS (smallest option; the recorded provenance cannot be rewritten to name a command nobody recorded), updated the TREE_GENERATOR and is_synthetic comments, test written first and failing on the old code; regenerated timeline.json marks the subject synthetic: true."
resolution: null
duplicate_of: null
---
Medium. timeline.is_synthetic (explorations/benchmarks/realtree/timeline.py) recognizes a generated subject only by gen_tree.py in tree_provenance or a label in SYNTHETIC_SUBJECTS. exp-102/103's provenance reads 'Generated balanced recipe, 450,001 entries ...' (the corpus generator, not gen_tree.py) and linux-450k is not in the set, so timeline.json records synthetic: false and the evidence page averages a generated tree with the real Linux subjects. Related closed bead fdu-t4kn (PR #38 R10) fixed the same gap for meta450k/vm450k/spike-15977. Fix (pick one): name the generator in provenance, add linux-450k to SYNTHETIC_SUBJECTS, or teach is_synthetic the corpus generator's wording; regenerate.
