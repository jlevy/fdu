---
type: is
id: is-01m392gedkq4bn4qbxbedkt29f
title: Self-installing agent skill and aligned install docs (uvx fdu@latest, uv tool install, --install-skill)
kind: epic
status: open
priority: 1
version: 4
labels:
  - release
  - skill
dependencies: []
child_order_hints:
  - is-01m392ghc6fp1q565cbz4h0xyg
  - is-01m392gm0je55zdmd5eavz0pzf
  - is-01m392gn7x58wqkjdzxng113py
created_at: 2026-09-24T06:44:06.701Z
updated_at: 2026-09-24T06:44:13.685Z
---
Decision (user, 2026-09-24): the skill tells agents to prefer an installed fdu on PATH, else run uvx fdu@latest; persistent installs use uv tool install fdu and uv tool upgrade fdu; fdu gains a flowmark-style --install-skill. Reverses fdu-gxvl (non-goal) and SKILL.md's 'never latest' line; overrides the tbd cli-agent-skill-patterns pin rule by the user's choice. Research (2026-09-24) compared tbd guidelines, flowmark (skill.py:118-150, 203-212, 381-387, 454-473) and flowmark-rs (skills/mod.rs:72-76,117-135). fdu is not yet on PyPI/crates.io, so zero-install works only after 0.1.0 publishes.
