---
type: is
id: is-01m4haj0hk6p5s2ty70q6v8ans
title: "PR #192 D6: first_overlap can pair a root with itself"
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
created_at: 2026-10-09T21:54:23.916Z
updated_at: 2026-10-09T23:36:03.102Z
started_at: 2026-10-09T21:54:40.315Z
closed_at: 2026-10-09T23:36:03.092Z
close_reason: "fixed in cb401572: both ancestor loops in first_overlap skip the root's own position; case added to the_overlap_check_names_the_first_pair_in_argument_order"
resolution: null
duplicate_of: null
---
Nit. crates/fdu-core/src/query/query_request.rs:646-650 identity-ancestor loop lacks outer != position filter. Filter; test synthetic chain. PR #192, review D: https://github.com/jlevy/fdu/pull/192#issuecomment-6089866519
