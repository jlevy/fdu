---
type: is
id: is-01m2ebdq4c8z47yct91aand7rh
title: "PR #48 review PY-4: the watch repaint capture filter is broader than documented"
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:24.843Z
updated_at: 2026-09-13T21:40:24.843Z
---
Low. tests/golden/bin/watch-repaint-capture.mjs:151, 173 drop every line ending in a tab plus invalidate, while the rationale covers only the empty-path root invalidation. Fix: match line === '\tinvalidate'. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
