---
type: is
id: is-01m3g53gjm5c6ks1t77k1az0qh
title: Add metric sorting and explainable classification to code drill-downs
kind: feature
status: closed
priority: 2
version: 7
spec_path: docs/project/specs/done/plan-2026-09-26-code-analysis-presentation.md
delegate: sol-code-metrics
labels: []
dependencies:
  - type: blocks
    target: is-01m3g53hk9nr8ff52d3xj77ttj
parent_id: is-01m3g53g8b7wg3qk2m70j8h0bp
hold: null
hold_until: null
created_at: 2026-09-27T00:44:09.683Z
updated_at: 2026-09-27T08:18:16.171Z
started_at: 2026-09-27T01:31:24.932Z
closed_at: 2026-09-27T03:12:09.132Z
close_reason: Implemented across engine, CLI, Python, schemas and docs in PR133.183 shared CLI goldens,886 core tests,68 cross-format cases, full16,787-case path matrix, cache fault proofs, and Apple/Windows cross-lint pass. Final packaging/CI handoff remains tracked by fdu-7jtp.
resolution: null
duplicate_of: null
---
Support registered numeric metric sorting starting with code_lines for files and directories; require the analyzer, deterministic ties, unavailable-last ordering, unchanged aggregation scope. Expose existing generated/vendor/documentation flags and heuristic provenance.
