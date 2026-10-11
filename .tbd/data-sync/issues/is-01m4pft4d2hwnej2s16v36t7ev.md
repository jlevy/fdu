---
type: is
id: is-01m4pft4d2hwnej2s16v36t7ev
title: "PR #196 B1: prevent timestamp writes through symlink ancestors"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: codex-fix-colors-review
labels: []
dependencies: []
parent_id: is-01m4pfsmyd25m58ce70tbe59gt
hold: null
hold_until: null
created_at: 2026-10-11T22:02:25.056Z
updated_at: 2026-10-11T22:10:04.343Z
started_at: 2026-10-11T22:02:43.382Z
closed_at: 2026-10-11T22:10:04.342Z
close_reason: "Fixed in pushed commit 379c1c90ad254f92a2ace9a355312e35ae577390. Focused regressions: 7 Python tests pass (red baseline 5 failures); cli-animate entry point passes 33 Node tests plus 7 Python tests; make docs-format and git diff --check pass. Marked dispositions published: review A https://github.com/jlevy/fdu/pull/196#issuecomment-6114393489; review B https://github.com/jlevy/fdu/pull/196#issuecomment-6114394003. Coordinator owns pending #194 integration and full gate/CI."
resolution: null
duplicate_of: null
---
Review B1 Medium, https://github.com/jlevy/fdu/pull/196#issuecomment-6114259196, packages/cli-animate/examples/fdu/commit-mtimes.py:69-78. A mismatched TREE whose tracked-directory path is a symlink can date outside files and directories. Reject symlink directory ancestors, validate resolved parent containment, avoid following directory symlinks, and retain leaf-link-only timestamp updates. Add isolated regression fixtures.
