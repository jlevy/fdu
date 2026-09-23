---
type: is
id: is-01m36bkft17tw7vbat35hsw790
title: Refresh persistence is keyed to this pass mutation, so owed saves can be lost
kind: bug
status: closed
priority: 2
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:20.320Z
updated_at: 2026-09-23T08:06:59.530Z
closed_at: 2026-09-23T08:06:59.529Z
close_reason: "Confirmed real (failed-save shape) and fixed in 4b40ff4c on #115; delta-reviewed; CI green"
resolution: null
duplicate_of: null
---
Stack review R115-1 (#115), PLAUSIBLE. lib.rs refresh calls persist_index_changes with report.apply.mutated(). If pass 1 mutates but is partial (entries not writable) and pass 2 is complete but mutates nothing, the snapshot never receives pass 1 facts; a failed save has the same shape. Cache-only reads then answer older (stale-labeled) facts than the index holds. Fix: carry a persistence-owed bit on Index/PyIndex like watch_session Persistence.pending, cleared only by a completed metadata write. First write a test to confirm whether pass 2 reports mutated().
