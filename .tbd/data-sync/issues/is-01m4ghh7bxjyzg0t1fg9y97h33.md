---
type: is
id: is-01m4ghh7bxjyzg0t1fg9y97h33
title: "PR #191 C14: price B1 fix on the event path"
kind: bug
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4ghgje0a1raa8fwqy6m085k
created_at: 2026-10-09T14:37:03.740Z
updated_at: 2026-10-09T15:39:13.525Z
closed_at: 2026-10-09T15:39:13.524Z
close_reason: "fixed in 91a86abd: parents deduplicated per intent, outside pending capacity; reply https://github.com/jlevy/fdu/pull/191#issuecomment-6084148241"
resolution: null
duplicate_of: null
---
Suggestion. Deduplicate parents across the batch; keep parent reads out of pending capacity. Review: https://github.com/jlevy/fdu/pull/191#issuecomment-6082950211
