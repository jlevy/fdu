---
type: is
id: is-01m3g0byx8e4ym59wj2mvef0dz
title: Reconcile history replay packaging with the one-build-feature rule
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md
labels: []
dependencies: []
created_at: 2026-09-26T23:21:23.623Z
updated_at: 2026-09-26T23:21:23.623Z
---
The active FSEvents plan proposes a new non-default history-replay Cargo build feature, while current repository policy says watch is fdu-core's only optional-capability build feature. Choose and document one design before implementation: fold historical replay into the existing native-observation feature, compile a dependency-free macOS accelerator under cfg(target_os) with runtime fallback, or deliberately revise the one-feature policy. Preserve no-default-features and cross-platform fallback.
