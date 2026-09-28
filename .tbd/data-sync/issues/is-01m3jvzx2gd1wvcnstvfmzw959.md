---
type: is
id: is-01m3jvzx2gd1wvcnstvfmzw959
title: Present code overview as a table with bold complete totals
kind: feature
status: in_progress
priority: 2
version: 8
delegate: codex@spud10.local
labels:
  - cli-presentation
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-28T02:02:37.517Z
updated_at: 2026-09-28T02:59:08.932Z
started_at: 2026-09-28T02:03:07.793Z
---
Present one code overview table with aligned per-language code/comment/blank/share/coverage columns and bold complete totals. Keep selected-population totals independent of row/share limits, distinguish unmeasured from measured zero, retain population details, and preserve structured report schema. Validate via full-output goldens, core formatting assertions, and CLI/Python parity; push and reinstall for local testing.

## Notes

User superseded the two-report split: retain one code report and existing schema, table with bold totals plus per-language metrics and compact coverage. Systematically review lines/code/words measurement versus view naming/header alignment; clarify lines value and redundant combinations. Superseded uncommitted split preserved as patch before restoring only its known files.
