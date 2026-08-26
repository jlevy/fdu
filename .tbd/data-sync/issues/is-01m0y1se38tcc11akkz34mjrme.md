---
type: is
id: is-01m0y1se38tcc11akkz34mjrme
title: Add the bounded commit journal and blocking changes poll
kind: feature
status: closed
priority: 1
version: 9
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
delegate: codex@spud10.local
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
updated_at: 2026-08-26T14:12:34.856Z
closed_at: 2026-08-26T14:12:34.855Z
close_reason: Bounded exact change polling, cursor validation, idle/reset recovery, close wakeup, and deterministic concurrency coverage implemented; make check passes.
resolution: null
duplicate_of: null
---
Implement opened-root change polling over the index single bounded exact commit history. Preserve existing nonblocking Index::since as a compatibility view; add session-aware cursor validation, condition-variable wakeup, state-only commits, idle timeout, foreign/future rejection, history reset, close wakeup, bounded invalidations, terminal state, and work without copying commits into a second store.

## Notes

Implemented blocking OpenedIndex::changes directly over Index::since and the index-owned bounded exact commit history. EngineVersion is the single cursor type; ChangePoll carries exact commits or idle/reset, bounded aggregate invalidation, coherent terminal version/state, and poll work. OpenOptions now binds journal capacity. JournalWait contains only wait/close synchronization and never copies commits.

Extracted the sound PR #47 concepts from fad3d2f, 1e1c207, 44e79c3, and eaae030 without importing its old session/facade API: one-guard range plus terminal cursor/state, lost-wakeup-safe condition-variable polling, consumer reset distinct from producer recovery, and observable state-only commits.

Review found and fixed missing wake notifications for terminal discovery and root-listing failure transitions. Deterministic barriers cover the commit-before-wait race, terminal state-only wakeup, close wakeup, timeout, foreign/future cursors, eviction reset, bounded invalidation, and detached/opened range identity. make check passes.
