---
type: is
id: is-01m2pmram44dgp78vm6xq4w7k7
title: "Stored-state model: identity headers, shared validity fingerprint, keyed storage"
kind: task
status: open
priority: 0
version: 5
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - design
  - cache
dependencies:
  - type: blocks
    target: is-01m2pmrbxvnerxyhnjhrswy1ye
  - type: blocks
    target: is-01m2phzn814exmf4ty5vw6zha0
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
child_order_hints:
  - is-01m2esgptzqkfqja3qfatsve2q
created_at: 2026-09-17T02:57:25.124Z
updated_at: 2026-09-17T03:38:50.849Z
---
One contract for entries, .gitignore control state, classification, and content: identity (engine fingerprint plus
exactly the request parts that shape the tier), one per-item fingerprint (size, allocated, mtime, ctime, inode,
device; source bytes where content-derived), serves and project (equality by default), storage keyed by identity
with eviction as a miss, per-tier provenance. New snapshot and content-store formats; old files read as stale.

## Notes

2026-09-17 (PR #78 review): Phase 1 item 2. Scope reduced: store headers carry every tier's identity (sidecar gains scope, engine fingerprint, analyzer versions and options), content served by equality of analyzer sets, per-tier write rule (entries only after a complete scan; control state and content records with verified items). Stores keyed by identity are a deferral (fdu-w3l5).
