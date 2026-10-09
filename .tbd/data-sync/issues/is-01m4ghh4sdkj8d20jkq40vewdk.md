---
type: is
id: is-01m4ghh4sdkj8d20jkq40vewdk
title: "PR #191 C7: modified_at per machine row; render-json/render-jsonl jobs not run"
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4ghgje0a1raa8fwqy6m085k
created_at: 2026-10-09T14:37:01.101Z
updated_at: 2026-10-09T15:39:11.609Z
closed_at: 2026-10-09T15:39:11.608Z
close_reason: "measured in exp-212 (405ddce7): render-json +4.27%, render-yaml +5.98% wall; buffer optimization deferred to fdu-oiuc; reply https://github.com/jlevy/fdu/pull/191#issuecomment-6084148241"
resolution: null
duplicate_of: null
---
Medium. report_format.rs:913-915, 951-956, 1230-1232; query_values.rs:156-171 format_rfc3339_nanos. Run render-json and render-yaml/jsonl components. Review: https://github.com/jlevy/fdu/pull/191#issuecomment-6082950211
