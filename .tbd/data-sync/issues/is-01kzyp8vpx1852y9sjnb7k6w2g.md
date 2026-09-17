---
type: is
id: is-01kzyp8vpx1852y9sjnb7k6w2g
title: Split profile-scoped content cache into independently reusable analyzer records
kind: bug
status: open
priority: 0
version: 5
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - release
dependencies: []
parent_id: is-01m2phzn814exmf4ty5vw6zha0
created_at: 2026-08-13T23:10:45.468Z
updated_at: 2026-09-17T02:57:30.151Z
---
The content-sidecar header keys the entire profile and analyzer identity. Switching from full to basic invalidates the sidecar and rereads unchanged lower-level results, so reuse is not additive across profiles. Store independently reusable analyzer results or preserve compatible subsets.
