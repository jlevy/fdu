---
type: is
id: is-01m4pft77gg5mtw8mph64pvfv8
title: "PR #196 A1: preserve NUL-framed Git filenames with newlines"
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
created_at: 2026-10-11T22:02:27.951Z
updated_at: 2026-10-11T22:02:43.412Z
started_at: 2026-10-11T22:02:43.412Z
---
Review A1 Low, https://github.com/jlevy/fdu/pull/196#issuecomment-6114226735, packages/cli-animate/examples/fdu/commit-mtimes.py:45. A tracked a-newline-b filename incorrectly dates b in the same log field. Parse commit framing separately from complete NUL-delimited paths and regress the two-name collision plus marker-looking filenames.
