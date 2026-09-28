---
type: is
id: is-01m3jxzb13npp9zm8kxwwb63f7
title: Rename analysis-workers CLI option to workers after scope review
kind: task
status: in_progress
priority: 2
version: 6
spec_path: docs/project/specs/done/plan-2026-09-26-code-analysis-presentation.md
delegate: codex@spud10.local
labels:
  - cli-presentation
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-28T02:37:16.194Z
updated_at: 2026-09-28T03:18:49.962Z
started_at: 2026-09-28T02:38:36.876Z
---
Confirm the simpler --workers name has no CLI conflict, rename the content-analysis concurrency option and current help/docs/skill/parity shim consistently, retain automatic zero and explicit scan-pool scope, validate parsing/goldens, and reinstall the resulting CLI and skill.

## Notes

No existing --workers CLI conflict. Rename flag to --workers while preserving content-analysis worker semantics and automatic zero; directory scan concurrency remains its engine-owned pool. Updated CLI help/docs and Python parity adapter; parser test covers new spelling.
