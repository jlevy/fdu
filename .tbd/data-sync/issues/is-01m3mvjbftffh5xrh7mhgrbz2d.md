---
type: is
id: is-01m3mvjbftffh5xrh7mhgrbz2d
title: Self-document the stacking, review, and parallel-id rules
kind: task
status: closed
priority: 1
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-28T20:33:42.392Z
updated_at: 2026-09-28T20:47:44.573Z
started_at: 2026-09-28T20:33:43.006Z
closed_at: 2026-09-28T20:47:44.563Z
close_reason: "Merged #154 (main 0d73ed54): AGENTS.md stacking and review rules; parallel id blocks named in the runbook's Current Pickup, with the RECORD and PREDICT steps and the registry header pointing there. Reviewed; review fixes applied."
resolution: null
duplicate_of: null
---
User 2026-09-28: the repo should self-document lasting, non-obvious guidance so handoffs carry only situational detail. Gaps found: tbd's stacked-prs shortcut prescribes rebase and force-push, but this project merges upward; the independent-review-as-PR-comment convention is undocumented; nothing stops two platforms from claiming the same experiment or hypothesis ids.
