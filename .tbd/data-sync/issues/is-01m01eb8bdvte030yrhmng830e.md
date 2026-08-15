---
type: is
id: is-01m01eb8bdvte030yrhmng830e
title: Qualify partial macOS scans without masking permission errors
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - macos
  - validation
dependencies:
  - type: blocks
    target: is-01m01ed61j7yty2bqp0zw8v0xc
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:49:58.636Z
updated_at: 2026-08-15T01:16:22.420Z
---
Qualify partial macOS scans without weakening fdu’s strict partial-result semantics. Use a deterministic portable permission-denial fixture where the platform can enforce it, and keep the live TCC-protected Application Support reproduction as confidential machine-specific diagnostics. Compare fdu, fdu --allow-partial, and validated dust error behavior for unreadable-directory sets, totals over readable entries, exits, warning-rendering cost, and policy/backend traces.

Acceptance: deterministic EACCES-style tests are distinct from TCC observations; fdu exposes every skipped directory and distinguishes partial from complete success; machine output retains full detail even when human summaries are aggregated; no error-bearing, mutable, or TCC-specific sample supports a speed claim; unavailable permission capabilities are null plus a reason; any UX proposal coordinates with fdu-oqoy rather than adopting dust’s exit/warning behavior implicitly.
