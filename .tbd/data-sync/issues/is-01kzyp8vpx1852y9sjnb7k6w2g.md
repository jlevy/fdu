---
type: is
id: is-01kzyp8vpx1852y9sjnb7k6w2g
title: Split profile-scoped content cache into independently reusable analyzer records
kind: feature
status: open
priority: 0
version: 7
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - release
dependencies: []
parent_id: is-01m2phzn814exmf4ty5vw6zha0
created_at: 2026-08-13T23:10:45.468Z
updated_at: 2026-09-30T06:25:26.651Z
---
The content-sidecar header keys the entire profile and analyzer identity. Switching from full to basic invalidates the sidecar and rereads unchanged lower-level results, so reuse is not additive across profiles. Store independently reusable analyzer results or preserve compatible subsets.

## Notes

2026-09-30 stability triage (epic fdu-l4u1): not a defect in shipped behaviour but a design, feature or measurement task, so relabelled from bug. It is not a 0.3.0 release blocker.
