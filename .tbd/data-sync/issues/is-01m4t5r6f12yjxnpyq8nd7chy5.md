---
type: is
id: is-01m4t5r6f12yjxnpyq8nd7chy5
title: Run final candidate regression and stability gates after disk cleanup
kind: task
status: in_progress
priority: 2
version: 2
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
hold: null
hold_until: null
created_at: 2026-10-13T08:23:33.588Z
updated_at: 2026-10-13T08:24:05.797Z
started_at: 2026-10-13T08:24:05.795Z
---
Run final e6b8850b release regression/stability gates with external build/test scratch. Any performance measurements use internal executable, source and caches per user request; preserve evidence and remove generated files after checks. Do not waive Linux performance, Library discrepancy, or human terminal gates.
