---
type: is
id: is-01kzyqkmswz9qha4knttpt9xzw
title: Expose bounded per-path content I/O diagnostics in reports
kind: bug
status: closed
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
hold: null
hold_until: null
created_at: 2026-08-13T23:34:07.404Z
updated_at: 2026-09-23T08:14:06.878Z
started_at: 2026-09-20T05:33:22.477Z
closed_at: 2026-09-23T08:14:06.878Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
FileAnalysis retains a bounded I/O error string, but grouped machine reports expose only aggregate coverage and the envelope receives only a generic category count. Add bounded per-path content diagnostics or explicitly revise the completed spec.

## Notes

Recovered typed-answer/status layer PR113 derives bounded path-specific content issues in TreeStatus::of, including retained error text, deterministic bounded details, and omitted counts. Content admission follow-up in PR115 keeps status tied to requested identity. Independently reviewed; final composed gate/CI pending.

2026-09-22 acceptance checkpoint: the typed-answer stack still carries bounded per-path content I/O diagnostics in status. Recent guide, parser, golden, and portability fixture corrections are test/documentation-only; they do not alter diagnostic runtime behavior. Final current-head validation remains pending.
