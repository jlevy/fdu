---
type: is
id: is-01kzksmmh4vv5hh349wgtdey8w
title: Expose JSON scan scope and tree projection completeness
kind: bug
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzksmvwnqxbgn4x2q9s8avby
  - type: blocks
    target: is-01kzksmw4sw25sx7cnnss31nys
  - type: blocks
    target: is-01kzksn3gepmk01a21gkxxs6bv
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T17:37:58.307Z
updated_at: 2026-08-09T17:54:35.337Z
closed_at: 2026-08-09T17:54:35.336Z
close_reason: Preserved complete as scan completeness and added display_depth, entries_per_directory, scan_max_depth, and tree_truncated. Added exact full/truncated/restricted-scope JSON sessions plus synthetic depth/number exact-fit boundary tests; goldens pass twice and make check passes.
---
Add exact JSON sessions and synthetic boundary tests. Preserve complete as scan completeness while adding display depth, per-directory row limit, scan max depth, and an accurate tree_truncated signal.
