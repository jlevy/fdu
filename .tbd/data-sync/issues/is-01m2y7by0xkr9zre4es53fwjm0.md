---
type: is
id: is-01m2y7by0xkr9zre4es53fwjm0
title: Core directory projection with subtree recency and aggregate selection
kind: feature
status: in_progress
priority: 1
version: 3
delegate: claude-code@spud10
labels: []
dependencies:
  - type: blocks
    target: is-01m2y7c1v3etw9wkzz87vgm6ke
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
hold: null
hold_until: null
created_at: 2026-09-20T01:37:22.965Z
updated_at: 2026-09-20T01:38:32.685Z
started_at: 2026-09-20T01:38:32.676Z
---
Implement the epic contract in fdu-core: Directories view, public DirectoryRow/Section, pure iterative recency calculation, whole-root matching, aggregate bounds, size/count/mtime/name sorting, unbounded default and explicit limits. Test empty/nested roots, recent children, pre-epoch/future dates, root ignored/exclude semantics and portable identity. Preserve existing views and cache identity.
