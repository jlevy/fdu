---
type: is
id: is-01m2ebdm962nswte313912pwxz
title: "PR #48 review LIFE-10: panic wakeups, first-failure masking, and a raw DirectoryComplete path"
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:21.925Z
updated_at: 2026-09-13T21:40:21.925Z
---
Low. opened/journal.rs:84-97; opened.rs:1510-1525; index.rs:1523-1529. After a worker panic or poisoned lock, blocked changes() waiters wake only on timeout or close; join_workers reports the first failure in spawn order, so discovery's poisoning error can mask the observation worker's panic; DirectoryComplete carries the producer's raw path, canonical only because discovery builds it. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
