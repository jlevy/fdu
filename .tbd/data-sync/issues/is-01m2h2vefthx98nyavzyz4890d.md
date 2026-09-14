---
type: is
id: is-01m2h2vefthx98nyavzyz4890d
title: "PR #55 review PR55-STALE-2: plan's journal naming collides with opened/journal.rs"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
parent_id: is-01m2h2v87crh401e90pfe52w8g
created_at: 2026-09-14T23:08:20.857Z
updated_at: 2026-09-14T23:54:32.814Z
closed_at: 2026-09-14T23:54:32.809Z
close_reason: "4727de0: FSEvents mechanism renamed history replay / replay cursor (history_replay module, ReplayCursor, history-replay build feature, max_cursor_age, warm-revalidate-replay); terminology notes separate it from opened/journal.rs"
resolution: null
duplicate_of: null
---
PR #55 at dc27c14: checkpoints plan :34 and FSEvents plan :383-389, :399 name a journal module and build feature that collide with main's crates/fdu-core/src/opened/journal.rs, the index journal (index.rs DEFAULT_JOURNAL_CAPACITY) and #56's journal_capacity_bytes. Pick a distinct name and use it consistently.
