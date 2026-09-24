---
type: is
id: is-01m395ntf98ec3w1jfnxy1acvh
title: "PR #122 review R5: let _ = remove_file(staged) lacks its stated reason"
kind: bug
status: closed
priority: 3
version: 3
delegate: claude-code@spud10
labels:
  - skill
  - review
dependencies: []
parent_id: is-01m395mzh0amwcywtecm7m1zkh
hold: null
hold_until: null
created_at: 2026-09-24T07:39:28.615Z
updated_at: 2026-09-24T07:45:21.168Z
started_at: 2026-09-24T07:40:47.992Z
closed_at: 2026-09-24T07:45:21.165Z
close_reason: "Fixed in b4bdb9b2 on claude/skill-install (PR #122); disposition posted on the PR."
resolution: null
duplicate_of: null
---
crates/fdu/src/skill_install.rs:148-150. rust-rules: a discarded fallible result needs an explicit reason at the discard. Fix: one comment. PR #122.
