---
type: is
id: is-01m3jvzx2gd1wvcnstvfmzw959
title: Present code overview as a table with bold complete totals
kind: feature
status: closed
priority: 2
version: 11
spec_path: docs/project/specs/done/plan-2026-09-26-code-analysis-presentation.md
delegate: codex
labels:
  - cli-presentation
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-28T02:02:37.517Z
updated_at: 2026-09-28T16:20:59.361Z
started_at: 2026-09-28T02:03:07.793Z
closed_at: 2026-09-28T03:27:57.787Z
close_reason: "Implemented and reviewed the unified Code table, analysis/view naming guidance, and --workers. Full make check passed (202 goldens, 914 core and 100 CLI unit tests, 70 Python tests, packaging, parity, 2267 path-independence cases, release and terminal tests). PRs #133, #135, and #136 are pushed and each passed all 19 CI jobs. Installed fdu 0.1.0-dev+g7a499493e with matching skill; 111 installed checks passed. Plan, README, and engineering review updated."
resolution: null
duplicate_of: null
---
Present one code overview table with aligned per-language code/comment/blank/share/coverage columns and bold complete totals. Keep selected-population totals independent of row/share limits, distinguish unmeasured from measured zero, retain population details, and preserve structured report schema. Validate via full-output goldens, core formatting assertions, and CLI/Python parity; push and reinstall for local testing.

## Notes

User superseded the two-report split: retain one code report and existing schema, table with bold totals plus per-language metrics and compact coverage. Systematically review lines/code/words measurement versus view naming/header alignment; clarify lines value and redundant combinations. Superseded uncommitted split preserved as patch before restoring only its known files.
