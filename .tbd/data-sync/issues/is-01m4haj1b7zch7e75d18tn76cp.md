---
type: is
id: is-01m4haj1b7zch7e75d18tn76cp
title: "PR #192 D7: plan ignores Delivery::cache_dir"
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
created_at: 2026-10-09T21:54:24.742Z
updated_at: 2026-10-09T23:36:04.533Z
started_at: 2026-10-09T21:54:41.949Z
closed_at: 2026-10-09T23:36:04.525Z
close_reason: "fixed in 04e8ee58: plan counts cache_dir as a cache location; test a_cache_directory_plans_as_the_file_it_names"
resolution: null
duplicate_of: null
---
Nit. crates/fdu-core/src/execution.rs:509-510 policy_requires_index reads only cache_path. Count cache_dir as a location; test. PR #192, review D: https://github.com/jlevy/fdu/pull/192#issuecomment-6089866519
