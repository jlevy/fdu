---
type: is
id: is-01m2qz75jqcj27rxjxm02z7t64
title: "PR #81 review R2: Depth codec uses a u64::MAX sentinel unlike the limit codec"
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - review
dependencies: []
parent_id: is-01m2qz74mbthxxy2eky1qgb7eh
created_at: 2026-09-17T15:19:31.669Z
updated_at: 2026-09-17T16:06:04.118Z
closed_at: 2026-09-17T16:06:04.118Z
close_reason: "Addressed in PR #81 (477d1f4b..a96d577c); CI passes"
resolution: null
duplicate_of: null
---
PR #81 review (https://github.com/jlevy/fdu/pull/81#issuecomment-5716816329), finding R2.

Encode max_depth as tag plus value; drop the refusal, the Result, and the conditional test.
