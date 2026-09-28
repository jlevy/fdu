---
type: is
id: is-01m3jxzam3r0kvatpfweb0xvys
title: Align content analyzer names views headers and coverage guidance
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
created_at: 2026-09-28T02:37:15.767Z
updated_at: 2026-09-28T16:20:59.014Z
started_at: 2026-09-28T02:38:36.854Z
closed_at: 2026-09-28T03:27:57.805Z
close_reason: "Implemented and reviewed the unified Code table, analysis/view naming guidance, and --workers. Full make check passed (202 goldens, 914 core and 100 CLI unit tests, 70 Python tests, packaging, parity, 2267 path-independence cases, release and terminal tests). PRs #133, #135, and #136 are pushed and each passed all 19 CI jobs. Installed fdu 0.1.0-dev+g7a499493e with matching skill; 111 installed checks passed. Plan, README, and engineering review updated."
resolution: null
duplicate_of: null
---
Review lines/code/words measurements, canonical view names, section headers, and coverage from first principles. Document lines value and implicit overlap; clarify Documents population versus word measurements; give actionable analyzer-as-view diagnostics on CLI and Python. Verify redundant analyzers preserve answers and unsupported SLOC still exposes physical lines.

## Notes

Systematic review: analyzers measure; views group/select. Preserve lines->families, code->code, words->documents; no misleading words alias since Documents is prose/markup subset while words measures other accepted text. Add actionable typed analyzer-as-view errors and mapping docs. lines counts physical text and remains useful alone; implicit in code/words, redundant explicit lines changes no work. Fix incorrect claim that unsupported code has no physical-line metrics. Cross-view Python checks and golden coverage pending.
