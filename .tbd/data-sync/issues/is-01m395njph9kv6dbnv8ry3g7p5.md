---
type: is
id: is-01m395njph9kv6dbnv8ry3g7p5
title: "PR #122 review R1: install() promises all-or-nothing but only the refusal is"
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
created_at: 2026-09-24T07:39:20.651Z
updated_at: 2026-09-24T07:45:17.582Z
started_at: 2026-09-24T07:40:43.647Z
closed_at: 2026-09-24T07:45:17.579Z
close_reason: "Fixed in b4bdb9b2 on claude/skill-install (PR #122); disposition posted on the PR."
resolution: null
duplicate_of: null
---
crates/fdu/src/skill_install.rs:91-127. Doc says 'every target, or none of them'; an Io failure on the second replace() leaves the first target installed and unreported (run_install_skill prints only the error, exit 1). Fix: narrow the doc to what holds and carry completed outcomes on InstallError::Io so they are printed before the error. PR #122.
