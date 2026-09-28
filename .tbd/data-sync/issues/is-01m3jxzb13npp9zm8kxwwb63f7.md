---
type: is
id: is-01m3jxzb13npp9zm8kxwwb63f7
title: Rename analysis-workers CLI option to workers after scope review
kind: task
status: in_progress
priority: 2
version: 3
delegate: codex@spud10.local
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-28T02:37:16.194Z
updated_at: 2026-09-28T02:42:01.598Z
started_at: 2026-09-28T02:38:36.876Z
---

## Notes

No existing --workers CLI conflict. Rename flag to --workers while preserving content-analysis worker semantics and automatic zero; directory scan concurrency remains its engine-owned pool. Updated CLI help/docs and Python parity adapter; parser test covers new spelling.
