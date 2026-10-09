---
type: is
id: is-01m4hahzqefpprzqg8c5wg97r6
title: "PR #192 D5: Cache-only reads compare stored directory identities with live ones"
kind: bug
status: in_progress
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4hahez6scpwtc6yrg9qws5e
hold: null
hold_until: null
created_at: 2026-10-09T21:54:23.085Z
updated_at: 2026-10-09T21:54:37.069Z
started_at: 2026-10-09T21:54:37.067Z
---
Low (nit). crates/fdu-core/src/execution.rs:710 entered on the stale/cache-only index. Confirm a hit with a stat of the live path before refusing; test. PR #192, review D: https://github.com/jlevy/fdu/pull/192#issuecomment-6089866519
