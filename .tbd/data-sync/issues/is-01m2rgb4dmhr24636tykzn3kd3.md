---
type: is
id: is-01m2rgb4dmhr24636tykzn3kd3
title: "PR #82 review F3: Python's content-status parsing has no coverage"
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - review
dependencies: []
parent_id: is-01m2rgb3631yn1waftyaj31wca
created_at: 2026-09-17T20:18:47.347Z
updated_at: 2026-09-17T21:10:34.484Z
closed_at: 2026-09-17T21:10:34.484Z
close_reason: "Addressed in PR #82 (7b2293f0..f1813c60); 24 CI checks pass including the full matrix"
resolution: null
duplicate_of: null
---
PR #82 review (https://github.com/jlevy/fdu/pull/82#issuecomment-5720664716), finding F3.

No golden or smoke test carries a non-null content. Add --cache-status after an analyzed run to a golden so parity replays it.
