---
type: is
id: is-01m4ghh555e0xa4wcgwxt2qh59
title: "PR #191 C8: each rendered tree row formats its age twice"
kind: bug
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4ghgje0a1raa8fwqy6m085k
created_at: 2026-10-09T14:37:01.476Z
updated_at: 2026-10-09T15:39:11.916Z
closed_at: 2026-10-09T15:39:11.915Z
close_reason: "fixed in a567db65: each age cell formatted once; reply https://github.com/jlevy/fdu/pull/191#issuecomment-6084148241"
resolution: null
duplicate_of: null
---
Low. report_format.rs:1940-1948 width pre-pass and :1974 call tree_age_cell twice per row. Review: https://github.com/jlevy/fdu/pull/191#issuecomment-6082950211
