---
type: is
id: is-01kzx1ay5qq8xd5sc790bvdeec
title: "Phase 2c: Expose basic metrics across Rust, CLI, schemas, and Python"
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1ayjgknng1athmvhxp5qy
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:45:38.998Z
updated_at: 2026-08-13T09:05:21.382Z
closed_at: 2026-08-13T09:05:21.382Z
close_reason: Added opt-in Rust, CLI, fdu.report/2, generic extensions/types/families/languages/documents views, exact shares/pages/coverage/provenance, cache lifecycle fields, and Python open/scan/report parity; full workspace and installed cp312-abi3 wheel tests pass.
---
Extend OpenConfig, Query, ViewSpec, CLI parsing, report rows/renderers, and fdu-py with opt-in basic analysis, metric-qualified sorting, generic metrics rows, the documents preset, raw words, and aggregate-derived pages. Emit fdu.report/2 only for type/family/content-capable requests while keeping unchanged metadata output on report/1.
