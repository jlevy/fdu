---
type: is
id: is-01m4ghh4ded8avbj86a1gp8ftd
title: "PR #191 C6: hot-path cost below wall resolution; measure with instruction counts"
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4ghgje0a1raa8fwqy6m085k
created_at: 2026-10-09T14:37:00.717Z
updated_at: 2026-10-09T15:39:11.301Z
closed_at: 2026-10-09T15:39:11.300Z
close_reason: "deferred: callgrind counts and 1M RSS are in H193, tracked by fdu-088k; exp-212 re-measured cold-scan-index and warm-snapshot-load; reply https://github.com/jlevy/fdu/pull/191#issuecomment-6084148241"
resolution: null
duplicate_of: null
---
Medium. index.rs merge paths; measure cold-scan-index, warm-snapshot-load, opened-discovery by instruction counts. Review: https://github.com/jlevy/fdu/pull/191#issuecomment-6082950211
