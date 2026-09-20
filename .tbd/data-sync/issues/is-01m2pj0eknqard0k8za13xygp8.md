---
type: is
id: is-01m2pj0eknqard0k8za13xygp8
title: --analyze code|words buffers whole unknown-type and Markdown files
kind: bug
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - content
  - memory
dependencies: []
parent_id: is-01m2phzn814exmf4ty5vw6zha0
hold: null
hold_until: null
created_at: 2026-09-17T02:09:25.618Z
updated_at: 2026-09-20T05:17:35.264Z
started_at: 2026-09-20T04:44:02.087Z
---
content_analysis.rs holds the entire file for unknown-type files (.log, .dat, no extension) and for
Markdown. A 100 MiB .out file peaked at 131 MB RSS against 24 MB for `lines`; several workers on
multi-GB logs can exhaust memory. Opt-in and documented as analyzed through EOF. Drop the buffer once
the 16 KiB prefix settles the type; bound Markdown source retention.

## Notes

2026-09-20 delegated implementation: bounded-prefix classification now stops retaining irrelevant unknown-file buffers after the fixed 16 KiB probe; validation pending. Exact Markdown remains a separate unresolved resource design: pulldown-cmark requires a complete source string and builds document-wide parser state, so mmap/spooling alone would not prove bounded resident memory. The engine contract forbids truncating or size-skipping eligible text. User asked whether to include the larger exact redesign or explicitly document/defer it for 0.1; no answer yet. Keep this bead open until both halves are addressed or an explicit scope decision is recorded.

2026-09-20 validation update: 9449aef1 + 7711150f bound unknown-file deferred buffers after classification, with >16KiB and >64KiB shebang/known-extension equivalence under code/words/all. Full core library suite passes. Markdown source and parser state are still unbounded and exact; this issue is not complete.
