---
type: is
id: is-01m01edt62s6s8mfeyqgykasxq
title: Publish the complete adaptive-worker gap-closure report
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - report
  - documentation
dependencies: []
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:51:22.433Z
updated_at: 2026-08-15T01:16:23.408Z
---
Publish one scoped gap-closure report that starts with the observed Application Support regression, audits why exp-015 through exp-036 and earlier fdu-versus-dust evidence missed it, and links the telemetry, profile, model, corpora, candidate screens, held-out confirmation, implementation or no-change decision, and release-CLI matrix. Link the experiment ledger for sample-level history instead of duplicating it.

Acceptance: report Apple Silicon/local-APFS topology and host coverage, trace-history distributions, exactness and partial-result behavior, statistical verdicts, wall/CPU/RSS/fault/context-switch intervals, installed fdu and dust provenance, residual Intel/non-APFS/cold-state limitations, rejected directions, and named future work. Reconcile the campaign status, platform-tuning guide, README claims, and active plan without implying that one host or corpus represents macOS. A no-production-change result is a valid conclusion. Before closing the epic, resolve every release-blocking child and either complete or explicitly reparent/defer nonblocking H70/H89 work with rationale; validate the soft schema, run docs formatting, and retain the common-document footer.
