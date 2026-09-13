---
type: is
id: is-01m2ebc0nr76aax2mkq8h342q8
title: "PR #48 review READ-2: a full flat page is discarded when the scan-ahead exhausts max_work"
kind: bug
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:39:29.079Z
updated_at: 2026-09-13T21:39:29.079Z
---
High. opened/read.rs:596-624. The flat loop charges work and checks spent > max_work before checking whether the page is already full, so a page holding limit rows returns Limit when the look-ahead for next runs out of budget; a fixed-budget client can never page past that position. Fix: when rows.len() == page.limit, set next to the current entry and stop before charging further budget. Envelope defect under fdu-91ru. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
