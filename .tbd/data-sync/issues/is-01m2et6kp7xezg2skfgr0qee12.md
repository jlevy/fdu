---
type: is
id: is-01m2et6kp7xezg2skfgr0qee12
title: "PR #48 review FIX48-3: identical issues accumulate on every re-walk of an unreadable subtree"
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2et5s9yyv9e6tz783rp2m6c
created_at: 2026-09-14T01:58:40.582Z
updated_at: 2026-09-14T02:49:13.807Z
closed_at: 2026-09-14T02:49:13.804Z
close_reason: "Fixed in 509b536: retain_issue dedupes by (kind, path); a complete reconcile drops disproven non-gap issues seen before it began; scan-error issue paths root-relative; no session golden records them. https://github.com/jlevy/fdu/pull/48#issuecomment-5658305981"
resolution: null
duplicate_of: null
---
Medium, introduced by f276cb5. At f917cb7: opened.rs:1529-1545 (Unreadable transition), index.rs:1650-1658 and retain_issue 1693-1700 (append only, no key; nothing removes an issue when the path later reconciles cleanly). Each provider gap over an unreadable 'blocked' retains one ObservationGap and one Permission issue; after 32 events the 64-slot list holds only copies and every later distinct issue is omitted with no text. Also the Permission issue carries an absolute path while ObservationGap carries the relative one. Fix: dedupe retain_issue by (kind, path); drop a path's retained issues once a later reconcile of that path completes without error; make scan-error issue paths root-relative. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5193206420
