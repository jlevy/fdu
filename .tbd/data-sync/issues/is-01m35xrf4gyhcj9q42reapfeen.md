---
type: is
id: is-01m35xrf4gyhcj9q42reapfeen
title: Arbitrate overlapping reconciliation verification even when metadata is unchanged
kind: bug
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-23T01:23:23.407Z
updated_at: 2026-09-23T02:10:01.784Z
---
Recovered state at abc35f96 protects newer omitted issues but an older finishing pass can reinsert errors disproved by a newer clean pass; its conditional facts can also overwrite newer unchanged verification because entry revisions do not move. Track scoped supersession only for active passes, reject stale overlapping observations, suppress only error evidence covered by a newer verification, and preserve disjoint sibling evidence. Deterministic same-scope and ancestor/child regressions plus bounded ownership review required.

## Notes

Follow-up at codex/alpha-serving-handoff-fix from serving5e9f6061: CI exposed interaction with main convergent handoff. Scope arbitration must preserve main holds_target no-op acceptance: overlapping old observations that would change newer facts remain stale, identical targets may settle. Complete older root evidence plus successful newer child evidence may publish root verification; newer failed/partial child remains protected. Existing opened convergent refresh regression covers ordinary file and control-file two-op case. Rust validation awaiting host slot; original bound/cancellation regressions must remain green.
