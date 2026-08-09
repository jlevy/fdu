---
type: is
id: is-01kzm9qxshmd1st8ck5rpjjg02
title: Allow no-op observations at the terminal logical clock
kind: bug
status: closed
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - pr-review
  - correctness
dependencies:
  - type: blocks
    target: is-01kzm3t12dcq5h7n92xztnhcyd
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T22:19:23.313Z
updated_at: 2026-08-09T22:22:39.470Z
closed_at: 2026-08-09T22:22:39.469Z
close_reason: Moved terminal-clock exhaustion behind arbitration using an isolated probe only on the practically unreachable terminal path. No-op and stale observations now return stats without changing clock, journal, index, or rollups; a real mutation remains an atomic typed failure. Added a regression test and all 145 all-feature library tests pass.
---
Index::apply currently computes checked_next before arbitration, so a nonempty observation that is entirely unchanged or stale fails with ClockExhausted even though it would not mint a clock. Compute exhaustion only when an effective mutation exists and add tests proving no-op and stale terminal-clock observations succeed without mutation while real changes fail atomically.
