---
type: is
id: is-01m0y1se38tcc11akkz34mjrme
title: Add the bounded commit journal and blocking changes poll
kind: feature
status: open
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1seqtkvcawjhawny979ry
  - type: blocks
    target: is-01m0y1sf2nph021wtx28p8ahxh
  - type: blocks
    target: is-01m0yhq8268z0qrza1fnwrddfm
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:30.311Z
updated_at: 2026-08-26T08:07:19.011Z
---
Implement opened-root change polling over the index single bounded exact commit history. Preserve existing nonblocking Index::since as a compatibility view; add session-aware cursor validation, condition-variable wakeup, state-only commits, idle timeout, foreign/future rejection, history reset, close wakeup, bounded invalidations, terminal state, and work without copying commits into a second store.

## Notes

Add deterministic wait/notify barriers through the per-owner test control while implementing the real journal; fdu-0kv7 owns final session traces and coverage closure.
