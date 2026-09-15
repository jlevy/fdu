---
type: is
id: is-01m2h78ge44eynkqyankwmmtvz
title: "PR #57 review PR57-AGB6-2: children and partition accessors state ignore facts an index did not observe"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m2h77qemaay1jkhbhfzh35me
created_at: 2026-09-15T00:25:23.139Z
updated_at: 2026-09-15T00:28:46.631Z
closed_at: 2026-09-15T00:28:46.627Z
close_reason: "53dc59a: partition_total/partition_rollup/partition_rollup_summary return Result and refuse; ChildSnapshot.ignored is Option and partitions None when unobserved; 30c3895 keeps the opened roll-up answering without the gitignore feature; 5c79c31 states the opened-root invariant. Holds at c141285; an_index_that_did_not_observe_controls_states_no_partition_or_child_ignore_fact passes."
resolution: null
duplicate_of: null
---
PR #57 review https://github.com/jlevy/fdu/pull/57#pullrequestreview-5200240760, P2. At 7b804df, crates/fdu-core/src/index.rs:1238-1252, :1806, :2734: IndexHandle::children (ChildSnapshot.ignored, .partitions), Index::partition_total and partition_rollup_summary answered 'not ignored' / unignored == all for an index that did not observe control state.
