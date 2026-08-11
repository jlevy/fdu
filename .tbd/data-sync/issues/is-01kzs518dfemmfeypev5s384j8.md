---
type: is
id: is-01kzs518dfemmfeypev5s384j8
title: Compose provenance through roll-ups
kind: task
status: open
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzs51acramxq68xg5k81n2d7
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T19:33:18.325Z
updated_at: 2026-08-11T19:33:20.407Z
---
RollUp gains worst-source, oldest-observation and worst-status, composed in merge/unmerge exactly like every other roll-up field, so a directory is only as trustworthy as its least trustworthy descendant. Reuses merge_upward rather than adding machinery. Also add Index::provenance(path) constructing the view type from the stored byte plus the index-level timestamps. Property test: composition is monotone (adding a worse child never improves a parent) and matches a brute-force recomputation over the subtree.
