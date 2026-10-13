---
type: is
id: is-01m4v2r2sx4pgcwxjfse8kre8n
title: Describe current fdu behavior in README without release history
kind: task
status: in_progress
priority: 2
version: 4
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-13T16:50:18.556Z
updated_at: 2026-10-13T17:45:27.660Z
started_at: 2026-10-13T16:50:35.156Z
---
User requested README-only cleanup: describe current behavior and remove historical version comparisons and performance charts, linking to the existing measurements report instead. Preserve useful installation compatibility, evidence provenance, author voice, and other agents work. Scope: README.md only; coordinator owns review, gates, PR, and release.

## Notes

README-only cleanup committed as 1c42c91f on codex/readme-current-state-handoff and pushed. Draft PR https://github.com/jlevy/fdu/pull/202 includes the complete Linux release handoff, exact comparison commits, completed Mac evidence links, remaining gates, and separate historical-chart followup. Coordinator reviewed patch; make docs-format, git diff --check, 29 local links passed. Full make check remains pending due Mac scratch capacity; independent formal PR review and Linux gates required before ready/merge. Other package surfaces still say Fastest while README says Fast; disposition documented in PR. Keep bead open until gate and PR workflow complete.
