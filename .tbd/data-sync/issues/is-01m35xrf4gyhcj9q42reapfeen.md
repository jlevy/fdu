---
type: is
id: is-01m35xrf4gyhcj9q42reapfeen
title: Arbitrate overlapping reconciliation verification even when metadata is unchanged
kind: bug
status: in_progress
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-23T01:23:23.407Z
updated_at: 2026-09-23T01:23:40.410Z
---
Recovered state at abc35f96 protects newer omitted issues but an older finishing pass can reinsert errors disproved by a newer clean pass; its conditional facts can also overwrite newer unchanged verification because entry revisions do not move. Track scoped supersession only for active passes, reject stale overlapping observations, suppress only error evidence covered by a newer verification, and preserve disjoint sibling evidence. Deterministic same-scope and ancestor/child regressions plus bounded ownership review required.
