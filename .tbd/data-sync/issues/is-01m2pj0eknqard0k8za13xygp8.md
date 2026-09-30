---
type: is
id: is-01m2pj0eknqard0k8za13xygp8
title: --analyze code|words buffers whole unknown-type and Markdown files
kind: bug
status: closed
priority: 1
version: 11
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex
labels:
  - content
  - memory
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
hold: null
hold_until: null
created_at: 2026-09-17T02:09:25.618Z
updated_at: 2026-09-30T06:16:10.443Z
started_at: 2026-09-20T04:44:02.087Z
closed_at: 2026-09-30T06:00:23.710Z
close_reason: "Fixed in 91717fbc3eaef374186cc2c1b05ea8bed067ad56. Markdown half: an analysis pass runs under AnalysisLimits{markdown_exact_bytes} = MARKDOWN_EXACT_BYTES (64 MiB, rationale at the constant in content_analysis.rs: rendering needs the whole source plus a document-wide parse; up to the bound a worker costs a few hundred MiB at most; real Markdown is a few MiB so every real file is still rendered exactly). Over the bound the file is counted as plain text in 64 KiB chunks, the words outcome carries CoverageReason::TextOnly with a value (every word visible, blank-line runs as paragraphs), AnalyzerCoverage counts it as analyzed, the row suffix says 'counted as text', JSON words coverage map carries 'text_only', and the report notes 'N Markdown file(s) over 64 MiB counted as plain text: ...'. Sidecar coverage code 7 with value; markdown-prose-v1 analyzer version 2 so cached records from v1 are re-analyzed once; sidecar format and report schema unchanged. Unknown-type half: confirmed by test an_unknown_type_file_retains_at_most_the_prefix_and_a_chunk (1 MiB unknown-type file retains at most the 16 KiB classification prefix plus one 64 KiB chunk; a Markdown file over the bound retains the same; only a Markdown file within the bound holds its source). Tests: a_markdown_file_over_the_exact_bound_is_counted_as_plain_text_and_says_so (record, counts vs plain-text twin, lines/code units unchanged, tally, note, text row, JSON key). Golden tests/golden/cli-content.tryscript.md changed only in the markdown-prose-v1 version line (hand-edited; make test-golden 212 passed, portability ok). CHANGELOG: Fixed entry + Breaking bullet for CoverageReason::TextOnly. Workspace tests green, clippy clean."
resolution: null
duplicate_of: null
---
content_analysis.rs holds the entire file for unknown-type files (.log, .dat, no extension) and for
Markdown. A 100 MiB .out file peaked at 131 MB RSS against 24 MB for `lines`; several workers on
multi-GB logs can exhaust memory. Opt-in and documented as analyzed through EOF. Drop the buffer once
the 16 KiB prefix settles the type; bound Markdown source retention.

## Notes

Follow-up fd9f4a767a884fa18969237c210ff84b7bdd5815: scripts/content-selfcheck.mjs pins the analyzer registry and still expected markdown-prose-v1 version 1, so the first make check on the final tree failed at content-selfcheck; the expectation now reads version 2 and make content-selfcheck passes (1003 tracked files, 829844 text lines, 146329 standard LOC, 39 types).
