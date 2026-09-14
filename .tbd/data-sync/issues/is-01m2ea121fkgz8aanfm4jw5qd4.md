---
type: is
id: is-01m2ea121fkgz8aanfm4jw5qd4
title: Commit a reproducible FSEvents daily-gap replay probe
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
created_at: 2026-09-13T21:16:01.454Z
updated_at: 2026-09-14T23:26:31.583Z
---
Follow up on the historical fdu-4q0e scratch spike with a committed probe and reproducible records. Compare stream flags with and without FullHistory using known pre-mutation fences; exercise overlap, create/edit/delete/rename, cross-process restart, crash, and 1h/24h/48h/7d gaps. Record OS, volume, exact flags, delivered IDs, replay and total cost, full-scan oracle parity or declared degradation. Do not infer retention from synthetic ancient/future IDs or enable journal defaults without evidence.

## Notes

2026-09-14 (PR #55 review, 4727de0): the plans now call the FSEvents mechanism history replay, with a replay cursor, so it does not collide with the engine's index journal (crates/fdu-core/src/opened/journal.rs). Scope unchanged. This probe's evidence also settles fdu-b9d5, which aligns the Source::JournalScoped rustdoc with the hedged history-loss interpretation.
