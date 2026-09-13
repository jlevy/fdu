---
type: is
id: is-01m2ebc09kpwbynxzd5dqmj5mh
title: "PR #48 review LIFE-1: a refresh racing discovery fails discovery permanently"
kind: bug
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:39:28.690Z
updated_at: 2026-09-13T21:39:28.690Z
---
High. opened.rs:1085-1089, 905-919, 424-434; index.rs:1362-1372, 1407. A queued directory removed, refreshed, and recreated makes discovery's commit fail with InvalidDirectoryCompletion (or UnknownAncestry for a flushed batch); run_discovery propagates it, the phase becomes Failed, observation never starts, and close() returns OpenedWorkerFailed. Fix: a directory the index no longer holds, or whose ancestry changed, is stale frontier work to skip; a per-directory commit rejection is not a discovery failure. Same root cause as LIFE-3 and LIFE-4. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
