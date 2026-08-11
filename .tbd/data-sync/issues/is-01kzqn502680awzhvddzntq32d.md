---
type: is
id: is-01kzqn502680awzhvddzntq32d
title: "P3: watch scope validation errors"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:36:29.253Z
updated_at: 2026-08-11T05:36:29.253Z
---
Constraint carried from the engine: watch requires full scope, so --watch combined with --scan-depth or --one-filesystem is a usage error (exit 2) with a message naming the conflict, until validate_for_watch_scope learns otherwise. Selection-axis flags (--depth, --include, --min-size, --modified-since) remain fully legal with --watch, since they filter the retained index rather than narrowing what is observed - that distinction is exactly the scope-versus-selection split and the error message should make it legible. tryscript coverage for each rejected combination and at least one accepted selection-plus-watch combination.
