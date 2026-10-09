---
type: is
id: is-01m4hahvn8dzb455n708vnjpgq
title: "PR #192 D1: A walk-time overlap exits 1 where an upfront overlap exits 2"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4hahez6scpwtc6yrg9qws5e
hold: null
hold_until: null
created_at: 2026-10-09T21:54:18.918Z
updated_at: 2026-10-09T23:35:56.072Z
started_at: 2026-10-09T21:54:30.874Z
closed_at: 2026-10-09T23:35:56.071Z
close_reason: "fixed in 98451612: walk_error maps RootReachedInside from prepare_roots_report* to a usage error (exit 2); macOS test an_overlap_the_walk_finds_exits_as_a_refused_request; usage exit-status section updated"
resolution: null
duplicate_of: null
---
Medium. crates/fdu/src/cli.rs:937, :947, :951 (prepare_roots_report*), is_usage_error :475, finish :2201. RootReachedInside propagates as plain fdu_core::Error -> exit 1; map to usage (exit 2); macOS firmlink CLI test; document. PR #192, review D: https://github.com/jlevy/fdu/pull/192#issuecomment-6089866519
