---
type: is
id: is-01m395nq4pejjgydhs31wb9fcw
title: "PR #122 review R3: symlink policy at the skill target is not stated"
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
created_at: 2026-09-24T07:39:25.189Z
updated_at: 2026-09-24T07:45:19.433Z
started_at: 2026-09-24T07:40:45.754Z
closed_at: 2026-09-24T07:45:19.431Z
close_reason: "Fixed in b4bdb9b2 on claude/skill-install (PR #122); disposition posted on the PR."
resolution: null
duplicate_of: null
---
crates/fdu/src/skill_install.rs:97-152. fs::read follows a symlinked SKILL.md (passes the marker check via its target) and fs::rename then replaces the link with a regular file; directory links are followed by create_dir_all/File::create. Fix (pick one): state the policy in the module doc, or refuse a symlinked target. PR #122.
