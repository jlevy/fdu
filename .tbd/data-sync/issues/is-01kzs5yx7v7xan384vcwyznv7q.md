---
type: is
id: is-01kzs5yx7v7xan384vcwyznv7q
title: "PR#6 R2: directory provenance does not compose its subtree"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T19:49:29.974Z
updated_at: 2026-08-11T19:49:29.974Z
---
Cursor Bugbot, Medium. Index::provenance returns a directory's OWN source rather than the composed weakest source of its subtree, so a parent can read Cached while revalidated children read Revalidated. Deferred rather than fixed: this is precisely fdu-fka6 (compose provenance through roll-ups), which cannot be done at query time without an O(subtree) walk per call and belongs in RollUp with the recompute-upward treatment non-invertible reducers need. The doc comment already states the current limitation; this bead adds a test pinning the narrow contract so no consumer mistakes it, and closes when fdu-fka6 lands.
