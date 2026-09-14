---
type: is
id: is-01m2ebdmy2cawv6vxnbv1fmqd3
title: "PR #48 review CLASS-5: gitignore bracket expressions diverge from git"
kind: bug
status: open
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:22.592Z
updated_at: 2026-09-14T01:49:45.593Z
---
Low. control/gitignore.rs:226-250. Checked against git check-ignore: [[:alpha:]], [a\-z], and [\]] answer differently; the module doc's 'character classes' overstates. Fix: support [:class:] and escapes inside classes, or document the unsupported forms. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
