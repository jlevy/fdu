---
type: is
id: is-01m4pft4d2hwnej2s16v36t7ev
title: "PR #196 B1: prevent timestamp writes through symlink ancestors"
kind: bug
status: in_progress
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: codex-fix-colors-review
labels: []
dependencies: []
parent_id: is-01m4pfsmyd25m58ce70tbe59gt
hold: null
hold_until: null
created_at: 2026-10-11T22:02:25.056Z
updated_at: 2026-10-11T22:02:43.382Z
started_at: 2026-10-11T22:02:43.382Z
---
Review B1 Medium, https://github.com/jlevy/fdu/pull/196#issuecomment-6114259196, packages/cli-animate/examples/fdu/commit-mtimes.py:69-78. A mismatched TREE whose tracked-directory path is a symlink can date outside files and directories. Reject symlink directory ancestors, validate resolved parent containment, avoid following directory symlinks, and retain leaf-link-only timestamp updates. Add isolated regression fixtures.
