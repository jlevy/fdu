---
type: is
id: is-01kzv0dfwab6whteads2kzy2f9
title: Revisit producer-side no-op elision with bounded parallel waves (H12)
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzpvt22bex8ed6d155y014py
created_at: 2026-08-12T12:51:05.225Z
updated_at: 2026-08-12T13:18:55.859Z
closed_at: 2026-08-12T13:18:55.859Z
close_reason: Accepted in exp-030. Four-worker bounded immutable-baseline waves improved warm wall 30.25% at 60k and 59.53% at 720k; component time fell 50.31%/72.55%, exact oracles passed, and RSS stayed within the registered bound. Effective deltas apply between waves; shared/scoped/one-worker paths remain serial; deferred overflow retries the full tree through the serial reconciler.
---
Revisit exp-002 after H14 direct no-op elision and exp-026 bulk metadata. For exclusive reconciliation only, scan bounded directory waves against one immutable index baseline in worker threads, emit no operation for exact fingerprint matches, then apply changes through the existing delta contract after workers join. Preserve batch-size limits and streaming between waves; if a wave exceeds the bounded deferred-change budget, rescan that wave through the serial path. Shared reconciliation remains unchanged. Pre-registered 60k gate: warm wall at least 15% lower with CI below zero, reconciliation component at least 25% lower, exact oracle parity, and RSS no more than 10% higher. Confirm at 720k only if the gate passes.
