---
type: is
id: is-01m32wjfbf8mphk5kf7ew664kd
title: Skip the subtree measurement pass for unfiltered flat reads, or record that its cost is acceptable
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-09-20-directory-query-formats.md
labels: []
dependencies: []
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
created_at: 2026-09-21T21:04:55.151Z
updated_at: 2026-09-21T21:04:55.151Z
---
PR #103 review R4 (https://github.com/jlevy/fdu/pull/103#issuecomment-5764980291). Query::needs_selection_walk() is true for List and Files in any flat format even with no filter, so 'fdu PATH --format json' and legacy --view files run query_subtrees::measure() (a BTreeMap over every directory with a PathBuf clone each) and walk() (row.clone() into rows and members per file) where one every_entry pass sufficed before; roughly 3x the allocations on large trees, and the opened-route charge doubles for a default JSON read, so an opened caller whose max_work covered a JSON query may now get Limit. Either serve unfiltered directory rows from the maintained roll-ups (partition_scalars_of) without measuring, or measure with make perf-compare and record the verdict per the performance loop before deciding it is acceptable.
