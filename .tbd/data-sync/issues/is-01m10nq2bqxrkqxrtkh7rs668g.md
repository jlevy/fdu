---
type: is
id: is-01m10nq2bqxrkqxrtkh7rs668g
title: Bind the opened-root value model and PyOpenedIndex in PyO3
kind: task
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
delegate: codex@spud10.local
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nq2s4pvy11rrdrjxyzv2k
parent_id: is-01m0y1sf2nph021wtx28p8ahxh
created_at: 2026-08-27T03:55:13.141Z
updated_at: 2026-08-27T05:21:56.852Z
---
Add crates/fdu-py/src/opened_binding.rs with total conversions for EngineVersion, IndexState, every ReadProjection and ProjectionResult, ChangePoll, RefreshResult, Work, issues, continuations, and typed errors. Register the native OpenedIndex in src/lib.rs; store only the shared native handle; detach the GIL for open, read, changes, refresh, prioritize, and close; add no executor, CLI subprocess, MetaBrowser vocabulary, query aggregation, or duplicate lifecycle.
