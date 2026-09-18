---
type: is
id: is-01m2sgqm7ym5byj6mstgbfenfs
title: "PR #87 review R1: smoke_crate never exercises the cargo_vcs_info skip"
kind: bug
status: closed
priority: 3
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m2sgakjkfqnw5tt0ytshwgrj
hold: null
hold_until: null
created_at: 2026-09-18T05:44:51.197Z
updated_at: 2026-09-18T05:50:21.729Z
started_at: 2026-09-18T05:45:13.607Z
closed_at: 2026-09-18T05:50:21.728Z
close_reason: "Fixed on PR #87 in 3e4d9f4d"
resolution: null
duplicate_of: null
---
scripts/release/smoke_crate.py:169
Install once without FDU_RELEASE_TAG so .cargo_vcs_info.json skip is actually run.
