---
type: is
id: is-01m4pft840jyjcraxpbwskrcsn
title: "PR #196 A2: document the tree size styling exception"
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
created_at: 2026-10-11T22:02:28.862Z
updated_at: 2026-10-11T22:10:04.367Z
started_at: 2026-10-11T22:02:43.420Z
closed_at: 2026-10-11T22:10:04.367Z
close_reason: "Fixed in pushed commit 379c1c90ad254f92a2ace9a355312e35ae577390. Focused regressions: 7 Python tests pass (red baseline 5 failures); cli-animate entry point passes 33 Node tests plus 7 Python tests; make docs-format and git diff --check pass. Marked dispositions published: review A https://github.com/jlevy/fdu/pull/196#issuecomment-6114393489; review B https://github.com/jlevy/fdu/pull/196#issuecomment-6114394003. Coordinator owns pending #194 integration and full gate/CI."
resolution: null
duplicate_of: null
---
Review A2 Low, https://github.com/jlevy/fdu/pull/196#issuecomment-6114226735, docs/project/architecture/fdu-output-design.md:189. Qualify the styled_bytes statement with the tree-column tree_size_cell exception and link to Row Styling. Preserve the common-doc footer and run docs-format.
