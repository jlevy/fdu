---
type: is
id: is-01m4hahz3nmknx28rtz3vk8rrs
title: "PR #192 D4: One root's summary fold tests alias state per entry"
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
created_at: 2026-10-09T21:54:22.448Z
updated_at: 2026-10-09T23:35:59.945Z
started_at: 2026-10-09T21:54:34.634Z
closed_at: 2026-10-09T23:35:59.936Z
close_reason: "fixed in 8eb0e780: alias watch moved out of SummaryFold into FoldAliases composed only over several roots; SummaryFold::new/observe byte-identical to 23d11b19; signature pin in a_walk_finds_another_root_it_enters_through_an_alias; H195 wording corrected in 9518c6e0"
resolution: null
duplicate_of: null
---
Low (nit). crates/fdu-core/src/execution.rs:1204 SummaryFold::observe; H195 wording docs/project/guides/performance-loop.md:939. Keep one root's fold free of alias state; pin with a test. PR #192, review D: https://github.com/jlevy/fdu/pull/192#issuecomment-6089866519
