---
type: is
id: is-01m2ea121fkgz8aanfm4jw5qd4
title: Commit a reproducible FSEvents daily-gap replay probe
kind: task
status: in_progress
priority: 1
version: 10
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
delegate: claude-code
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-13T21:16:01.454Z
updated_at: 2026-09-28T16:19:50.953Z
started_at: 2026-09-26T23:50:12.137Z
---
Follow up on the historical fdu-4q0e scratch spike with a committed probe and reproducible records. Compare stream flags with and without FullHistory using known pre-mutation fences; exercise overlap, create/edit/delete/rename, cross-process restart, crash, and 1h/24h/48h/7d gaps. Record OS, volume, exact flags, delivered IDs, replay and total cost, full-scan oracle parity or declared degradation. Do not infer retention from synthetic ancient/future IDs or enable journal defaults without evidence.

## Notes

2026-09-14 (PR #55 review, 4727de0): the plans now call the FSEvents mechanism history replay, with a replay cursor, so it does not collide with the engine's index journal (crates/fdu-core/src/opened/journal.rs). Scope unchanged. This probe's evidence also settles fdu-b9d5, which aligns the Source::JournalScoped rustdoc with the hedged history-loss interpretation.

2026-09-15 (PR #55 delta review 5205945198, a3fb4e8): the FSEvents plan's Phase 0 volume identity item gains a probe question. FSEvents.h documents the per-device UUID as stored on the device and travelling with it to other computers, so G3's UUID comparison does not catch a disk changed on another Mac and brought back. Record whether replay from a cursor saved before the move names those changes, and which gate (G4, G7, or another) falls closed if it does not. G3 no longer lists "moved disk".

2026-09-26: Reproducible probe and README now in explorations/fsevents-replay with 48 hash-identified native trials (24 internal APFS v1, 24 external APFS v2). Quiet/mixed/deep cases match the independent lstat oracle; omitted-scope negative controls fail as intended. FullHistory overlap widens the conservative candidate to the root. Parent independent mixed fixture: 8 further passes, four self-tests pass. Keep open: long-gap, reboot, loss, quiet-boundary advancement, broader semantics and engine integration are pending. These are bounded mechanism tests, not whole-command performance claims.

2026-09-27: Added real_tree.py (fdu-eqdr) and sanitized observations. Quiet 12,280-entry root matched both oracles with no candidate entry scan. Live 441,777-entry root: zero stable mismatches, 38 concurrent paths (inconclusive), conservative normalization widened to the full root. Next-day external fixtures (~26h) delivered events but all 16 attempts timed out at 10s without HistoryDone; not a retention pass or evidence of lost history. Priorities: validate shallow relist versus recursive invalidation; investigate bounded completion and volume-history cost. Production replay remains gated. See explorations/fsevents-replay/observations-2026-09-27.json and research-2026-09-27-persistent-change-prior-art.md.

2026-09-27 continuation: Controlled 20,206-entry tree matched with 712 observations. Busy 454,775-entry root had 1 then 2 stable misses despite HistoryDone. Aged owned-file append+fsync with descriptor held open reproduced zero fresh events and a stable miss; closing and replaying the same cursor matched exactly, independently repeated. Track active-writer coverage in fdu-vhrb. Day-old completion varied: initial 32-33s, final 60s timeouts, then a 120s diagnostic completed at 92.5s; matching callbacks arrived in milliseconds. Default deadline unchanged. Keep this matrix open for bounded completion, 48h/7d, reboot/loss, quiet cursor advancement and engine integration. Evidence: explorations/fsevents-replay/observations-2026-09-27-continuation.json. Continuation task fdu-uz5r.

2026-09-27 change-source review (research-2026-09-27-disk-growth-change-sources.md, epic fdu-tawn): replay cost = ~0.124 s per compressed MB of the volume's journal behind the cursor (r2 0.98, n=26, CPU-bound in fseventsd) + ~10 us per matching record, independent of path filter and age. Day on the churning external scratch volume: 45 s; quiet folder on the internal Data volume: 1.8 s. The 32 -> 92.5 s variance coincided with abandoned replays and a concurrent scan, not history growth (+1.6%); backlog test fdu-lwcz. FlushSync at start blocks until HistoryDone; FlushAsync returns 0. The device-relative filter must be the firmlink-free path minus the mount point (fdu-43bc). Use a budget rule, not a longer deadline. Home-filter matching cost: fdu-yj8z; multi-path stream start failure: fdu-befp.

Correction (2026-09-27, final replay-cost results): abandoned replays leave NO backlog (fdu-lwcz closed); the 32 -> 92.5 s variance is contention with other fseventsd clients plus host load (concurrent same-volume replays 1.6x, cross-volume 2x, host state alone 1.8x). Budgeted attempts that abandon and fall back to the walk are safe.
