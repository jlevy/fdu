---
type: is
id: is-01m4hahzqefpprzqg8c5wg97r6
title: "PR #192 D5: Cache-only reads compare stored directory identities with live ones"
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
created_at: 2026-10-09T21:54:23.085Z
updated_at: 2026-10-09T23:36:01.875Z
started_at: 2026-10-09T21:54:37.067Z
closed_at: 2026-10-09T23:36:01.859Z
close_reason: "fixed in 5c3ccbfa: unverified (stale) index matches confirmed by one live symlink_metadata stat (reached_root, Index::entered_directory_where); tests a_snapshots_match_is_confirmed_against_the_tree_as_it_is_now and macOS cache-only firmlink case"
resolution: null
duplicate_of: null
---
Low (nit). crates/fdu-core/src/execution.rs:710 entered on the stale/cache-only index. Confirm a hit with a stat of the live path before refusing; test. PR #192, review D: https://github.com/jlevy/fdu/pull/192#issuecomment-6089866519
