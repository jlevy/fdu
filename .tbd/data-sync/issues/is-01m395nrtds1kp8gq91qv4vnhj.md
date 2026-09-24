---
type: is
id: is-01m395nrtds1kp8gq91qv4vnhj
title: "PR #122 review R4: replacement takes default permissions, not the old file's"
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
created_at: 2026-09-24T07:39:26.924Z
updated_at: 2026-09-24T07:45:20.326Z
started_at: 2026-09-24T07:40:46.916Z
closed_at: 2026-09-24T07:45:20.324Z
close_reason: "Fixed in b4bdb9b2 on claude/skill-install (PR #122); disposition posted on the PR."
resolution: null
duplicate_of: null
---
crates/fdu/src/skill_install.rs:139-152. A chmod 444 SKILL.md is rewritten with the process default mode on update; flowmark-rs copies the mode. Nothing promises preservation. Fix: say so in replace()'s doc, or copy the mode under cfg(unix). PR #122.
