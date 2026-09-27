---
type: is
id: is-01m3j4pdt0kayyhvgkq50j20dg
title: Unify report remainders and note warn tip perf diagnostics
kind: feature
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/done/plan-2026-09-26-code-analysis-presentation.md
labels: []
dependencies: []
created_at: 2026-09-27T19:15:29.716Z
updated_at: 2026-09-27T21:17:17.694Z
closed_at: 2026-09-27T21:17:17.692Z
close_reason: "Implemented and reviewed in PR #133, restacked through #136. Every local handoff target passed across the full gate and authoritative Linux parity rerun. All 19 checks passed on each updated PR layer; implementation macOS required one unchanged-code retry, tracked separately as fdu-21ns. Installed clean candidate and bundled skill verified; manual acceptance remainder stays under fdu-kwjc."
resolution: null
duplicate_of: null
---
Apply the user-approved note:/warn:/tip:/perf: output design. Keep formatted results on stdout and diagnostics on stderr; make omitted bytes explicit, align gray annotations under filenames, deduplicate tips, and preserve inline debugging details. Document the contract beside shared rendering and diagnostic code, link it from AGENTS.md and README development docs, and keep the architecture guide consistent. Enforce with shared goldens, cross-surface parity, stream separation, ordering, and terminal-color tests. Documentation and focused CLI checks completed; full golden migration and handoff validation remain.
