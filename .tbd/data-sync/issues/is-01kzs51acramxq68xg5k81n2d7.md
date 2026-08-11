---
type: is
id: is-01kzs51acramxq68xg5k81n2d7
title: A snapshot-loaded index reports Cached, not Fresh
kind: task
status: open
priority: 0
version: 1
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T19:33:20.407Z
updated_at: 2026-08-11T19:33:20.407Z
---
The concrete gap that motivated the model: an index loaded from a snapshot currently reports Freshness::Fresh, because the snapshot was complete when it was WRITTEN. For a CLI that revalidates before printing that is harmless; for a browser painting on load it is exactly backwards, since nothing has been checked since the file was read. Loading marks entries Cached with the snapshot's capture time; revalidation promotes touched entries to Revalidated. Requires the snapshot to carry its capture timestamp (check whether format v3 from fdu-2cdv already reserves it, and share the field rather than adding a second). Regression test: load a snapshot, assert Cached and the right as-of; revalidate, assert promotion.
