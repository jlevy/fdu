---
type: is
id: is-01m35xrf4gyhcj9q42reapfeen
title: Arbitrate overlapping reconciliation verification even when metadata is unchanged
kind: bug
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-23T01:23:23.407Z
updated_at: 2026-09-23T01:37:52.526Z
---
Recovered state at abc35f96 protects newer omitted issues but an older finishing pass can reinsert errors disproved by a newer clean pass; its conditional facts can also overwrite newer unchanged verification because entry revisions do not move. Track scoped supersession only for active passes, reject stale overlapping observations, suppress only error evidence covered by a newer verification, and preserve disjoint sibling evidence. Deterministic same-scope and ancestor/child regressions plus bounded ownership review required.

## Notes

Implementation prepared on codex/alpha-state-fixes from recovered integration47dbb51c. Red baseline older_pass_cannot_publish_errors_after_newer_clean_verification failed by resurrecting a stale cause. ActiveReconcile now owns normalized newer scopes and an explicit Retry state. Budget is one scope per live entry at pass start (including root, minimum1); a root-only real filesystem fixture and100distinct absent-child unit verifications prove bounded retention. Full ancestor proof collapses descendants; overflow never widens verified scope, discards exact evidence, refuses later batches, preserves prior issues/newer facts, publishes partial state plus retry cause, then returns the private retry result. Existing incomplete-reconciliation retry protocol consumes it; no new public error variant. Finished passes release all evidence. Focused24reconciliation,3overflow,10newer-verification tests pass;5scripted watch handoff tests,4state integrations,10native watch integrations pass; all-features/all-targets core clippy passes. Full no-watch run ran723tests (720pass,2introduced plumbing failures,1existing ignored); both failures fixed and explicitly rerun. Parent owns full make check, stack integration, spec note, PR/CI publication. Retain bead open pending those gates.
