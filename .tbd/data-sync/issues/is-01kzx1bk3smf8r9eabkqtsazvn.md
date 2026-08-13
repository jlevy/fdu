---
type: is
id: is-01kzx1bk3smf8r9eabkqtsazvn
title: "Phase 4a: Implement additive logical and structural prose metrics"
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1bkh01q3gvbwpcpxyrfk9
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:46:00.440Z
updated_at: 2026-08-13T10:18:51.914Z
closed_at: 2026-08-13T10:18:51.913Z
close_reason: Implemented additive FlexDoc-compatible logical words with exact rational 3..6 clamp, half-wide weighting, aggregate-before-round, paragraph runs, 250-word pages, and basic/documents depth separation.
---
Implement FlexDoc-compatible LogicalWordStats with integer 3..6 clamps, half-weight wide characters, and round-half-up only after aggregation. Add raw/logical word, paragraph-run, and query-derived page reporting across unfiltered and filtered rollups; pin English, long-token, punctuation, symbolic, spaced multilingual, and CJK cases. Do not promise sentence counts without a separate dialect gate.
