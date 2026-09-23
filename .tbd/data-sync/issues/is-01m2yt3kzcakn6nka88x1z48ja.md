---
type: is
id: is-01m2yt3kzcakn6nka88x1z48ja
title: Older reconciliation can publish stale failures after newer verification
kind: bug
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-20T07:04:53.482Z
updated_at: 2026-09-23T01:27:17.842Z
---
Overlapping reconciliations can finish out of order. Retained issues now carry the originating pass epoch, but an older pass that closes after a newer clean pass can still add its stale errors and partial freshness because completed verification ownership is not retained by epoch. Conditional fact arbitration does not necessarily reject unchanged newer verification. Track latest completed coverage by scope/path or otherwise suppress older closure evidence.

## Notes

Confirmed during fdu-8jn2 review. The omission fix tags pass-produced details with the originating epoch, but no completed-scope epoch currently lets an older closer know a newer clean verification superseded its errors. This broader fact/freshness ordering defect remains open for a dedicated change; it is distinct from scalar omitted-count erasure and skipped-subtree cleanup.
