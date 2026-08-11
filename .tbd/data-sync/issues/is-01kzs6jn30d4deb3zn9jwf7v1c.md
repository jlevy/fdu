---
type: is
id: is-01kzs6jn30d4deb3zn9jwf7v1c
title: "PR#6 R3: intervals promote paths whose trust was withdrawn"
kind: task
status: closed
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T20:00:16.990Z
updated_at: 2026-08-11T20:02:21.994Z
closed_at: 2026-08-11T20:02:21.993Z
close_reason: "Fixed in 07e2073: an interval may promote only a path the index still considers Fresh, so InvalidateSubtree (Stale) and an in-progress sweep (Reconciling) both stop it. Regression test asserts withdrawing trust over a subtree stops the promotion."
---
Cursor Bugbot, Medium, on the R1 fix itself. provenance_of promotes an unverified entry to Revalidated whenever a covering verification interval exists, without consulting freshness. After InvalidateSubtree marks a path Stale, or while begin_reconcile marks it Reconciling, provenance can still report is_verified() true even though the index has withdrawn trust. Contradictory: status_of already reports Partial for those paths from the same freshness marks, so one struct would say partial-and-verified. Fix: only let an interval promote when the path's freshness is Fresh.
