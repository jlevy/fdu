---
type: is
id: is-01kzqn502680awzhvddzntq32d
title: "P3: watch scope validation errors"
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:36:29.253Z
updated_at: 2026-08-11T17:08:48.090Z
closed_at: 2026-08-11T17:08:48.090Z
close_reason: Session composes IndexHandle+Watcher+Query; --watch loop with fdu.stream/1 tagged change records, dirty-gated aggregate repaint, and event-driven detection throughout. Selection filters the stream, with removals filtered only by path and escalations never filtered. --watch + --scan-depth is a usage error that teaches scope-vs-selection. watch joins default features while cli-only and no-default-features still build. 5 integration tests against real filesystem events, including the idle-cost contract.
---
Constraint carried from the engine: watch requires full scope, so --watch combined with --scan-depth or --one-filesystem is a usage error (exit 2) with a message naming the conflict, until validate_for_watch_scope learns otherwise. Selection-axis flags (--depth, --include, --min-size, --modified-since) remain fully legal with --watch, since they filter the retained index rather than narrowing what is observed - that distinction is exactly the scope-versus-selection split and the error message should make it legible. tryscript coverage for each rejected combination and at least one accepted selection-plus-watch combination.
