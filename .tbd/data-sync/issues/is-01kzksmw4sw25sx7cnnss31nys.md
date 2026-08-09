---
type: is
id: is-01kzksmw4sw25sx7cnnss31nys
title: Specify the complete CLI cache lifecycle as a golden session
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzksn3gepmk01a21gkxxs6bv
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T17:38:06.104Z
updated_at: 2026-08-09T18:03:09.298Z
closed_at: 2026-08-09T18:03:09.297Z
close_reason: Added a six-open sequential cache golden proving no-cache has no side effect, cold write, unchanged warm revalidation, changed-file warm totals, semantic scope mismatch, corrupt snapshot fail-closed recovery, and replacement. The session passes twice; all 25 golden blocks and make check pass.
---
Add one sequential sandbox session covering no-cache side effects, cold write, unchanged warm revalidation, changed-file warm totals, semantic scope mismatch, and corrupt-snapshot fail-closed recovery.
