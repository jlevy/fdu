---
type: is
id: is-01m47zjtznrvpex6edd3kagpt9
title: Review and update the agent skill and its installation for the 0.4.0 changes
kind: task
status: in_progress
priority: 1
version: 2
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
hold: null
hold_until: null
created_at: 2026-10-06T06:49:26.761Z
updated_at: 2026-10-06T06:49:58.285Z
started_at: 2026-10-06T06:49:58.284Z
---
Full review of the fdu agent skill (crates/fdu/src/skills/SKILL.md), fdu --skill / --install-skill (where it installs, idempotence, refusal of foreign skills, runner line), the skill's own tests and goldens, and the README/usage sections about agent setup, against the 0.4.0 changes: notes/tips after the result and consolidated (#174), content views imply analysis (#177), and the document-shares fix. Make the skill correct, concise, and leading with the common commands (tree, then --view code,documents).
