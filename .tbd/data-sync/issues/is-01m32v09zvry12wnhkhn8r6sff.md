---
type: is
id: is-01m32v09zvry12wnhkhn8r6sff
title: "PR #96 review R11: read_controls / --no-gitignore never mentioned (Medium)"
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m32h6cjrkz58ce2t05m3n2v7
created_at: 2026-09-21T20:37:31.259Z
updated_at: 2026-09-21T20:37:31.259Z
---
R11 Medium. plan :156-160, :222, :558. Fix: state the refusal for ignored-state selection, ignored stays null on rows and aggregates, and that read_controls=false is a separate snapshot scope.
