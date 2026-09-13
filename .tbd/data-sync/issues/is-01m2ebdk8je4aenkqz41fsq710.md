---
type: is
id: is-01m2ebdk8je4aenkqz41fsq710
title: "PR #48 review LIFE-8: journal capacity counts items, not bytes"
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:20.881Z
updated_at: 2026-09-13T21:40:20.881Z
---
Low. engine_contract.rs:1560-1565; index.rs:1696-1713. Retention counts changes, transitions, and dirty paths as items, so a 64 KiB item budget can hold tens of MiB of paths, and since() clones all of it. Fix: weight retention by bytes. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
