---
type: is
id: is-01m4ghh6mf0xvqwxprhn5v6eqd
title: "PR #191 C12: store newest_activity_ns as i64 with a sentinel"
kind: bug
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4ghgje0a1raa8fwqy6m085k
created_at: 2026-10-09T14:37:02.990Z
updated_at: 2026-10-09T15:39:12.873Z
closed_at: 2026-10-09T15:39:12.872Z
close_reason: "fixed in 4d2483ba: i64 with i64::MIN sentinel, size test; reply https://github.com/jlevy/fdu/pull/191#issuecomment-6084148241"
resolution: null
duplicate_of: null
---
Suggestion. index.rs:271, :292. 8 bytes instead of 16, branch-free merge. Review: https://github.com/jlevy/fdu/pull/191#issuecomment-6082950211
