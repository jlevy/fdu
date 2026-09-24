---
type: is
id: is-01m392gm0je55zdmd5eavz0pzf
title: Add fdu --install-skill (project and user scope, idempotent, refuses foreign files)
kind: task
status: closed
priority: 1
version: 3
labels:
  - skill
dependencies:
  - type: blocks
    target: is-01m392gn7x58wqkjdzxng113py
parent_id: is-01m392gedkq4bn4qbxbedkt29f
created_at: 2026-09-24T06:44:12.433Z
updated_at: 2026-09-24T08:42:47.516Z
closed_at: 2026-09-24T08:42:47.491Z
close_reason: "Shipped in PR #122 (merged to main as 0162e951, 2026-09-24): make check and cross-lint on tree 14e8c200, CI green, delta reviews approved."
resolution: null
duplicate_of: null
---
Flag in the CLI crate beside --skill/--docs: default writes .agents/skills/fdu/SKILL.md and .claude/skills/fdu/SKILL.md at the git root (or cwd if none); --agent-base DIR for user scope (~/.claude, Codex's user skills dir); temp file + rename; reports installed/updated/unchanged; refuses to overwrite a SKILL.md lacking fdu's generated marker. No AGENTS.md block. Tests: a tryscript golden in a temp dir, byte-identical rerun, refusal case; add to the parity shim's declined list (tests/parity/py/parity_cli.py). Document uninstall as removing the two directories.
