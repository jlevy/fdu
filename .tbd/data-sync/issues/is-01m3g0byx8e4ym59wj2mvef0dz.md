---
type: is
id: is-01m3g0byx8e4ym59wj2mvef0dz
title: Reconcile history replay packaging with the one-build-feature rule
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md
labels: []
dependencies: []
created_at: 2026-09-26T23:21:23.623Z
updated_at: 2026-09-27T00:45:38.753Z
closed_at: 2026-09-27T00:45:38.753Z
close_reason: "Completed in PR #131 (d607d489): reviewed reproducible cross-process probe, explicit opt-in journal freshness contract, reuse of watch build feature, and additional multi-root hour/day disk-history proposal. Full local make check, cross-lint, probe tests, and all CI checks passed. Production replay, long-gap acceptance, and end-to-end large-tree latency remain separately tracked."
resolution: null
duplicate_of: null
---
The active FSEvents plan proposes a new non-default history-replay Cargo build feature, while current repository policy says watch is fdu-core's only optional-capability build feature. Choose and document one design before implementation: fold historical replay into the existing native-observation feature, compile a dependency-free macOS accelerator under cfg(target_os) with runtime fallback, or deliberately revise the one-feature policy. Preserve no-default-features and cross-platform fallback.
