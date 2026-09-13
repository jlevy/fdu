---
type: is
id: is-01m2ebbzxr2pf419p03tfzqv5b
title: "PR #48 review READ-1: symlink or special entry attrs update panics or corrupts a file tally"
kind: bug
status: closed
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:39:28.311Z
updated_at: 2026-09-13T21:40:43.002Z
closed_at: 2026-09-13T21:40:43.001Z
close_reason: "Fixed in 4d4e37d: remove_serving_file_semantics is guarded to EntryKind::File at the call and inside the function; the conservation proof covers Symlink and Other same-kind updates, plus a dedicated regression test."
resolution: null
duplicate_of: null
---
Blocker. index.rs:3249-3252 (upsert_beneath same-kind branch) calls remove_serving_file_semantics (index.rs:2083-2096) for any non-directory kind, but only EntryKind::File is interned (index.rs:1990). A same-kind attrs update of a Symlink or Other entry panics under the index write guard (poisoning the opened root) when no file shares its classification, or subtracts from a real file's tally and releases its interned id when one does. Fix: guard both the call and the function to EntryKind::File, mirroring move_serving_file_partition; add Symlink and Other same-kind updates to the serving-index conservation proof. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
