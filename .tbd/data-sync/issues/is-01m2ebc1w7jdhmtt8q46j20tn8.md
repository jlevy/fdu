---
type: is
id: is-01m2ebc1w7jdhmtt8q46j20tn8
title: "PR #48 review LIFE-4: discovery's Finish reopens a budget-stopped root"
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:39:30.310Z
updated_at: 2026-09-13T21:39:30.310Z
---
Medium. index.rs:1553-1563; opened.rs:926. A refresh refused by the budget moves the root to Stopped/Partial(Budget), but discovery's later Finish sets phase Ready unconditionally, so prioritize() succeeds and observation can reach Watching with Partial(Budget). Fix: Finish and Inaccessible never change a Stopped or Failed phase; discovery stops its frontier when the index reports Stopped. Related: fdu-97dd. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
