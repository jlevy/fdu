---
type: is
id: is-01kzq53dw7bawahp6m77sx9gd4
title: FSEvents historical replay via dispatch-queue API
kind: task
status: open
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzq53e2qffv7d9a2q7vg2yth
created_at: 2026-08-11T00:56:00.646Z
updated_at: 2026-09-14T23:26:34.631Z
---
Phase 2a. journal/fsevents.rs behind feature 'journal', cfg(target_os=macos): FFI declarations (fsevent-sys already in the locked tree via notify; self-declared FSEventStreamSetDispatchQueue, FSEventsCopyUUIDForDevice, dispatch_queue_create/release — zero new crates), current-event-id, volume UUID, and one-shot historical replay using the NON-DEPRECATED FSEventStreamSetDispatchQueue path (ScheduleWithRunLoop is deprecated since macOS 13). Callback marshals (path, flags) over a channel and nothing else; deadline enforces gate G6. Scoped #[allow(unsafe_code)] with per-call safety comments; no unsafe outside this module.

## Notes

Review 2026-09-13: use the committed probe fdu-uwhl to establish FullHistory overlap and exact stream flags. Retain event IDs through delivery and publish only a fully applied cursor with the corresponding inventory. UUID equality and HistoryDone do not prove complete history; the 24-hour age bound remains provisional and does not yet satisfy next-day refresh.

2026-09-14 (PR #55 review, 4727de0): the FSEvents plan renames journal/fsevents.rs to history_replay/fsevents.rs and the build feature 'journal' to 'history-replay', so the module does not collide with the engine's index journal (crates/fdu-core/src/opened/journal.rs).
