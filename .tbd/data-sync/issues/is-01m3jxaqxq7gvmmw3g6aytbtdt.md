---
type: is
id: is-01m3jxaqxq7gvmmw3g6aytbtdt
title: Make watch progress equivalence independent of native event timing
kind: bug
status: closed
priority: 1
version: 3
delegate: codex@spud10.local
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-28T02:26:01.270Z
updated_at: 2026-09-28T02:51:22.621Z
started_at: 2026-09-28T02:26:11.323Z
closed_at: 2026-09-28T02:51:22.620Z
close_reason: be93cba0 moves exact progress report equivalence to identical scripted watcher streams with Summary coverage, preserves native start/progress/completeness checks, and masks no diagnostics. Full local gate and final macOS/Linux/Windows CI pass; senior review5333468298 confirms no findings.
resolution: null
duplicate_of: null
---
PR137 CI at321882ac failed a_started_session_reports_its_second_pass_and_then_nothing: independently started macOS watchers produced different legitimate WatchSetupRace diagnostics. Preserve native progress smoke coverage and move exact observer/no-observer parity to identical scripted event streams using existing handoff seam. No error masking or production semantic changes.
