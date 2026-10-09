---
type: is
id: is-01m4ghh2713ewpvzsqjjkefwpr
title: "PR #191 B2: hold an incremental route to a cold walk with a property test"
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4ghgje0a1raa8fwqy6m085k
created_at: 2026-10-09T14:36:58.464Z
updated_at: 2026-10-09T15:39:09.417Z
closed_at: 2026-10-09T15:39:09.416Z
close_reason: "fixed in 6e4272d7: seeded property test holds a watched tree to a cold walk after every step; reply https://github.com/jlevy/fdu/pull/191#issuecomment-6084147920"
resolution: null
duplicate_of: null
---
Suggestion. One property test comparing a watch session (or the index apply path) with a fresh scan_into_index after random create/remove/rename/move-in steps with stamped times. Review: https://github.com/jlevy/fdu/pull/191#issuecomment-6082917102
