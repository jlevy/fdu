---
type: is
id: is-01kzj8we2pr4g03f5tdt8n7t08
title: "PR #1 review R8: Add reconciliation driver, freshness state, and subtree repair"
kind: bug
status: closed
priority: 2
version: 6
labels:
  - pr-review
dependencies:
  - type: blocks
    target: is-01kzj8wdcey7qb1bkq7y9m2f3q
  - type: blocks
    target: is-01kzj8wf0jzt8asjpae7eynvp1
  - type: blocks
    target: is-01kzj8wgcxppt8qpvkzw907j0s
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:53.621Z
updated_at: 2026-08-09T03:54:45.167Z
closed_at: 2026-08-09T03:54:45.166Z
close_reason: Implemented applying full/subtree/shared reconciliation, effective delta publication, freshness states with epoch-safe invalidation handling, and watch invalidation closure; focused and workspace tests pass.
---
PR #1 review R8. Files: crates/fdu/src/scan.rs, crates/fdu/src/lib.rs, crates/fdu/src/index.rs, crates/fdu/src/watch.rs. Provide an owning reconciliation path that applies and publishes effective deltas while scanning, expose freshness, and close InvalidateSubtree through a subtree reconciliation API. Document any remaining asynchronous tier as future work.
