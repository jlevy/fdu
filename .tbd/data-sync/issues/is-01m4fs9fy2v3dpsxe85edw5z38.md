---
type: is
id: is-01m4fs9fy2v3dpsxe85edw5z38
title: Record the 0.4.0 release
kind: task
status: in_progress
priority: 1
version: 3
delegate: claude-code@spud10
labels:
  - release
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
hold: null
hold_until: null
created_at: 2026-10-09T07:33:24.545Z
updated_at: 2026-10-09T07:42:33.534Z
started_at: 2026-10-09T07:33:27.114Z
---
Checklist step 12 (Clean up): commit the durable record of fdu 0.4.0 (published 2026-10-09). Adds docs/project/reports/report-2026-10-09-release-0.4.0-stability-pass.md (the stability pass from make release-stability, regime confirmed, plus a Release Record section: release commit, tag, rehearsal and publishing runs, GitHub release, make release-published, make release-announced, homepage checks), updates the QA playbook's Current Status and the correctness runbook's Last Recorded Run, and marks 0.4.0 published in TODO.md. Delivered as a draft PR against main.

## Notes

2026-10-09: draft PR https://github.com/jlevy/fdu/pull/190, head c05712f125db1198eeb84618c410b5a58aaeb34b, branch claude/release-0.4.0-record off main c041ed1c. Docs only: new report-2026-10-09-release-0.4.0-stability-pass.md (stability pass + Release Record), QA playbook Current Status, correctness runbook Last Recorded Run, 0.3.0 report anchor link, TODO.md. Found: Phase 4 medium-tree subdirectory analyze did not run (FDU_QA_MEDIUM_ANALYZE unset, Linux tree has Documentation/ not docs/) and the harness does not count it as a skip. Not recorded: After Publishing's three by-eye checks. Awaiting review; not merged.
