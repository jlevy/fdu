---
type: is
id: is-01m3jxaqxq7gvmmw3g6aytbtdt
title: Make watch progress equivalence independent of native event timing
kind: bug
status: in_progress
priority: 1
version: 2
delegate: codex@spud10.local
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-28T02:26:01.270Z
updated_at: 2026-09-28T02:26:11.325Z
started_at: 2026-09-28T02:26:11.323Z
---
PR137 CI at321882ac failed a_started_session_reports_its_second_pass_and_then_nothing: independently started macOS watchers produced different legitimate WatchSetupRace diagnostics. Preserve native progress smoke coverage and move exact observer/no-observer parity to identical scripted event streams using existing handoff seam. No error masking or production semantic changes.
