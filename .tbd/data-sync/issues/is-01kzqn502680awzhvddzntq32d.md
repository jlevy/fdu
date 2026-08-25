---
type: is
id: is-01kzqn502680awzhvddzntq32d
title: "P3: watch scope validation errors"
kind: task
status: open
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47
    at: 2026-08-25T09:54:14.887Z
  - kind: pr
    url: https://github.com/jlevy/metabrowser/pull/74
    at: 2026-08-25T09:54:14.888Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5017522830
    at: 2026-08-25T09:57:37.843Z
labels:
  - pr47-review
  - metabrowser
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-11T05:36:29.253Z
updated_at: 2026-08-25T09:57:37.844Z
closed_at: null
close_reason: null
resolution: null
duplicate_of: null
---
Constraint carried from the engine: watch requires full scope, so --watch combined with --scan-depth or --one-filesystem is a usage error (exit 2) with a message naming the conflict, until validate_for_watch_scope learns otherwise. Selection-axis flags (--depth, --include, --min-size, --modified-since) remain fully legal with --watch, since they filter the retained index rather than narrowing what is observed - that distinction is exactly the scope-versus-selection split and the error message should make it legible. tryscript coverage for each rejected combination and at least one accepted selection-plus-watch combination.

## Notes

Reopened: Reopened as a MetaBrowser Phase 2 adoption gate at FDU d19b0ce. MetaBrowser InventoryConfig always supplies positive max_depth and max_files and expects the selected provider to own the live observer. FDU validate_for_watch_scope rejects any watch with either field (scan.rs:340-350), so the ordinary FDU-backed handle could never enter watching. Implement event admission/reconciliation that preserves the opened semantic scope, or revise both provider designs together; adapter-side second watching or dropping the scope fields is forbidden.
