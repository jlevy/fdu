---
type: is
id: is-01m0pqhvgb65fe32d9kfzmnr6d
title: "PR #38 review R10: SYNTHETIC_SUBJECTS misses every Linux generated subject"
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m0pqh0yf7etx8dywann7tnx3
created_at: 2026-08-23T07:14:55.114Z
updated_at: 2026-08-23T07:34:39.661Z
closed_at: 2026-08-23T07:34:39.660Z
close_reason: "Fixed: is_synthetic derives from tree_provenance naming gen_tree.py, falling back to the label set for pre-provenance artifacts; meta450k, vm450k and spike-15977 added. Five tests. The evidence page now marks all three as synthetic."
---
timeline.py SYNTHETIC_SUBJECTS is a hand list of four labels; spike-15977, meta450k and vm450k are all gen_tree.py trees and are absent, so the evidence report averages generated Linux subjects with real trees. PR #38 added tree_provenance, from which synthetic can be derived.
