---
type: is
id: is-01m2ea121fkgz8aanfm4jw5qd4
title: Commit a reproducible FSEvents daily-gap replay probe
kind: task
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-13T21:16:01.454Z
updated_at: 2026-09-27T00:21:32.449Z
started_at: 2026-09-26T23:50:12.137Z
---
Follow up on the historical fdu-4q0e scratch spike with a committed probe and reproducible records. Compare stream flags with and without FullHistory using known pre-mutation fences; exercise overlap, create/edit/delete/rename, cross-process restart, crash, and 1h/24h/48h/7d gaps. Record OS, volume, exact flags, delivered IDs, replay and total cost, full-scan oracle parity or declared degradation. Do not infer retention from synthetic ancient/future IDs or enable journal defaults without evidence.

## Notes

2026-09-14 (PR #55 review, 4727de0): the plans now call the FSEvents mechanism history replay, with a replay cursor, so it does not collide with the engine's index journal (crates/fdu-core/src/opened/journal.rs). Scope unchanged. This probe's evidence also settles fdu-b9d5, which aligns the Source::JournalScoped rustdoc with the hedged history-loss interpretation.

2026-09-15 (PR #55 delta review 5205945198, a3fb4e8): the FSEvents plan's Phase 0 volume identity item gains a probe question. FSEvents.h documents the per-device UUID as stored on the device and travelling with it to other computers, so G3's UUID comparison does not catch a disk changed on another Mac and brought back. Record whether replay from a cursor saved before the move names those changes, and which gate (G4, G7, or another) falls closed if it does not. G3 no longer lists "moved disk".

2026-09-26: Reproducible probe and README now in explorations/fsevents-replay with 48 hash-identified native trials (24 internal APFS v1, 24 external APFS v2). Quiet/mixed/deep cases match the independent lstat oracle; omitted-scope negative controls fail as intended. FullHistory overlap widens the conservative candidate to the root. Parent independent mixed fixture: 8 further passes, four self-tests pass. Keep open: long-gap, reboot, loss, quiet-boundary advancement, broader semantics and engine integration are pending. These are bounded mechanism tests, not whole-command performance claims.
