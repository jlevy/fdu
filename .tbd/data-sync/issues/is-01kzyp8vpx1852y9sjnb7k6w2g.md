---
type: is
id: is-01kzyp8vpx1852y9sjnb7k6w2g
title: Split profile-scoped content cache into independently reusable analyzer records
kind: bug
status: open
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-13T23:10:45.468Z
updated_at: 2026-08-14T00:04:19.312Z
---
The content-sidecar header keys the entire profile and analyzer identity. Switching from full to basic invalidates the sidecar and rereads unchanged lower-level results, so reuse is not additive across profiles. Store independently reusable analyzer results or preserve compatible subsets.
