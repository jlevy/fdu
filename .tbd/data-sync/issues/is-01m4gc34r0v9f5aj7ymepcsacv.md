---
type: is
id: is-01m4gc34r0v9f5aj7ymepcsacv
title: Refresh ages on an idle watch screen when a shown age would change
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T13:01:59.423Z
updated_at: 2026-10-09T13:01:59.423Z
---
Review on #191 (forward-looking): a watch repaints only on filesystem events, and a repaint whose only difference is rolled-over ages is skipped, so an idle screen shows ages as of its last repaint. A timer that re-renders from the index (no filesystem work) when a displayed age cell would change would keep the column truthful; weigh against 'an idle tree costs no work'.
