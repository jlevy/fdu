---
type: is
id: is-01m0hfrm2xbqzdx4avgegcvf0t
title: "Content axis: composable --analyze set and the display contract"
kind: epic
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
created_at: 2026-08-21T06:22:36.118Z
updated_at: 2026-08-21T06:22:36.118Z
---
Phase 5 of the composable CLI spec. `--analyze` shipped after the original five axes and
was never given an axis slot, so nothing connected it to the view axis.

Two defects follow:

1. `fdu --analyze all PATH` reads every eligible file and prints a report byte-identical
   to `fdu PATH` (verified 2026-08-20 on the current build). The default view is `tree`,
   one of the four views that ignore content analysis; only the performance footer
   differs.
2. `AnalysisProfile` flattens a set of independent analyzers into five ordered values.
   The registry (content-basic-v1, code-sloc-v1, text-logical-v1, markdown-prose-v1) and
   the sidecar provenance are already set-shaped, so the flag is the lossy layer. It can
   express 4 of 8 meaningful combinations, and sidecar reuse tests
   `record.profile == request.profile`, so an `all` sidecar forces a complete re-read for
   a later `code` query.

Grammar: `--analyze` becomes a comma-delimited list (none, lines, code, words, all),
`--view` gains `all`, the default view is derived from the analyzer set, and both
directions of Principle 13 report a note rather than an error. Full checklist is in the
spec's Phase 5 section.
