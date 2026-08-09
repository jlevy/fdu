---
type: is
id: is-01kzksmw4sw25sx7cnnss31nys
title: Specify the complete CLI cache lifecycle as a golden session
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzksn3gepmk01a21gkxxs6bv
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T17:38:06.104Z
updated_at: 2026-08-09T17:38:13.645Z
---
Add one sequential sandbox session covering no-cache side effects, cold write, unchanged warm revalidation, changed-file warm totals, semantic scope mismatch, and corrupt-snapshot fail-closed recovery.
