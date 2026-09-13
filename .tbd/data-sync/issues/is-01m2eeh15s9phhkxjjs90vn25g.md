---
type: is
id: is-01m2eeh15s9phhkxjjs90vn25g
title: "PR #48 review READ-7: tree level advance recurses once per tree level"
kind: bug
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T22:34:39.160Z
updated_at: 2026-09-13T23:38:41.392Z
closed_at: 2026-09-13T23:38:41.391Z
close_reason: "Fixed in ef1db6f (test made faster in f917cb7): directory_at_depth climbs iteratively; a 1,000-level chain on a 64 KiB stack, which the recursive ascent overflows. CI green. fdu-pokc stays open."
resolution: null
duplicate_of: null
---
Low. crates/fdu-core/src/opened/read.rs directory_at_depth recursed once per level when ending a level, so stack depth grew with tree depth: bounded by PATH_MAX on POSIX, about 16,000 levels on Windows extended paths. Fix: make the ascent iterative. Sibling of fdu-pokc (the level-advance work bound), which stays open. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
