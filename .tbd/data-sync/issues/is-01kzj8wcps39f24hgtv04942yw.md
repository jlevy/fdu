---
type: is
id: is-01kzj8wcps39f24hgtv04942yw
title: "PR #1 review R2: Journal only effective changes within an operation budget"
kind: bug
status: closed
priority: 1
version: 4
labels:
  - pr-review
dependencies:
  - type: blocks
    target: is-01kzj8we2pr4g03f5tdt8n7t08
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:52.216Z
updated_at: 2026-08-09T03:35:03.118Z
closed_at: 2026-08-09T03:35:03.117Z
close_reason: Fixed with effective-only AppliedDelta output, clock-zero scan/snapshot baselines, no-op clock stability, an operation-count journal budget, eviction truncation, and oversized-batch coverage.
---
PR #1 review R2. Files: crates/fdu/src/index.rs, crates/fdu/src/scan.rs, crates/fdu/src/snapshot.rs. Return an AppliedDelta containing only effective mutations, establish bootstrap scans and snapshots as baselines, and bound retained history by operations or bytes including oversized single batches.
