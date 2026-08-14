---
type: is
id: is-01kzz29x4thpjssb3keq3yk2at
title: Snapshot load resolves every record by linear sibling scan
kind: bug
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01kzz29dspd7bsy6jk98mpb9z3
created_at: 2026-08-14T02:41:02.618Z
updated_at: 2026-08-14T02:49:27.567Z
closed_at: 2026-08-14T02:49:27.567Z
close_reason: "Added Index::child_id, a single map descent, and pointed snapshot::parse_stream at it instead of scanning the parent's sibling list. Measured on this container (release, warm): 4,096 siblings 61.6ms -> 6.4ms, 16,384 siblings 2,183ms -> 24.7ms, 65,536 siblings 38,894ms -> 107ms. The quadratic term dominated everything else at width; a 65k-wide directory took 39 seconds to load from a snapshot. Added round_trip_handles_wide_directory_fanout at 4,096 children."
---
crates/fdu/src/snapshot.rs parse_stream resolves each freshly inserted record's EntryId with index.children_of(parent).find_map(...), a linear scan of the parent's children. Loading a directory with N children costs O(N^2) name comparisons; at 4,096 siblings that is roughly 8M OsString compares for one directory.

Index already stores children in a BTreeMap<OsString, EntryId>, so the id is one map lookup away. PR #4 added Index::child_id(parent, name) and a round_trip_handles_wide_directory_fanout regression test at 4,096 children. Neither is on main, and snapshot load is a path the ledger has repeatedly optimized (exp-005, exp-009).
