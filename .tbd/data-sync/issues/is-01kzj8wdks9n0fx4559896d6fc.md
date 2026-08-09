---
type: is
id: is-01kzj8wdks9n0fx4559896d6fc
title: "PR #1 review R6: Update rollups for all changed observed attributes"
kind: bug
status: closed
priority: 1
version: 3
labels:
  - pr-review
dependencies: []
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:53.144Z
updated_at: 2026-08-09T03:40:03.283Z
closed_at: 2026-08-09T03:40:03.282Z
close_reason: Fixed by using complete Attrs equality for state updates while retaining Fingerprint only as a derived-content reuse key; allocated-byte changes now update stored attributes and ancestor rollups.
---
PR #1 review R6. Files: crates/fdu/src/types.rs, crates/fdu/src/index.rs. Distinguish content-reuse fingerprint equality from full observed-state equality. A change to allocated bytes or device metadata must update stored attributes and affected rollups.
