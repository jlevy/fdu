---
type: is
id: is-01m3jxzb13npp9zm8kxwwb63f7
title: Rename analysis-workers CLI option to workers after scope review
kind: task
status: closed
priority: 2
version: 8
spec_path: docs/project/specs/done/plan-2026-09-26-code-analysis-presentation.md
delegate: codex
labels:
  - cli-presentation
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-28T02:37:16.194Z
updated_at: 2026-09-28T16:20:59.681Z
started_at: 2026-09-28T02:38:36.876Z
closed_at: 2026-09-28T03:27:57.811Z
close_reason: "Implemented and reviewed the unified Code table, analysis/view naming guidance, and --workers. Full make check passed (202 goldens, 914 core and 100 CLI unit tests, 70 Python tests, packaging, parity, 2267 path-independence cases, release and terminal tests). PRs #133, #135, and #136 are pushed and each passed all 19 CI jobs. Installed fdu 0.1.0-dev+g7a499493e with matching skill; 111 installed checks passed. Plan, README, and engineering review updated."
resolution: null
duplicate_of: null
---
Confirm the simpler --workers name has no CLI conflict, rename the content-analysis concurrency option and current help/docs/skill/parity shim consistently, retain automatic zero and explicit scan-pool scope, validate parsing/goldens, and reinstall the resulting CLI and skill.

## Notes

No existing --workers CLI conflict. Rename flag to --workers while preserving content-analysis worker semantics and automatic zero; directory scan concurrency remains its engine-owned pool. Updated CLI help/docs and Python parity adapter; parser test covers new spelling.
