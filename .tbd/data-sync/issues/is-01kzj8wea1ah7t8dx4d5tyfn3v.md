---
type: is
id: is-01kzj8wea1ah7t8dx4d5tyfn3v
title: "PR #1 review R9: Reject unsupported symlink-following semantics"
kind: bug
status: closed
priority: 2
version: 4
labels:
  - pr-review
dependencies:
  - type: blocks
    target: is-01kzj8weryq7v9tf11vr9q5psv
  - type: blocks
    target: is-01kzj8wgcxppt8qpvkzw907j0s
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:53.856Z
updated_at: 2026-08-09T03:54:45.342Z
closed_at: 2026-08-09T03:54:45.341Z
close_reason: follow_symlinks=true is now rejected consistently before cold scan, revalidation, or reconciliation until safe semantics exist; regression tests pass.
---
PR #1 review R9. Files: crates/fdu/src/scan.rs, crates/fdu/src/index.rs. Until cycle detection, root-boundary, filesystem-boundary, and coherent entry-kind semantics exist, reject follow_symlinks=true explicitly on cold scan and revalidation.
