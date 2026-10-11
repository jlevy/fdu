---
type: is
id: is-01m4pft5tz1wj4tpcjs2wzsfvm
title: "PR #196 B2: fail missing demo trees and report actual timestamp updates"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: codex-fix-colors-review
labels: []
dependencies: []
parent_id: is-01m4pfsmyd25m58ce70tbe59gt
hold: null
hold_until: null
created_at: 2026-10-11T22:02:26.525Z
updated_at: 2026-10-11T22:10:04.352Z
started_at: 2026-10-11T22:02:43.404Z
closed_at: 2026-10-11T22:10:04.352Z
close_reason: "Fixed in pushed commit 379c1c90ad254f92a2ace9a355312e35ae577390. Focused regressions: 7 Python tests pass (red baseline 5 failures); cli-animate entry point passes 33 Node tests plus 7 Python tests; make docs-format and git diff --check pass. Marked dispositions published: review A https://github.com/jlevy/fdu/pull/196#issuecomment-6114393489; review B https://github.com/jlevy/fdu/pull/196#issuecomment-6114394003. Coordinator owns pending #194 integration and full gate/CI."
resolution: null
duplicate_of: null
---
Review B2 Low, https://github.com/jlevy/fdu/pull/196#issuecomment-6114259196, packages/cli-animate/examples/fdu/commit-mtimes.py:70-80. Missing TREE currently succeeds with planned counts. Validate TREE before history lookup, report successful path/directory updates and skipped paths, fail mismatches honestly, and add a nonexistent-root fixture.
