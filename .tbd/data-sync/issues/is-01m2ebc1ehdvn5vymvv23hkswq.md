---
type: is
id: is-01m2ebc1ehdvn5vymvv23hkswq
title: "PR #48 review LIFE-3: a directory deleted during discovery marks the root Partial(Inaccessible) for the session"
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:39:29.872Z
updated_at: 2026-09-13T21:39:29.872Z
---
Medium. opened.rs:951-965; index.rs:1575-1585, 1553-1563, 1610-1625. discover_directory publishes Inaccessible for any read_dir error including NotFound; Finish only upgrades Partial(Building) and Watching never re-derives coverage, so a zero-error handoff pass still leaves Partial(Inaccessible). Fix: NotFound/NotADirectory during discovery is stale frontier work (no transition); Watching after a complete zero-error pass re-derives Complete. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
