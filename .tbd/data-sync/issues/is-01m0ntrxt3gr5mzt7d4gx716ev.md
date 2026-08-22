---
type: is
id: is-01m0ntrxt3gr5mzt7d4gx716ev
title: Move one-shot report planning out of the cli feature gate
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-08-22T22:51:58.146Z
updated_at: 2026-08-22T22:51:58.146Z
---
crates/fdu/src/lib.rs: prepare_report and PerformanceSummary are re-exported behind feature="cli", but the Python API is now a non-CLI consumer of that path (fdu-py enables the cli feature to reach it). One-shot planning is an execution strategy, not a command-line concern. Moving it into the core build is a feature-graph change with its own lib-only and MSRV consequences, so it is deliberately separate from the parity work that surfaced it (review R10 on PR #40).
