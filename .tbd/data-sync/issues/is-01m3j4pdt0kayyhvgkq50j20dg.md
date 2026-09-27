---
type: is
id: is-01m3j4pdt0kayyhvgkq50j20dg
title: Unify report omissions and note hint perf epilogue
kind: feature
status: in_progress
priority: 1
version: 2
labels: []
dependencies: []
created_at: 2026-09-27T19:15:29.716Z
updated_at: 2026-09-27T19:32:07.077Z
---
Apply the user-approved note:/warn:/tip:/perf: output design. Keep formatted results on stdout and diagnostics on stderr; make omitted bytes explicit, align gray annotations under filenames, deduplicate tips, and preserve inline debugging details. Document the contract beside shared rendering and diagnostic code, link it from AGENTS.md and README development docs, and keep the architecture guide consistent. Enforce with shared goldens, cross-surface parity, stream separation, ordering, and terminal-color tests. Documentation and focused CLI checks completed; full golden migration and handoff validation remain.
