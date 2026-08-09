---
type: is
id: is-01kzm9qxjxbrbcm4nm0mmj21ge
title: Specify watch stat-to-commit linearization and convergence
kind: task
status: closed
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - pr-review
  - concurrency
dependencies:
  - type: blocks
    target: is-01kzm3t12dcq5h7n92xztnhcyd
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T22:19:23.100Z
updated_at: 2026-08-09T22:22:39.279Z
closed_at: 2026-08-09T22:22:39.278Z
close_reason: "Specified the attainable concurrency contract in rustdoc, README, and the Rust-quality plan: filesystem samples linearize at stat; no index lock spans I/O; later events remain queued; reported loss or ambiguity invalidates and reconciles; logical clocks arbitrate in-memory commits only. Existing deterministic blocked-verifier, bounded-queue, overflow, and reconciliation tests pass in the 145-test all-feature suite."
---
Adjudicate the PR finding about a filesystem mutation between stat and index commit. State and test the attainable contract: filesystem I/O never occurs under an index lock; the committed sample is valid at its stat linearization point; backend events arriving during or after verification remain queued; overflow or ambiguity invalidates and reconciles. Do not claim the process can freeze external filesystem mutation or add double-stat work without measured benefit.
