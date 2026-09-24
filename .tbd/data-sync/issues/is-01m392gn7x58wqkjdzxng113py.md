---
type: is
id: is-01m392gn7x58wqkjdzxng113py
title: Align README, Python README, usage, release notes, and release process on install and skill wording
kind: task
status: open
priority: 1
version: 1
labels:
  - skill
dependencies: []
parent_id: is-01m392gedkq4bn4qbxbedkt29f
created_at: 2026-09-24T06:44:13.685Z
updated_at: 2026-09-24T06:44:13.685Z
---
One wording everywhere: zero-install uvx fdu@latest; persistent uv tool install fdu, upgrade uv tool upgrade fdu; cargo install --locked fdu; skill via fdu --install-skill. Fix: two runner spellings (SKILL.md --from vs README/release-notes/fdu-py README fdu@X), unpinned vs pinned mixes (pip install fdu==0.1.0 vs fdu), no upgrade instruction anywhere, no skill-install line (README ~89, usage.md ~315, release notes ~26, CHANGELOG ~82), broken pointer in crates/fdu-py/README.md ~191 to README#install. Keep the 'once published' hedges until 0.1.0 publishes, then drop them together; add 'uvx fdu@latest --skill' to the release process's after-publishing checklist.
