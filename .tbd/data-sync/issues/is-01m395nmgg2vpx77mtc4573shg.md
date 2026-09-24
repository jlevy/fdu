---
type: is
id: is-01m395nmgg2vpx77mtc4573shg
title: "PR #122 review R2: no test for the install I/O failure path (exit 1)"
kind: bug
status: closed
priority: 2
version: 3
delegate: claude-code@spud10
labels:
  - skill
  - review
dependencies: []
parent_id: is-01m395mzh0amwcywtecm7m1zkh
hold: null
hold_until: null
created_at: 2026-09-24T07:39:22.512Z
updated_at: 2026-09-24T07:45:18.598Z
started_at: 2026-09-24T07:40:44.803Z
closed_at: 2026-09-24T07:45:18.596Z
close_reason: "Fixed in b4bdb9b2 on claude/skill-install (PR #122); disposition posted on the PR."
resolution: null
duplicate_of: null
---
crates/fdu/src/skill_install.rs tests and cli.rs:3063-3130. Refusal (exit 2) is tested at both levels; the Io path (exit 1, 'cannot install the skill at ...') is not. Fix: a target whose parent is a regular file, unit and process level; assert variant/path, exit code, context line; pin R1's contract for the first target. PR #122.
