---
type: is
id: is-01m3fne40s3j5m64r2z6dy6k5c
title: Research concise codebase analysis, SLOC accuracy, and ignored-file scope
kind: task
status: closed
priority: 2
version: 5
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-26T20:10:20.055Z
updated_at: 2026-09-26T22:59:31.622Z
started_at: 2026-09-26T20:11:20.917Z
closed_at: 2026-09-26T22:59:31.620Z
close_reason: "Expanded the codebase-analysis research with a clean alpha design proposal in PR #130: three-valued ignored population, selected-content read avoidance, optional traversal pruning, code overview and metric sorting, cache and coverage rules, and phased acceptance criteria. Full make check and latest-commit CI passed."
resolution: null
duplicate_of: null
---
Write research brief using new-research-brief shortcut. Compare fdu with Tokei empirical study fdu-4il8, GitHub Linguist, and Metabrowser directory overview; propose low-cost incremental accuracy, presentation, ignored-scope and complexity improvements. Documentation only; no engine/default changes.

## Notes

User requested a concrete alpha design proposal without backward-compatibility constraints: clean ignored-file selection, automatic content-read avoidance, explicit traversal scope, defaults, report contract, and incremental implementation plan.
