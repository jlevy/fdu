---
type: is
id: is-01kzzbrrjhtjae841swca4fpc8
title: "PR #20 R4: make FSEvents plan consistently use a pre-scan cursor"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01kzzanfm0vcgrcdmjwr90rcja
created_at: 2026-08-14T05:26:26.640Z
updated_at: 2026-08-14T05:38:24.618Z
closed_at: 2026-08-14T05:38:24.617Z
close_reason: Reconciled the format, gate table, component ownership, API notes, and Phase 2 checklist around one pre-scan cursor fence; docs formatting passes.
---
Correct the snapshot format, component ownership, and Phase 2 checklist so the event ID and timestamp are captured immediately before the scan and only persisted by snapshot save; eliminate remaining save-time capture wording.
