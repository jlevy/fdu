---
type: is
id: is-01m4haj0hk6p5s2ty70q6v8ans
title: "PR #192 D6: first_overlap can pair a root with itself"
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
created_at: 2026-10-09T21:54:23.916Z
updated_at: 2026-10-09T21:54:40.317Z
started_at: 2026-10-09T21:54:40.315Z
---
Nit. crates/fdu-core/src/query/query_request.rs:646-650 identity-ancestor loop lacks outer != position filter. Filter; test synthetic chain. PR #192, review D: https://github.com/jlevy/fdu/pull/192#issuecomment-6089866519
