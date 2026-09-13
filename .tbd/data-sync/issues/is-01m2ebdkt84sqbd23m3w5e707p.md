---
type: is
id: is-01m2ebdkt84sqbd23m3w5e707p
title: "PR #48 review LIFE-9: Fresh published mid-handoff; one failed subtree downgrades a multi-path refresh"
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:21.447Z
updated_at: 2026-09-13T21:40:21.447Z
---
Low. index.rs:1802-1848; scan.rs:3323-3329. The handoff publishes freshness Fresh while the phase is still Reconciling, and reconcile_paths_target applies one completion flag to every subtree of a multi-path refresh, so one failed subtree downgrades verified siblings to Partial. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
