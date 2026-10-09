---
type: is
id: is-01m4hahxrb325fbrmp6k1zpn8q
title: "PR #192 D3: report_roots composes indexes without the walk-time alias check"
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
created_at: 2026-10-09T21:54:21.066Z
updated_at: 2026-10-09T21:54:32.276Z
started_at: 2026-10-09T21:54:32.273Z
---
Low. crates/fdu-core/src/query/query_report.rs:1797-1830. Run entered_directory_with against roots.aliases on each index; refuse RootReachedInside; doc Errors; unit test. PR #192, review D: https://github.com/jlevy/fdu/pull/192#issuecomment-6089866519
