---
type: is
id: is-01m47zjtznrvpex6edd3kagpt9
title: Review and update the agent skill and its installation for the 0.4.0 changes
kind: task
status: closed
priority: 1
version: 3
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
hold: null
hold_until: null
created_at: 2026-10-06T06:49:26.761Z
updated_at: 2026-10-06T09:48:37.948Z
started_at: 2026-10-06T06:49:58.284Z
closed_at: 2026-10-06T09:48:37.936Z
close_reason: "Skill reviewed and updated on claude/skill-refresh (1f7cbf85, 927f988d, b80baa7f, b14cfcb6, merge 6c4a77ae, b86d98c9): leads with fdu ., --view=code,documents, and a bounded JSON tree; new section on the result/notes/tips contract; stale omission-row, percentage-label, gitignored-pair, and perf wording replaced; --docs remainder row fixed; new test resolves every skill command and the vocabulary test covers --view=; goldens and parity re-recorded; install behavior verified; make check exit 0. Not pushed: awaiting the document-shares merge."
resolution: null
duplicate_of: null
---
Full review of the fdu agent skill (crates/fdu/src/skills/SKILL.md), fdu --skill / --install-skill (where it installs, idempotence, refusal of foreign skills, runner line), the skill's own tests and goldens, and the README/usage sections about agent setup, against the 0.4.0 changes: notes/tips after the result and consolidated (#174), content views imply analysis (#177), and the document-shares fix. Make the skill correct, concise, and leading with the common commands (tree, then --view code,documents).
