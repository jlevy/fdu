---
type: is
id: is-01kzm514w7qntv14b7msxkk1jw
title: Audit concurrency and thread-safety contracts across fdu
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - concurrency
  - review
dependencies: []
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T20:57:02.598Z
updated_at: 2026-08-09T21:17:55.283Z
closed_at: 2026-08-09T21:17:55.282Z
close_reason: Completed the senior concurrency and thread-safety review at planning commit 24b5488. Traced every current shared-state boundary; found no unsafe/FFI memory unsafety but confirmed three High ownership/liveness issues and one deterministic-evidence gap. Created fdu-1j0b, fdu-8jte, and fdu-gd6n; expanded fdu-s7wr; tightened snapshot, contention, watcher backend, atomic-rollup, and parallel-walk beads; rewired fdu-sn43 to the concurrency gate; synchronized the Phase 1, Rust quality, and research documents; and updated epic order/labels/descriptions. Flowmark, tbd integrity, local make check, and GitHub Actions run 31336339852 pass. Updated the PR description and published the full hold review at https://github.com/jlevy/fdu/pull/1#issuecomment-5233890470. Implementation beads deliberately remain open.
---
Perform a full senior review of every shared-state and cross-thread boundary on PR #1: Index and IndexHandle locking, observation atomicity and logical clocks, journal/subscriber delivery, snapshot load/save and concurrent replacement, scan/reconciliation ownership, watcher startup/worker/shutdown and backend callbacks, CLI/Python access including GIL release, Send/Sync assumptions, poison and panic behavior, lock ordering, backpressure, and future parallel-walk contracts. Prove suspected findings with targeted tests or precise interleavings, distinguish current correctness blockers from pre-release hardening and measured performance work, update the Rust and Phase 1 plans, create or refine implementation beads with real blockers and acceptance tests, then commit, push, and validate CI.

## Notes

Concurrency audit complete. Baseline and final make check pass: 122 all-feature library tests, 2 CLI unit tests, 1 CLI integration test, 2 doctests, 25 golden scenarios, 95 no-default-feature library tests plus doctests, rustdoc, Clippy, Cargo/npm policy, and installed-wheel smoke. No handwritten unsafe, async runtime, manual Send/Sync implementation, extra mutation path, snapshot temp-name race, or PyO3 memory-unsafety finding was found. High findings and owners: public guard retention/self-deadlock (fdu-s7wr); watch fallback symlink_metadata under writer lock (fdu-1j0b); unbounded watcher channels/pending state plus naive bounded-shutdown deadlock (fdu-8jte). Preferred watch design is now explicit: I/O-free bounded coalescer, consumer-side just-in-time verification, bounded apply-if-clock, root invalidation on exhausted contention, nonblocking sticky overflow, cancellation and join. fdu-gd6n owns deterministic final evidence for linearized clocks, before/after batch visibility, freshness epochs, callbacks/I/O after unlock, snapshot old/new visibility, worker lifecycle, Send/Sync ownership, and Python GIL/same-object behavior. fdu-r27g retains std RwLock until measurement; fdu-gdrv/fdu-aky1 require safe-first scoped ownership, memory-ordering proof, bounded work, panic/cancel/join semantics, model checking for custom atomics, and separate evidence before any unsafe boundary. Phase 1 plan, Rust quality plan, original research, root/child epic descriptions, child order, labels, and dependencies now agree. Flowmark, diff checks, tbd integrity, commit/push, PR publication, CI, close, and sync remain.
