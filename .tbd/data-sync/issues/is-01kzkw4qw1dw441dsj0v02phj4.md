---
type: is
id: is-01kzkw4qw1dw441dsj0v02phj4
title: Reverify queued watch observations at the index commit boundary
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
parent_id: is-01kzkw4ddv9g9jry50tp4xzgtw
created_at: 2026-08-09T18:21:43.168Z
updated_at: 2026-08-09T18:31:47.233Z
closed_at: 2026-08-09T18:31:47.232Z
close_reason: Implemented with regression coverage; the complete local handoff gate passes.
---
A watcher stats paths before placing an Observation on an unbounded queue. A reconciliation can commit newer state before that queued unconditional observation is applied, allowing the old sample to overwrite it. The applying driver must reverify against the indexed root immediately before a clock-stable commit, reject watcher/index root mismatches, and treat root disappearance as invalidation rather than a no-op remove.
