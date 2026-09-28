---
type: is
id: is-01m3jxzam3r0kvatpfweb0xvys
title: Align content analyzer names views headers and coverage guidance
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
created_at: 2026-09-28T02:37:15.767Z
updated_at: 2026-09-28T03:18:44.179Z
started_at: 2026-09-28T02:38:36.854Z
---
Review lines/code/words measurements, canonical view names, section headers, and coverage from first principles. Document lines value and implicit overlap; clarify Documents population versus word measurements; give actionable analyzer-as-view diagnostics on CLI and Python. Verify redundant analyzers preserve answers and unsupported SLOC still exposes physical lines.

## Notes

Systematic review: analyzers measure; views group/select. Preserve lines->families, code->code, words->documents; no misleading words alias since Documents is prose/markup subset while words measures other accepted text. Add actionable typed analyzer-as-view errors and mapping docs. lines counts physical text and remains useful alone; implicit in code/words, redundant explicit lines changes no work. Fix incorrect claim that unsupported code has no physical-line metrics. Cross-view Python checks and golden coverage pending.
