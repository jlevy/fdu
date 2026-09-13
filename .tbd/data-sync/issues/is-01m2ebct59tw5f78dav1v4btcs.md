---
type: is
id: is-01m2ebct59tw5f78dav1v4btcs
title: "PR #48 review LIFE-6: read() holds the lifecycle mutex for the whole projection"
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:39:55.176Z
updated_at: 2026-09-13T22:58:27.315Z
closed_at: 2026-09-13T22:58:27.314Z
close_reason: "Fixed in 1e8706f (Clippy binding rename in ce97922): read() releases the lifecycle guard after the phase check; the continuation table closes at shutdown so a racing read retains nothing. Tests green on all platforms."
resolution: null
duplicate_of: null
---
Medium. opened.rs:266-275. read() binds the lifecycle MutexGuard and calls read::read as its tail expression, so every read excludes every other read, begin_refresh, spawn_worker, and the start of close() for a full page of work. Fix: drop the guard after the phase check, as ensure_open does. Must land with READ-4, which this lock masks. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
