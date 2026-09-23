---
type: is
id: is-01m35t596gbsj2mgy77cdpben8
title: "PR #103 A103-1: preserve flat diagnostics on opened Python reports"
kind: bug
status: in_progress
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
delegate: codex-alpha-coordinator
labels: []
dependencies: []
parent_id: is-01m35sm4jn2ytfmy91136g50b7
hold: null
hold_until: null
created_at: 2026-09-23T00:20:29.005Z
updated_at: 2026-09-23T01:47:53.852Z
started_at: 2026-09-23T01:32:16.719Z
---
At 467609e6, opened_binding.rs:1001 returns raw Report.notes instead of the flat_diagnostics selected by PyOneShot.notes. Runtime repro: retained Paths limit=1 over two directories reports (1 of 2; --limit all for every one), opened Report.notes is empty, both have bound 1/2. Use one shared diagnostic path and test bounded Paths/Long and progressive incomplete coverage on opened Python reports.

## Notes

Implemented in isolated branch codex/alpha-opened-diagnostics. Opened Python report conversion now obtains notes from the same PyOneShot diagnostic method as retained reports. Bounded Paths/Long and budget-limited partial reports both reproduced the missing notes before the fix; all seven directory-query tests pass after rebuilding the native extension. Ruff and formatting pass. Independent Astra review found no findings. The complete make check gate is running before commit and publication to PR103; no bead closure or merge yet.
