---
type: is
id: is-01kzs5yv16x68sg9x8mzeep15c
title: "PR#6 R1: upsert leaves a stale entry source after verification"
kind: task
status: closed
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T19:49:27.714Z
updated_at: 2026-08-11T19:52:46.605Z
closed_at: 2026-08-11T19:52:46.604Z
close_reason: "Fixed in 94957f2, though not the way the finding suggested, and the reason is worth keeping. Stamping inside apply_upsert cannot work for the entries that matter: the exclusive reconcile path ELIDES unchanged entries in the producer before they become a delta (the measured 18% warm-path win from abeb377), so verified-but-unchanged entries structurally never reach the consumer. Recording verification per entry would have meant giving that optimization back. Instead finish_reconcile stores one interval per completed swept subtree and provenance consults those intervals when an entry's own stamp is unverified - same store-where-it-varies choice as the timestamps, leaves both the elision and the Delta contract untouched. Entries that DO flow through apply are stamped as well, including the unchanged case, and snapshot load sets applying_source to Revalidated. Regression test reproduces the reviewer's scenario end to end and failed before the fix."
---
Cursor Bugbot, High. apply_upsert stamps source only when ALLOCATING a new entry, so an existing entry that a producer freshly stats keeps whatever source it had. After a snapshot load, warm reconcile/open verifies entries yet provenance still reports Cached and is_verified() stays false. I had recorded this as a known limitation on the grounds that under-claiming trust is the safe direction, and that is true, but the reviewer is right that it makes the feature useless for its purpose: a browser would never clear an indicator after verification. Fix: stamp the source whenever an observation touches an existing entry, including the unchanged case (an unchanged-but-stat'd entry HAS been verified), and set applying_source to Revalidated after a snapshot load so post-load observations are labelled honestly.
