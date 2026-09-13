---
type: is
id: is-01m2ebcth0cebeeyb7e2b9y2bk
title: "PR #48 review READ-4: the continuation table can exceed its 128-record bound"
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:39:55.551Z
updated_at: 2026-09-13T22:58:27.640Z
closed_at: 2026-09-13T22:58:27.639Z
close_reason: "Fixed in 1e8706f with LIFE-6: insert evicts while at or above the bound and restore drops the record it would otherwise evict; table-level take-insert-restore reproduction. Tests green on all platforms."
resolution: null
duplicate_of: null
---
Medium. opened/continuation.rs:111-114, 136-142. take then insert then restore reaches 129 records; eviction fires only at exactly MAX_CONTINUATIONS, so it never fires again and the table grows one record per nonterminal page. Masked only by LIFE-6's lock. Fix: evict with while len >= MAX in insert, and have restore evict when full. Land with LIFE-6. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
