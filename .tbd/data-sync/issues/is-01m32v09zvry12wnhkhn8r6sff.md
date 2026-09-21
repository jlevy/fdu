---
type: is
id: is-01m32v09zvry12wnhkhn8r6sff
title: "PR #96 review R11: read_controls / --no-gitignore never mentioned (Medium)"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m32h6cjrkz58ce2t05m3n2v7
created_at: 2026-09-21T20:37:31.259Z
updated_at: 2026-09-21T21:11:45.397Z
closed_at: 2026-09-21T21:11:45.397Z
close_reason: "Fixed in c3aeed8a on codex/directory-query-plan (R1 implementation in 5d6e56a2 on #103). Disposition: https://github.com/jlevy/fdu/pull/96"
resolution: null
duplicate_of: null
---
R11 Medium. plan :156-160, :222, :558. Fix: state the refusal for ignored-state selection, ignored stays null on rows and aggregates, and that read_controls=false is a separate snapshot scope.
