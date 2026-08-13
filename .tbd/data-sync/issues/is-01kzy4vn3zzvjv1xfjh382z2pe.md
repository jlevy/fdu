---
type: is
id: is-01kzy4vn3zzvjv1xfjh382z2pe
title: "PR #8 review M2: make permission-denial fixtures privilege-aware"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - review
  - correctness
dependencies: []
parent_id: is-01kzy4tve6eej9e0jhxfqmqqmz
created_at: 2026-08-13T18:06:26.942Z
updated_at: 2026-08-13T18:28:25.017Z
closed_at: 2026-08-13T18:28:25.016Z
close_reason: "Fixed: scan, snapshot-save, and CLI permission fixtures now probe whether permission bits are enforced and skip only when the environment cannot induce EACCES."
---
Senior review M2: chmod-based EACCES fixtures fail under root or CAP_DAC_OVERRIDE because the process can still read the target. Add a capability probe and skip only when permission bits cannot induce the boundary; keep ordinary unprivileged coverage unchanged. Apply to scan, snapshot-save, and CLI exit fixtures.
