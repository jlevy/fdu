---
type: is
id: is-01kzs5141vz8jtgb4wh2j432vb
title: "Spec: progressive results — order, provenance, sessions, lazy open"
kind: epic
status: open
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
child_order_hints:
  - is-01kzs5162sbb3w464j2p1mgs6k
  - is-01kzs518dfemmfeypev5s384j8
  - is-01kzs51acramxq68xg5k81n2d7
created_at: 2026-08-11T19:33:13.914Z
updated_at: 2026-08-11T19:33:20.407Z
---
Make fdu usable by a consumer that needs answers WHILE the walk runs and INSTANTLY on the second open. Independent of FSEvents: everything here lands on every platform, helps the first scan as much as the second, and is what makes the journal worth having rather than a consequence of it. Motivating measurements on this host: a home folder of 4,366,510 files and 1,016,449 dirs (224 GiB) walks cold in 791 s, and a warm snapshot of that size would take ~11 s just to load at ~2 us/record - neither is compatible with an interactive first paint. Two data-structure principles govern the work: delta-friendly (existing) and partial-friendly (new peer) - a partially walked tree is a valid, useful answer as long as the boundary of incompleteness is knowable, and a delta applied to a partial structure yields another valid partial structure.
