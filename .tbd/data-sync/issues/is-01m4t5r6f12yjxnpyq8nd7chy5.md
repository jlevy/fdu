---
type: is
id: is-01m4t5r6f12yjxnpyq8nd7chy5
title: Run final candidate regression and stability gates after disk cleanup
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
created_at: 2026-10-13T08:23:33.588Z
updated_at: 2026-10-13T08:23:33.588Z
---
Run final e6b8850b release regression/stability gates with external build/test scratch. Any performance measurements use internal executable, source and caches per user request; preserve evidence and remove generated files after checks. Do not waive Linux performance, Library discrepancy, or human terminal gates.
