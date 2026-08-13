---
type: is
id: is-01kzy4vnnjsa5z7a53tx7fssve
title: "PR #8 review tests: prove parallel producers against the serial reference"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - review
  - correctness
  - testing
dependencies: []
parent_id: is-01kzy4tve6eej9e0jhxfqmqqmz
created_at: 2026-08-13T18:06:27.505Z
updated_at: 2026-08-13T18:28:25.487Z
closed_at: 2026-08-13T18:28:25.486Z
close_reason: "Fixed: integrated six portable differential tests covering cold scans, mutation reconciliation, no-op behavior, automatic workers, bounded scopes, and concurrent churn against the serial/fresh-scan oracle."
---
Integrate the senior review's portable differential suite. Compare full index images, rollups, reconciliation outcomes, bounded scopes, automatic workers, no-op behavior, and post-churn convergence against the serial reference across worker counts and scan orders. Keep fixture assumptions portable across APFS, Linux, and Windows.
