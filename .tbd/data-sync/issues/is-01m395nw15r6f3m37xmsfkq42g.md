---
type: is
id: is-01m395nw15r6f3m37xmsfkq42g
title: "PR #122 review R6: install golden assumes the temp sandbox has no .git ancestor"
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
created_at: 2026-09-24T07:39:30.212Z
updated_at: 2026-09-24T07:45:21.939Z
started_at: 2026-09-24T07:40:48.827Z
closed_at: 2026-09-24T07:45:21.928Z
close_reason: "Fixed in b4bdb9b2 on claude/skill-install (PR #122); disposition posted on the PR."
resolution: null
duplicate_of: null
---
tests/golden/cli-surface.tryscript.md:767-795 and skill_install.rs:186-190. With TMPDIR inside a checkout the walk-up writes into that repository's root, then the golden fails (paths become absolute). Fix: mkdirSync('.git') in the sandbox before the first install so the session is its own root and shows the primary walk-up case. PR #122.
