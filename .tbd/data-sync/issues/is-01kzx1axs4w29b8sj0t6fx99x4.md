---
type: is
id: is-01kzx1axs4w29b8sj0t6fx99x4
title: "Phase 2b: Integrate content workers, deltas, and sidecar caches"
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1ay5qq8xd5sc790bvdeec
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:45:38.595Z
updated_at: 2026-08-13T08:50:45.402Z
closed_at: 2026-08-13T08:50:45.402Z
close_reason: Implemented bounded content worker pool, conditional file-handle fingerprint checks, sparse incremental rollups, independently versioned atomic sidecars, cache-only semantics, and cache lifecycle integration with focused all-feature tests.
---
Add content/worker.rs, content/index.rs, and content/cache.rs; extend Index with generation/revision/fingerprint-checked apply_analysis and subtract/add invalidation; orchestrate from OpenConfig without a second tree walk; extract reusable atomic cache-file lifecycle; add bounded workers, coverage, cancellation, cache-only/revalidated/scanned provenance, analyzer-local corruption behavior, and integration/cache/race tests.
