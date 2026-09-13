---
type: is
id: is-01m2ebc11er5wb1mb6xafrgd04
title: "PR #48 review LIFE-2: an unclosable invalidation is re-walked on every later event"
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:39:29.453Z
updated_at: 2026-09-13T21:39:29.453Z
---
Medium. scan.rs:4172-4177, 806-808; opened.rs:1300-1318. A provider gap over an unreadable directory commits InvalidateSubtree; the walk's scan error counts as incomplete, so the root is restored to pending invalidations and re-walked on every unrelated event, forever; the apply report is discarded so no issue explains the Partial freshness. Fix: restore only for retryable outcomes (stale, resource_refused); treat scan errors as a settled Partial boundary retained as issues, as the handoff does. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
