---
type: is
id: is-01m4fs9fy2v3dpsxe85edw5z38
title: Record the 0.4.0 release
kind: task
status: in_progress
priority: 1
version: 4
delegate: claude-code@spud10
labels:
  - release
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
hold: null
hold_until: null
created_at: 2026-10-09T07:33:24.545Z
updated_at: 2026-10-09T08:05:42.600Z
started_at: 2026-10-09T07:33:27.114Z
---
Checklist step 12 (Clean up): commit the durable record of fdu 0.4.0 (published 2026-10-09). Adds docs/project/reports/report-2026-10-09-release-0.4.0-stability-pass.md (the stability pass from make release-stability, regime confirmed, plus a Release Record section: release commit, tag, rehearsal and publishing runs, GitHub release, make release-published, make release-announced, homepage checks), updates the QA playbook's Current Status and the correctness runbook's Last Recorded Run, and marks 0.4.0 published in TODO.md. Delivered as a draft PR against main.

## Notes

2026-10-09: PR https://github.com/jlevy/fdu/pull/190. Head c05712f1 (record), then 0b8e4af4 (review A: by-eye checks recorded, fdu-15gr cited). CI run 37901907719 success at 0b8e4af4. Review A dispositions posted (issuecomment-6076996872); PR marked ready, not merged. Close this bead when #190 merges.
