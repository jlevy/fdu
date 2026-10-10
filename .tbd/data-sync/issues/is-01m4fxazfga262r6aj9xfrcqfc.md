---
type: is
id: is-01m4fxazfga262r6aj9xfrcqfc
title: Watch several roots in one session
kind: feature
status: open
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T08:44:07.534Z
updated_at: 2026-10-10T21:49:46.461Z
---
Out of scope for 0.5.0, which refuses --watch with a second PATH. Needs a watcher per root, a merged repaint, and a change stream whose paths carry the root label.

## Notes

Review E10 on PR #192 (jlevy/fdu#192): write_shown_activity (watch_session.rs) sees only Section::Tree { root: Some(..) }. Nothing is wrong while watch refuses several roots, but a several-root watch must hash roots.total and each roots.trees[].tree into the idle-repaint identity, or that identity misses every age change.
