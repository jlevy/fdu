---
type: is
id: is-01m4hahvn8dzb455n708vnjpgq
title: "PR #192 D1: A walk-time overlap exits 1 where an upfront overlap exits 2"
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
created_at: 2026-10-09T21:54:18.918Z
updated_at: 2026-10-09T21:54:30.877Z
started_at: 2026-10-09T21:54:30.874Z
---
Medium. crates/fdu/src/cli.rs:937, :947, :951 (prepare_roots_report*), is_usage_error :475, finish :2201. RootReachedInside propagates as plain fdu_core::Error -> exit 1; map to usage (exit 2); macOS firmlink CLI test; document. PR #192, review D: https://github.com/jlevy/fdu/pull/192#issuecomment-6089866519
