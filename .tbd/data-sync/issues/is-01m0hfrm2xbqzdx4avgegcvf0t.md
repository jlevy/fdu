---
type: is
id: is-01m0hfrm2xbqzdx4avgegcvf0t
title: "Content axis: composable --analyze set and the display contract"
kind: epic
status: closed
priority: 1
version: 11
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
child_order_hints:
  - is-01m0hg8gavsqypx9sdrk636y8k
  - is-01m0hg8gp6tp30ezekmbdpf53b
  - is-01m0hg8h0t611hf16gbtnzacat
  - is-01m0hg8hb2dn9d9wjntnpd4xzk
  - is-01m0hg9ed5z364cssftdx592jw
  - is-01m0hg9eqv3kghgjvk05dngs2k
  - is-01m0hg9f26pmh0c9vbfswb624n
  - is-01m0hg9fca6p7hs5xw2qp04315
  - is-01m0hg9fppyeggat22bf4m99me
created_at: 2026-08-21T06:22:36.118Z
updated_at: 2026-08-21T07:16:00.042Z
closed_at: 2026-08-21T07:16:00.041Z
close_reason: "Phase 5 complete: content axis, display contract, containment reuse, docs and goldens. make check green."
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
