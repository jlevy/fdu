---
type: is
id: is-01m4ps2pk8vt2j5kqhvrtnazec
title: "PR #198 A1: distinguish Library and subtree allocation differences"
kind: bug
status: in_progress
priority: 3
version: 2
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
hold: null
hold_until: null
created_at: 2026-10-12T00:44:23.015Z
updated_at: 2026-10-12T00:45:01.531Z
started_at: 2026-10-12T00:45:01.530Z
---
Low A1 in review A, https://github.com/jlevy/fdu/pull/198#issuecomment-6115732999, pinned fe8c1f608a2c6eea0f59c2c17be1050baa086003. docs/project/reports/report-2026-10-12-release-0.5.0-stability-pass.md:53-55 conflates the whole Library GNU du -l allocation difference with the 6,205,440-byte Application Support difference relative to preceding fdu. Correct only prose, preserve all tables, run focused privacy/format/evidence checks. Coordinator owns push and disposition; close after independent confirmation.
