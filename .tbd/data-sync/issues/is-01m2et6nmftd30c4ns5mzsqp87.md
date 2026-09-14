---
type: is
id: is-01m2et6nmftd30c4ns5mzsqp87
title: "PR #48 review FIX48-7: widen fdu-1onj to name what still fails on a watched root"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2et5s9yyv9e6tz783rp2m6c
created_at: 2026-09-14T01:58:42.574Z
updated_at: 2026-09-14T01:58:42.574Z
---
Low, disclosure gap. At f917cb7: scan.rs:4195-4198; opened.rs:1547-1548, 468-477. The CLASS-1 disposition omits two consequences: on a watched root the root still ends Failed (later, after an extra full walk, and close() names the observation worker), and in steady state, once the cumulative bound is reached, the first event touching any .gitignore-bearing directory fails the observer the same way. No code change: append both to fdu-1onj's notes. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5193206420
