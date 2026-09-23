---
type: is
id: is-01m35xrf4gyhcj9q42reapfeen
title: Arbitrate overlapping reconciliation verification even when metadata is unchanged
kind: bug
status: closed
priority: 1
version: 8
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-23T01:23:23.407Z
updated_at: 2026-09-23T08:14:06.788Z
closed_at: 2026-09-23T08:14:06.788Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Recovered state at abc35f96 protects newer omitted issues but an older finishing pass can reinsert errors disproved by a newer clean pass; its conditional facts can also overwrite newer unchanged verification because entry revisions do not move. Track scoped supersession only for active passes, reject stale overlapping observations, suppress only error evidence covered by a newer verification, and preserve disjoint sibling evidence. Deterministic same-scope and ancestor/child regressions plus bounded ownership review required.

## Notes

Initial implementation7af768c8: active reconciliation evidence is normalized by ancestor scopes and bounded by retained entry count at pass begin (minimum root1), so zero/small trees have finite budgets. Existing ancestors subsume descendants; exact evidence never widens to a fabricated parent. Non-existent-child adversary triggers explicit cancellation/retry after budget exhausted; unrelated prior issues and newer facts survive, and interrupted old pass cannot publish stale facts. Publication occurs before caller sees retry error. Original focused reconciliation24, overflow3, newer10, scriptedwatch5, publicstate4+nativewatch10 and clippy passed; full gate parent-owned. Follow-up codex/alpha-serving-handoff-fix from serving fa073b0b fixes CI interaction with main convergent handoff. Overlapping observations whose target is already held may settle as no-ops; changed older facts remain stale. Complete older root evidence composes with successful newer child verification, never with failed child evidence. Added failed-child regression; existing opened test covers ordinary and control-file convergence. Awaiting scheduled Rust validation and independent review; do not close yet.

Follow-up validation: da435e22 from serving fa073b0b passed handoff10 and newer-state11 tests, including failed-child preservation and bounded evidence controls. Core all-build-features/all-targets clippy clean. Parent independent Astra review cleared da435e22. Integrated forward by parent; full final candidate gate remains pending. Older bead fdu-ems3 is covered by this scope/version arbitration implementation.
