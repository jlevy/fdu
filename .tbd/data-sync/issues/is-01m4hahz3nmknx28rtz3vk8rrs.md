---
type: is
id: is-01m4hahz3nmknx28rtz3vk8rrs
title: "PR #192 D4: One root's summary fold tests alias state per entry"
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
created_at: 2026-10-09T21:54:22.448Z
updated_at: 2026-10-09T21:54:34.636Z
started_at: 2026-10-09T21:54:34.634Z
---
Low (nit). crates/fdu-core/src/execution.rs:1204 SummaryFold::observe; H195 wording docs/project/guides/performance-loop.md:939. Keep one root's fold free of alias state; pin with a test. PR #192, review D: https://github.com/jlevy/fdu/pull/192#issuecomment-6089866519
