---
type: is
id: is-01m2gbm25znsy1z5x949dzcx3z
title: WorkerFailures::first demotes a poison trace even when no panic was recorded
kind: bug
status: closed
priority: 3
version: 5
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2h3ches22p7k9x4kka6bq6q
created_at: 2026-09-14T16:22:21.623Z
updated_at: 2026-09-15T01:08:04.283Z
closed_at: 2026-09-15T01:08:04.280Z
close_reason: "b1f6f8a: WorkerFailures::first demotes a poison trace only when a WorkerPanicked is recorded; otherwise close reports the earliest failure. Test close_reports_a_poisoning_no_worker_panic_explains_when_it_came_first."
resolution: null
duplicate_of: null
---
PR #56 review PR56-LIFE-3 (https://github.com/jlevy/fdu/pull/56#pullrequestreview-5200187448). Location at cfd1335: opened.rs:1870-1878. first() demotes a poison-trace failure whenever any non-poison failure exists, without checking that a panic was recorded. A poisoning caused on the client side, followed by an unrelated worker error, is reported as the later error, which contradicts the function's own comment. Fix: demote the poison trace only when a panic is recorded, and add a test.
