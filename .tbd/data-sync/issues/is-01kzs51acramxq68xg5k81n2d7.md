---
type: is
id: is-01kzs51acramxq68xg5k81n2d7
title: A snapshot-loaded index reports Cached, not Fresh
kind: task
status: closed
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T19:33:20.407Z
updated_at: 2026-08-11T19:41:39.442Z
closed_at: 2026-08-11T19:41:39.442Z
close_reason: "Landed in 3e4c62d. Snapshot loading stamps entries Cached with the snapshot file mtime as the as-of, and resets to Scanned afterwards so later observations are attributed correctly. Regression test asserts a loaded index reports Cached with a nonzero as-of while a scanned one reports Scanned. LIMITATION recorded for fdu-2cdv: file mtime slightly overstates freshness because the walk began earlier - format v3 should carry the true capture instant and this should read it. ALSO NOT DONE (belongs with the reconcile work): a revalidation sweep does not yet promote unchanged entries from Cached to Revalidated, so they stay Cached, which under-claims trust rather than over-claiming it."
---
The concrete gap that motivated the model: an index loaded from a snapshot currently reports Freshness::Fresh, because the snapshot was complete when it was WRITTEN. For a CLI that revalidates before printing that is harmless; for a browser painting on load it is exactly backwards, since nothing has been checked since the file was read. Loading marks entries Cached with the snapshot's capture time; revalidation promotes touched entries to Revalidated. Requires the snapshot to carry its capture timestamp (check whether format v3 from fdu-2cdv already reserves it, and share the field rather than adding a second). Regression test: load a snapshot, assert Cached and the right as-of; revalidate, assert promotion.
