---
type: is
id: is-01m4haj2z3q1tf5w9cr7sfyept
title: "PR #192 D9: Test coverage of the new routes (firmlink full-index and cache-only; cache dir via refresh and watch)"
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
created_at: 2026-10-09T21:54:26.402Z
updated_at: 2026-10-09T23:36:07.580Z
started_at: 2026-10-09T21:54:44.361Z
closed_at: 2026-10-09T23:36:07.577Z
close_reason: "fixed in 5c3ccbfa and f06c5230: firmlink full-index and cache-only cases (macOS, /System/Volumes/Data/usr beside /usr/local); refresh case in the cache-directory test; watch case a_watch_names_its_snapshot_in_a_cache_directory"
resolution: null
duplicate_of: null
---
Nit. execution.rs:5009 firmlink test; :4825 a_cache_directory_names_one_roots_snapshot_on_every_route. Add full-index/cache-only firmlink cases and refresh/watch cache-dir cases. PR #192, review D: https://github.com/jlevy/fdu/pull/192#issuecomment-6089866519
