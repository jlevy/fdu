---
type: is
id: is-01m4pft5tz1wj4tpcjs2wzsfvm
title: "PR #196 B2: fail missing demo trees and report actual timestamp updates"
kind: bug
status: in_progress
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: codex-fix-colors-review
labels: []
dependencies: []
parent_id: is-01m4pfsmyd25m58ce70tbe59gt
hold: null
hold_until: null
created_at: 2026-10-11T22:02:26.525Z
updated_at: 2026-10-11T22:02:43.404Z
started_at: 2026-10-11T22:02:43.404Z
---
Review B2 Low, https://github.com/jlevy/fdu/pull/196#issuecomment-6114259196, packages/cli-animate/examples/fdu/commit-mtimes.py:70-80. Missing TREE currently succeeds with planned counts. Validate TREE before history lookup, report successful path/directory updates and skipped paths, fail mismatches honestly, and add a nonexistent-root fixture.
